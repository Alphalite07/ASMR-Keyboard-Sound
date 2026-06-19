use rdev::{listen, Event, EventType};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::Arc;
use std::thread;

fn main() {
    // 1. Initialize the OS Audio System
    // We keep the _stream alive in the main thread so audio doesn't cut out
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    
    // 2. Pre-load the audio file into memory (RAM)
    // Place a short, punchy file named 'thock.wav' in your project root
    let audio_file_path = "thock.wav";
    let audio_bytes = std::fs::read(audio_file_path)
        .expect("Failed to read thock.wav. Make sure the file exists in the directory!");
    
    // Wrap the bytes in an Arc (Atomic Reference Counted pointer) 
    // so we can safely share it across multiple high-speed threads
    let audio_data = Arc::new(audio_bytes);

    println!("⌨️ Ghost-Switches is running! Listening for keystrokes...");

    // 3. Define the OS-level keylogger callback
    let callback = move |event: Event| {
        // We only care when a key goes DOWN, not when it comes UP (for now)
        if let EventType::KeyPress(_) = event.event_type {
            
            // Clone our references (this is extremely fast and doesn't copy the actual audio data)
            let data = Arc::clone(&audio_data);
            let handle = stream_handle.clone();
            
            // 4. Spawn a lightweight background thread
            // If we play audio on the main thread, it will block the OS input stream 
            // and make your real keyboard lag.
            thread::spawn(move || {
                // Read the audio from RAM
                let cursor = Cursor::new((*data).clone());
                
                if let Ok(decoder) = Decoder::new(cursor) {
                    let sink = Sink::try_new(&handle).unwrap();
                    sink.append(decoder);
                    
                    // Keep the thread alive just long enough for the "thock" to finish
                    sink.sleep_until_end(); 
                }
            });
        }
    };

    // 5. Start listening. This will block the main thread forever.
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error);
    }
}