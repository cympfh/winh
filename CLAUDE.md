# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**winh** is a Windows 11 voice transcription GUI application written in Rust. It provides a single-window interface with a start/stop button that captures audio input, automatically detects silence, and transcribes speech to text using OpenAI's API. The transcribed text is displayed in the GUI and copied to the clipboard.

### Key Features
- Single executable file (no installer required)
- Cross-platform compilation (can be built on WSL, runs on Windows 11)
- Voice input using Windows default audio input
- Automatic silence detection (default 2 seconds)
- Audio saved temporarily as MP3 and sent to OpenAI for transcription
- Settings modal for configuration (silence detection duration, API key, model selection)
- Local settings persistence

## Development Commands

### Building
```bash
cargo build
```

### Running
```bash
cargo run
```

### Building Release Binary
```bash
cargo build --release
```
The output executable will be in `target/release/winh.exe`

### Checking Code
```bash
cargo check
```

### Running Tests
```bash
cargo test
```

## Project Status

This is a new project with minimal implementation. The main.rs currently contains only a "Hello, world!" placeholder. Refer to TODO.md for the full specification and implementation roadmap.

## Architecture Notes

### GUI Framework
The project will need a Rust GUI framework suitable for Windows 11 that can:
- Create a single-window application
- Handle button state changes (start/stop toggle)
- Display transcribed text
- Show modal dialogs for settings

### Core Components (To Be Implemented)
1. **Audio Capture**: Interface with Windows audio input APIs
2. **Silence Detection**: Monitor audio stream for silence periods
3. **Audio Processing**: Save captured audio as MP3 format
4. **API Integration**: HTTP client for OpenAI transcription API
5. **Clipboard Management**: Copy transcribed text to system clipboard
6. **Settings Management**: Local storage for user configuration
7. **GUI State Management**: Handle button states, text display, and modal interactions

### Configuration Storage
User settings (API key, model, silence detection duration) need to be persisted locally. Consider using a standard location for Windows application data.
