use enigo::{Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

pub fn call_qvpen() -> Result<(), String> {
    thread::spawn(|| {
        if let Err(e) = call_qvpen_sync() {
            eprintln!("call_qvpen failed: {}", e);
        }
    });
    Ok(())
}

#[cfg(windows)]
fn call_qvpen_sync() -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow};

    let title: Vec<u16> = "VRChat\0".encode_utf16().collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if hwnd.is_null() {
        return Err("VRChat window not found".to_string());
    }
    unsafe { SetForegroundWindow(hwnd) };

    thread::sleep(Duration::from_millis(200));

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create Enigo: {:?}", e))?;

    enigo
        .key(Key::Tab, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press Tab: {:?}", e))?;
    enigo
        .key(Key::Unicode('q'), enigo::Direction::Click)
        .map_err(|e| format!("Failed to press Q: {:?}", e))?;
    enigo
        .key(Key::Tab, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release Tab: {:?}", e))?;

    Ok(())
}

#[cfg(not(windows))]
fn call_qvpen_sync() -> Result<(), String> {
    Err("call_qvpen is only supported on Windows".to_string())
}

/// Types text character-by-character into the currently focused window
/// This function is non-blocking and spawns a background thread
pub fn type_text(text: &str) -> Result<(), String> {
    let text_owned = text.to_string();

    thread::spawn(move || {
        if let Err(e) = type_text_sync(&text_owned) {
            eprintln!("Auto-input failed: {}", e);
        }
    });

    Ok(())
}

/// Sends Ctrl+V to paste from clipboard into the currently focused window
/// This function is non-blocking and spawns a background thread
pub fn send_ctrl_v() -> Result<(), String> {
    thread::spawn(move || {
        if let Err(e) = send_ctrl_v_sync() {
            eprintln!("Auto-input (Ctrl+V) failed: {}", e);
        }
    });

    Ok(())
}

/// Sends Ctrl+V followed by Enter key
/// This function is non-blocking and spawns a background thread
pub fn send_ctrl_v_with_enter() -> Result<(), String> {
    thread::spawn(move || {
        if let Err(e) = send_ctrl_v_with_enter_sync() {
            eprintln!("Auto-input (Ctrl+V + Enter) failed: {}", e);
        }
    });

    Ok(())
}

/// Types text character-by-character followed by Enter key
/// This function is non-blocking and spawns a background thread
pub fn type_text_with_enter(text: &str) -> Result<(), String> {
    let text_owned = text.to_string();

    thread::spawn(move || {
        if let Err(e) = type_text_with_enter_sync(&text_owned) {
            eprintln!("Auto-input (typing + Enter) failed: {}", e);
        }
    });

    Ok(())
}

/// Internal synchronous implementation of text typing
fn type_text_sync(text: &str) -> Result<(), String> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create Enigo: {:?}", e))?;

    // Small delay to ensure target window is focused
    thread::sleep(Duration::from_millis(100));

    // Type the text using the text() method which handles Unicode properly
    enigo
        .text(&text)
        .map_err(|e| format!("Failed to type text: {:?}", e))?;

    Ok(())
}

/// Internal synchronous implementation of Ctrl+V
fn send_ctrl_v_sync() -> Result<(), String> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create Enigo: {:?}", e))?;

    // Small delay to ensure target window is focused
    thread::sleep(Duration::from_millis(100));

    // Press Ctrl+V
    enigo
        .key(Key::Control, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press Ctrl: {:?}", e))?;
    enigo
        .key(Key::Unicode('v'), enigo::Direction::Click)
        .map_err(|e| format!("Failed to press V: {:?}", e))?;
    enigo
        .key(Key::Control, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release Ctrl: {:?}", e))?;

    Ok(())
}

/// Internal synchronous implementation of Ctrl+V followed by Enter
fn send_ctrl_v_with_enter_sync() -> Result<(), String> {
    // Execute Ctrl+V
    send_ctrl_v_sync()?;

    // Small delay between Ctrl+V and Enter
    thread::sleep(Duration::from_millis(100));

    // Press Enter
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create Enigo: {:?}", e))?;
    enigo
        .key(Key::Return, enigo::Direction::Click)
        .map_err(|e| format!("Failed to press Enter: {:?}", e))?;

    Ok(())
}

/// Internal synchronous implementation of text typing followed by Enter
fn type_text_with_enter_sync(text: &str) -> Result<(), String> {
    // Type the text
    type_text_sync(text)?;

    // Small delay between typing and Enter
    thread::sleep(Duration::from_millis(100));

    // Press Enter
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create Enigo: {:?}", e))?;
    enigo
        .key(Key::Return, enigo::Direction::Click)
        .map_err(|e| format!("Failed to press Enter: {:?}", e))?;

    Ok(())
}
