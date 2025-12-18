use enigo::{Enigo, Keyboard, Settings};
use std::thread;
use std::time::Duration;

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
