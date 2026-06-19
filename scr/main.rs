use rdev::{listen, Event, EventType};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::Arc;
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait};
use rand::seq::SliceRandom; // Required for picking random sounds

// --- 1. HEADPHONE DETECTOR ---
fn is_headphone_active() -> bool {
    let host = cpal::default_host();
    
    // Grab whatever device the OS is currently routing audio to
    if let Some(device) = host.default_output_device() {
        if let Ok(name) = device.name() {
            let name_lower = name.to_lowercase();
            
            // Add any specific names your personal headphones use here!
            return name_lower.contains("headphone") || 
                   name_lower.contains("airpods") || 
                   name_lower.contains("bluetooth") || 
                   name_lower.contains("earbud") ||
                   name_lower.contains("headset");
        }
    }
    false // Default to false so we don't accidentally play on speakers
}

// --- 2. MAIN ENGINE ---
fn main() {
    // Initialize the OS Audio System
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    
    // Create a pool to hold all our audio bytes in RAM
    let mut audio_pool: Vec<Arc<Vec<u8>>> = Vec::new();
    
    println!("Loading sound files...");
    
    // Loop through the sounds directory and load every file
    // (Adjust the '20' to however many clicks your Python script generated)
    for i in 0..20 {
        let file_path = format!("sounds/click_{}.wav", i);
        if let Ok(audio_bytes) = std::fs::read(&file_path) {
            audio_pool.push(Arc::new(audio_bytes));
        }
    }
    
    if audio_pool.is_empty() {
        panic!("No audio files found! Make sure you ran the Python extractor and have a 'sounds' folder next to your executable.");
    }

    println!("⌨️ Loaded {} clicks. Ghost-Switches is running!", audio_pool.len());

    // Wrap the pool in an Arc so it can be safely shared across threads
    let shared_pool = Arc::new(audio_pool);

    // Define the OS-level keylogger callback
    let callback = move |event: Event| {
        if let EventType::KeyPress(_) = event.event_type {
            
            // ONLY proceed if headphones are actively connected
            if is_headphone_active() {
                let pool = Arc::clone(&shared_pool);
                let handle = stream_handle.clone();
                
                thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    
                    // Pick a random sound from the pool
                    if let Some(random_audio) = pool.choose(&mut rng) {
                        let cursor = Cursor::new((**random_audio).clone());
                        
                        if let Ok(decoder) = Decoder::new(cursor) {
                            let sink = Sink::try_new(&handle).unwrap();
                            sink.append(decoder);
                            sink.sleep_until_end(); 
                        }
                    }
                });
            }
        }
    };

    // Start listening. This will block the main thread forever.
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error);
    }
}