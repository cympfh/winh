use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::PathBuf;

pub struct AudioRecorder {
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    sample_rate: u32,
    last_sound_time: Arc<Mutex<Instant>>,
    silence_threshold: f32,
    recording_start_time: Arc<Mutex<Option<Instant>>>,
}

pub struct SilenceDetector {
    threshold: f32,
    silence_duration: Duration,
    last_sound_time: Arc<Mutex<Instant>>,
}

impl SilenceDetector {
    pub fn new(threshold: f32, silence_duration_secs: f32) -> Self {
        Self {
            threshold,
            silence_duration: Duration::from_secs_f32(silence_duration_secs),
            last_sound_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn is_silent(&self) -> bool {
        let last_sound = self.last_sound_time.lock().unwrap();
        last_sound.elapsed() >= self.silence_duration
    }

    pub fn update_with_samples(&self, samples: &[f32]) {
        // Check if any sample exceeds the threshold
        let has_sound = samples.iter().any(|&sample| sample.abs() > self.threshold);

        if has_sound {
            let mut last_sound = self.last_sound_time.lock().unwrap();
            *last_sound = Instant::now();
        }
    }

    pub fn get_silence_duration(&self) -> Duration {
        let last_sound = self.last_sound_time.lock().unwrap();
        last_sound.elapsed()
    }

    pub fn reset(&self) {
        let mut last_sound = self.last_sound_time.lock().unwrap();
        *last_sound = Instant::now();
    }
}

impl AudioRecorder {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            sample_rate: 0,
            last_sound_time: Arc::new(Mutex::new(Instant::now())),
            silence_threshold: 0.01, // Default threshold
            recording_start_time: Arc::new(Mutex::new(None)),
        })
    }

    pub fn new_with_threshold(silence_threshold: f32) -> Result<Self, String> {
        Ok(Self {
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            sample_rate: 0,
            last_sound_time: Arc::new(Mutex::new(Instant::now())),
            silence_threshold,
            recording_start_time: Arc::new(Mutex::new(None)),
        })
    }

    pub fn is_silent(&self, silence_duration_secs: f32) -> bool {
        // Check if we're still in the grace period (3 seconds after recording starts)
        let start_time = self.recording_start_time.lock().unwrap();
        if let Some(start) = *start_time {
            if start.elapsed() < Duration::from_secs(3) {
                // Still in grace period, not silent
                return false;
            }
        }

        let last_sound = self.last_sound_time.lock().unwrap();
        last_sound.elapsed() >= Duration::from_secs_f32(silence_duration_secs)
    }

    pub fn get_silence_duration(&self) -> Duration {
        let last_sound = self.last_sound_time.lock().unwrap();
        last_sound.elapsed()
    }

    pub fn reset_silence_timer(&self) {
        let mut last_sound = self.last_sound_time.lock().unwrap();
        *last_sound = Instant::now();
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        // Get the default host
        let host = cpal::default_host();

        // Get the default input device
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        println!("Using input device: {}", device.name().unwrap_or_default());

        // Get the default input config
        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        self.sample_rate = config.sample_rate().0;
        println!("Sample rate: {}", self.sample_rate);

        // Clear previous buffer and reset silence timer
        {
            let mut buffer = self.audio_buffer.lock().unwrap();
            buffer.clear();
        }
        self.reset_silence_timer();

        // Set recording start time for grace period
        {
            let mut start_time = self.recording_start_time.lock().unwrap();
            *start_time = Some(Instant::now());
        }

        // Create the input stream
        let buffer_clone = Arc::clone(&self.audio_buffer);
        let last_sound_clone = Arc::clone(&self.last_sound_time);
        let threshold = self.silence_threshold;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_input_stream::<f32>(&device, &config.into(), buffer_clone, last_sound_clone, threshold),
            cpal::SampleFormat::I16 => self.build_input_stream::<i16>(&device, &config.into(), buffer_clone, last_sound_clone, threshold),
            cpal::SampleFormat::U16 => self.build_input_stream::<u16>(&device, &config.into(), buffer_clone, last_sound_clone, threshold),
            _ => return Err("Unsupported sample format".to_string()),
        }?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        // Stop and drop the stream
        self.stream = None;

        // Return the recorded audio data
        let buffer = self.audio_buffer.lock().unwrap();
        buffer.clone()
    }

    pub fn get_buffer_size(&self) -> usize {
        let buffer = self.audio_buffer.lock().unwrap();
        buffer.len()
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<Mutex<Vec<f32>>>,
        last_sound_time: Arc<Mutex<Instant>>,
        threshold: f32,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();
                    let mut has_sound = false;

                    for &sample in data.iter() {
                        let sample_f32: f32 = cpal::Sample::from_sample(sample);
                        buffer.push(sample_f32);

                        // Check if this sample exceeds the silence threshold
                        if sample_f32.abs() > threshold {
                            has_sound = true;
                        }
                    }

                    // Update last sound time if sound was detected
                    if has_sound {
                        let mut last_sound = last_sound_time.lock().unwrap();
                        *last_sound = Instant::now();
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        Ok(stream)
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

pub fn save_audio_to_wav(audio_data: &[f32], sample_rate: u32) -> Result<PathBuf, String> {
    // Trim leading silence but keep 0.2 seconds
    let silence_threshold = 0.01;
    let keep_samples = (sample_rate as f32 * 0.2) as usize; // 0.2 seconds worth of samples

    let trimmed_data = trim_leading_silence(audio_data, silence_threshold, keep_samples);

    if trimmed_data.is_empty() {
        return Err("Audio data is empty after trimming".to_string());
    }

    // Create a temporary file
    let temp_file = tempfile::Builder::new()
        .prefix("winh_audio_")
        .suffix(".wav")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let temp_path = temp_file.path().to_path_buf();

    // Create WAV writer
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&temp_path, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    // Write samples
    for &sample in trimmed_data {
        // Convert f32 to i16
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer.finalize()
        .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

    // Keep the file alive by forgetting the tempfile handle
    std::mem::forget(temp_file);

    println!("Audio saved to: {:?} (trimmed {} samples from start)", temp_path, audio_data.len() - trimmed_data.len());
    Ok(temp_path)
}

fn trim_leading_silence(audio_data: &[f32], threshold: f32, keep_samples: usize) -> &[f32] {
    // Find the first sample that exceeds the threshold
    let first_sound_idx = audio_data.iter()
        .position(|&sample| sample.abs() > threshold)
        .unwrap_or(0);

    // Calculate start index: go back by keep_samples, but not before 0
    let start_idx = first_sound_idx.saturating_sub(keep_samples);

    &audio_data[start_idx..]
}
