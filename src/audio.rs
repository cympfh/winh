use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

pub struct AudioRecorder {
    stream: Option<cpal::Stream>,
    sample_rate: u32,
    last_sound_time: Arc<Mutex<Instant>>,
    silence_threshold: f32,
    recording_start_time: Arc<Mutex<Option<Instant>>>,
    current_max_amplitude: Arc<Mutex<f32>>,
}

impl AudioRecorder {
    pub fn new(silence_threshold: f32) -> Result<Self, String> {
        Ok(Self {
            stream: None,
            sample_rate: 0,
            last_sound_time: Arc::new(Mutex::new(Instant::now())),
            silence_threshold,
            recording_start_time: Arc::new(Mutex::new(None)),
            current_max_amplitude: Arc::new(Mutex::new(0.0)),
        })
    }

    pub fn get_max_amplitude(&self) -> f32 {
        *self.current_max_amplitude.lock().unwrap()
    }

    pub fn is_silent(&self, silence_duration_secs: f32) -> bool {
        let start_time = self.recording_start_time.lock().unwrap();
        if let Some(start) = *start_time {
            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(3) {
                return false;
            }
        }

        let last_sound = self.last_sound_time.lock().unwrap();
        let silence_duration = last_sound.elapsed();
        let is_silent = silence_duration >= Duration::from_secs_f32(silence_duration_secs);

        if is_silent {
            println!(
                "SILENT DETECTED: {:.1}s >= {:.1}s",
                silence_duration.as_secs_f32(),
                silence_duration_secs
            );
        }

        is_silent
    }

    pub fn get_silence_duration(&self) -> Duration {
        let last_sound = self.last_sound_time.lock().unwrap();
        last_sound.elapsed()
    }

    pub fn reset_silence_timer(&self) {
        let mut last_sound = self.last_sound_time.lock().unwrap();
        *last_sound = Instant::now();
    }

    pub fn get_recording_duration(&self) -> f32 {
        let start_time = self.recording_start_time.lock().unwrap();
        if let Some(start) = *start_time {
            start.elapsed().as_secs_f32()
        } else {
            0.0
        }
    }

    pub fn start_recording_with_device(
        &mut self,
        device_name: Option<&str>,
        chunk_sender: Option<UnboundedSender<Vec<f32>>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();

        let device = if let Some(name) = device_name {
            host.input_devices()
                .map_err(|e| format!("Failed to get input devices: {}", e))?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or(format!("Input device '{}' not found", name))?
        } else {
            host.default_input_device()
                .ok_or("No input device available")?
        };

        println!("Using input device: {}", device.name().unwrap_or_default());

        let default_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let mono_config = cpal::StreamConfig {
            channels: 1,
            sample_rate: default_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        self.reset_silence_timer();

        {
            let mut max_amp = self.current_max_amplitude.lock().unwrap();
            *max_amp = 0.0;
        }

        {
            let mut start_time = self.recording_start_time.lock().unwrap();
            *start_time = Some(Instant::now());
        }

        let last_sound_clone = Arc::clone(&self.last_sound_time);
        let max_amplitude_clone = Arc::clone(&self.current_max_amplitude);
        let threshold = self.silence_threshold;
        let chunk_size = (mono_config.sample_rate.0 as f32 * 0.1) as usize; // 100ms

        let stream_result = match default_config.sample_format() {
            cpal::SampleFormat::F32 => self.build_input_stream::<f32>(
                &device,
                &mono_config,
                last_sound_clone.clone(),
                max_amplitude_clone.clone(),
                threshold,
                chunk_sender.clone(),
                chunk_size,
            ),
            cpal::SampleFormat::I16 => self.build_input_stream::<i16>(
                &device,
                &mono_config,
                last_sound_clone.clone(),
                max_amplitude_clone.clone(),
                threshold,
                chunk_sender.clone(),
                chunk_size,
            ),
            cpal::SampleFormat::U16 => self.build_input_stream::<u16>(
                &device,
                &mono_config,
                last_sound_clone.clone(),
                max_amplitude_clone.clone(),
                threshold,
                chunk_sender.clone(),
                chunk_size,
            ),
            _ => return Err("Unsupported sample format".to_string()),
        };

        let stream = match stream_result {
            Ok(stream) => {
                self.sample_rate = mono_config.sample_rate.0;
                println!(
                    "Sample rate: {}Hz, Channels: 1 (forced mono), Format: {:?}",
                    self.sample_rate,
                    default_config.sample_format()
                );
                stream
            }
            Err(e) => {
                println!(
                    "Mono config not supported ({}), falling back to default config",
                    e
                );
                let default_stream_config = default_config.config();
                self.sample_rate = default_stream_config.sample_rate.0;
                let channels = default_stream_config.channels;
                let fallback_chunk_size = (default_stream_config.sample_rate.0 as f32 * 0.1) as usize;

                println!(
                    "Sample rate: {}Hz, Channels: {} (using default), Format: {:?}",
                    self.sample_rate,
                    channels,
                    default_config.sample_format()
                );

                match default_config.sample_format() {
                    cpal::SampleFormat::F32 => self.build_input_stream_with_channels::<f32>(
                        &device,
                        &default_stream_config,
                        last_sound_clone,
                        max_amplitude_clone,
                        threshold,
                        channels,
                        chunk_sender,
                        fallback_chunk_size,
                    ),
                    cpal::SampleFormat::I16 => self.build_input_stream_with_channels::<i16>(
                        &device,
                        &default_stream_config,
                        last_sound_clone,
                        max_amplitude_clone,
                        threshold,
                        channels,
                        chunk_sender,
                        fallback_chunk_size,
                    ),
                    cpal::SampleFormat::U16 => self.build_input_stream_with_channels::<u16>(
                        &device,
                        &default_stream_config,
                        last_sound_clone,
                        max_amplitude_clone,
                        threshold,
                        channels,
                        chunk_sender,
                        fallback_chunk_size,
                    ),
                    _ => return Err("Unsupported sample format".to_string()),
                }?
            }
        };

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) {
        // streamをdropするとコールバッククロージャがdrop → chunk_senderがdrop
        // → UnboundedReceiver側がdisconnectを検知 → WebSocketタスクがaudio.doneを送信
        self.stream = None;
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[allow(clippy::too_many_arguments)]
    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        last_sound_time: Arc<Mutex<Instant>>,
        current_max_amplitude: Arc<Mutex<f32>>,
        threshold: f32,
        chunk_sender: Option<UnboundedSender<Vec<f32>>>,
        chunk_size: usize,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);
        let mut local_chunk: Vec<f32> = Vec::with_capacity(chunk_size);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let mut has_sound = false;
                    let mut max_amplitude = 0.0f32;

                    for &sample in data.iter() {
                        let sample_f32: f32 = cpal::Sample::from_sample(sample);
                        let abs_sample = sample_f32.abs();
                        max_amplitude = max_amplitude.max(abs_sample);

                        if abs_sample > threshold {
                            has_sound = true;
                        }

                        local_chunk.push(sample_f32);
                        if local_chunk.len() >= chunk_size {
                            if let Some(ref tx) = chunk_sender {
                                let _ = tx.send(local_chunk.drain(..).collect());
                            } else {
                                local_chunk.clear();
                            }
                        }
                    }

                    {
                        let mut current_max = current_max_amplitude.lock().unwrap();
                        *current_max = current_max.max(max_amplitude) * 0.95;
                    }

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

    #[allow(clippy::too_many_arguments)]
    fn build_input_stream_with_channels<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        last_sound_time: Arc<Mutex<Instant>>,
        current_max_amplitude: Arc<Mutex<f32>>,
        threshold: f32,
        channels: u16,
        chunk_sender: Option<UnboundedSender<Vec<f32>>>,
        chunk_size: usize,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);
        let mut local_chunk: Vec<f32> = Vec::with_capacity(chunk_size);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let mut has_sound = false;
                    let mut max_amplitude = 0.0f32;

                    let mono_samples: Vec<f32> = if channels == 1 {
                        data.iter()
                            .map(|&s| <f32 as cpal::Sample>::from_sample(s))
                            .collect()
                    } else {
                        data.chunks_exact(channels as usize)
                            .map(|chunk| {
                                let sum: f32 = chunk
                                    .iter()
                                    .map(|&s| <f32 as cpal::Sample>::from_sample(s))
                                    .sum();
                                sum / channels as f32
                            })
                            .collect()
                    };

                    for mono_sample in mono_samples {
                        let abs_sample = mono_sample.abs();
                        max_amplitude = max_amplitude.max(abs_sample);
                        if abs_sample > threshold {
                            has_sound = true;
                        }

                        local_chunk.push(mono_sample);
                        if local_chunk.len() >= chunk_size {
                            if let Some(ref tx) = chunk_sender {
                                let _ = tx.send(local_chunk.drain(..).collect());
                            } else {
                                local_chunk.clear();
                            }
                        }
                    }

                    {
                        let mut current_max = current_max_amplitude.lock().unwrap();
                        *current_max = current_max.max(max_amplitude) * 0.95;
                    }

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
        Self::new(0.01).unwrap()
    }
}

/// Get list of available input devices
pub fn get_input_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to get input devices: {}", e))?;

    let mut device_names = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            device_names.push(name);
        }
    }

    Ok(device_names)
}
