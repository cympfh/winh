# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**winh** is a Windows 11 voice transcription GUI application written in Rust. It provides a single-window interface with a start/stop button that captures audio input, automatically detects silence, and transcribes speech to text using OpenAI's Whisper API. The transcribed text is displayed in the GUI and copied to the clipboard.

### Key Features (Implemented)
- Single executable file (~30MB, no installer required)
- Cross-platform compilation (can be built on WSL/Linux, runs on Windows 11)
- Voice input using Windows default audio input (forced to mono)
- Input device selection (choose from available audio input devices)
- Global hotkey support (default: Ctrl+Shift+H, customizable)
  - Works even when application is not focused
  - Starts recording when pressed (does not toggle)
- Automatic silence detection with configurable duration (default 2 seconds)
- Grace period (3 seconds) after recording starts before silence detection
- Leading silence trimming (keeps 0.2 seconds)
- Visual feedback:
  - Silence progress indicator (cyan fill on Stop button)
  - Real-time volume level indicator (gray/green/red bar)
- Audio saved temporarily as WAV and sent to OpenAI for transcription
- Settings modal with scrollable content:
  - API key management
  - Model selection (default: gpt-4o-transcribe)
  - Silence duration (0.5-10.0 seconds)
  - Silence threshold (0.001-0.3, logarithmic scale)
  - Input device selection
  - Hotkey customization (Ctrl/Shift/Alt/Super + A-Z/F1-F12)
- Transcribed text area (read-only, click to copy to clipboard)
- Local settings persistence (JSON format in user config directory)
- Japanese font support (Noto Sans JP)
- Background transcription (non-blocking UI)
- Error handling and display

## Development Commands

### Using Makefile (Recommended)
```bash
# Show available targets
make help

# Install dependencies (mingw-w64)
make install-deps

# Add Windows target
make setup-windows-target

# Build debug version for Windows
make build-windows

# Build release version for Windows
make build-windows-release

# Clean build artifacts
make clean
```

### Manual Building
```bash
# For Windows target (from WSL/Linux)
cargo build --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# For local target (testing only on Linux)
cargo build
cargo run
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features
```

## Project Status

v0.1.0 released (2025-12-18). v0.2.0 is in progress with additional features. See TODO.md for detailed development history.

### Completed Phases
- ✅ Phase 1: GUI framework (egui/eframe)
- ✅ Phase 2: Audio input (cpal)
- ✅ Phase 3: Audio processing and silence detection
- ✅ Phase 4: OpenAI Whisper API integration
- ✅ Phase 5: Clipboard integration
- ✅ Phase 6: Settings management and local storage
- ✅ Phase 6.5: Audio processing improvements
- ✅ Phase 7: Refactoring, CI/CD, visual feedback, license

### v0.2.0 Features (Completed)
- ✅ Transcribed text area with click-to-copy
- ✅ Input device selection
- ✅ Global hotkey support (Ctrl+Shift+H)
- ✅ Customizable hotkey configuration

## Architecture

### Project Structure
```
src/
├── main.rs         # Main application, GUI, state management
├── audio.rs        # Audio recording, silence detection, WAV export
├── config.rs       # Configuration management (JSON persistence)
├── openai.rs       # OpenAI Whisper API client
└── icon.png        # Application icon (embedded)

fonts/
└── NotoSansJP-Regular.ttf  # Japanese font (embedded)

.github/workflows/
├── ci.yml          # CI: format check, clippy, tests, Windows build
└── release.yml     # CD: automated releases on tags
```

### Core Components (Implemented)

1. **Audio Capture** (audio.rs)
   - Uses `cpal` for cross-platform audio input
   - Forces mono (1 channel) recording to avoid stereo issues
   - Real-time amplitude monitoring with exponential decay
   - Thread-safe buffer management with Arc<Mutex<>>

2. **Silence Detection** (audio.rs)
   - Configurable threshold (0.001-0.3)
   - Configurable duration (0.5-10.0 seconds)
   - Grace period (3 seconds) after recording starts
   - Real-time silence duration tracking

3. **Audio Processing** (audio.rs)
   - WAV format export using `hound`
   - Leading silence trimming (keeps 0.2 seconds)
   - 16-bit PCM, mono, variable sample rate
   - Temporary file management with `tempfile`

4. **API Integration** (openai.rs)
   - Blocking HTTP client using `reqwest`
   - Multipart form upload for audio files
   - Error handling and response parsing
   - Configurable model selection

5. **Clipboard Management** (main.rs)
   - Automatic clipboard copy on transcription success
   - Uses `arboard` for cross-platform clipboard access
   - Error handling for clipboard failures

6. **Settings Management** (config.rs)
   - JSON persistence using `serde_json`
   - Config file location: OS-specific config directory
   - Command-line argument support
   - Default values for all settings
   - Hotkey parsing (string format to HotKey object)

7. **Global Hotkey Management** (main.rs)
   - Uses `global-hotkey` crate for system-wide hotkey registration
   - Dynamic hotkey registration/unregistration
   - Works even when application is not focused
   - Periodic UI updates (100ms) to detect hotkey events
   - Only triggers when not recording and not transcribing

8. **Input Device Selection** (audio.rs, main.rs)
   - Enumerates available input devices via `cpal`
   - Device selection stored in configuration
   - ComboBox UI for device selection
   - Falls back to default device if specified device unavailable

9. **GUI State Management** (main.rs)
   - Single-window application with egui
   - Custom button rendering with progress indicators
   - Real-time visual feedback (silence progress, volume meter)
   - Modal settings dialog with scrollable content
   - Background transcription using channels
   - Japanese font support
   - Read-only transcribed text area with click-to-copy

### Key Implementation Details

#### Mono Recording Fix
The application forces mono (1 channel) recording by creating a custom `StreamConfig`:
```rust
let config = cpal::StreamConfig {
    channels: 1,
    sample_rate: default_config.sample_rate(),
    buffer_size: cpal::BufferSize::Default,
};
```
This prevents stereo audio from being saved as mono, which caused slow playback issues.

#### Visual Feedback
- **Silence Progress**: Stop button bottom is filled with cyan color proportional to `silence_elapsed / silence_duration`
- **Volume Indicator**: Horizontal bar below button shows current amplitude:
  - Gray: below threshold
  - Green: threshold to 1.0
  - Red: above 1.0 (clipping)
  - Clipped at 1.2 for display

#### Background Transcription
Uses `std::sync::mpsc` channels to communicate between transcription thread and UI:
```rust
enum TranscriptionMessage {
    InProgress,
    Success(String),
    Error(String),
}
```

#### Global Hotkey Implementation
The application registers a global hotkey that works system-wide:
```rust
// Parse hotkey from config string (e.g., "Ctrl+Shift+H")
let current_hotkey = config.parse_hotkey().unwrap_or_else(|e| {
    HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH)
});

// Register with global hotkey manager
hotkey_manager.register(current_hotkey);

// In update loop, check for hotkey events
if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
    if event.id == self.current_hotkey.id() {
        // Start recording
    }
}

// Request periodic repaints to ensure hotkey events are detected
ctx.request_repaint_after(std::time::Duration::from_millis(100));
```

Hotkeys can be dynamically changed by unregistering the old hotkey and registering a new one.

#### Settings Dialog with Scrolling
The settings dialog uses a `ScrollArea` to handle overflow:
```rust
egui::ScrollArea::vertical()
    .max_height(400.0)
    .show(ui, |ui| {
        // Settings content
    });

// Save/Cancel buttons outside ScrollArea for always-visible controls
```

### Configuration Storage
Settings are stored in OS-specific locations:
- Windows: `%APPDATA%\winh\config.json`
- Linux: `~/.config/winh/config.json`
- macOS: `~/Library/Application Support/winh/config.json`

### Dependencies
- **GUI**: eframe 0.29, egui 0.29
- **Audio**: cpal 0.15, hound 3.5
- **HTTP**: reqwest 0.12 (blocking, multipart)
- **Serialization**: serde 1.0, serde_json 1.0
- **System**: dirs 5.0, arboard 3.3, tempfile 3.8, image 0.24
- **Hotkey**: global-hotkey 0.6
- **Build**: winres 0.1 (Windows icon)

### License
MIT License - see LICENSE file for details.
