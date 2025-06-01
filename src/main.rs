use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use enigo::{Enigo, Key, KeyboardControllable};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use log::{info, error, debug};

const BUFFER_SIZE: usize = 2048;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Starting up pitch-to-key program...");

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| {
            error!("No input device available. Please ensure a microphone is connected and recognized by your system.");
            Box::<dyn std::error::Error>::from("No input device available")
        })?;
    info!("Found default input device: {}", device.name()?);

    let config = device.default_input_config()?;
    info!("Using default input stream config: {:?}", config);

    let sample_rate = config.sample_rate().0 as usize;
    let mut detector = McLeodDetector::new(BUFFER_SIZE, BUFFER_SIZE / 2);

    // --- State variables for continuous key presses ---
    let mut current_active_key: Option<Key> = None;
    let mut current_key_start_time: Option<Instant> = None;
    let mut last_continuous_send_time: Option<Instant> = None;

    // Constants for timing
    const HOLD_THRESHOLD_MILLIS: u64 = 250; // How long to hold a note before continuous presses start
    const REPEAT_INTERVAL_MILLIS: u64 = 100; // How often to send a key press once continuous is active

    let shared_audio_queue = Arc::new(Mutex::new(VecDeque::new()));
    let audio_queue_clone = Arc::clone(&shared_audio_queue);

    info!("Building audio input stream...");
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut audio_queue = audio_queue_clone.lock().unwrap();
            audio_queue.extend(data.iter().cloned());
        },
        move |err| error!("Stream error: {:?}", err),
        None,
    )?;

    stream.play()?;
    info!("Successfully started audio stream!");
    info!("Listening for pitch... (sing into your mic)");
    info!("Ensure mGBA (or the target application) is the active window.");
    info!("---");

    let mut enigo = Enigo::new();

    loop {
        let mut audio_queue = shared_audio_queue.lock().unwrap();

        // Process audio in chunks of BUFFER_SIZE
        while audio_queue.len() >= BUFFER_SIZE {
            let audio_window: Vec<f32> = audio_queue.drain(0..BUFFER_SIZE).collect();

            let mut new_key_to_press: Option<Key> = None;

            if let Some(pitch) = detector.get_pitch(&audio_window, sample_rate, 0.7, 0.2) {
                info!(
                    "Input: Detected pitch = {:.2} Hz (Clarity: {:.2})",
                    pitch.frequency, pitch.clarity
                );
                new_key_to_press = map_frequency_to_key(pitch.frequency);
            } else {
                // If no clear pitch is detected, you can log it (debug level)
                debug!("Input: No clear pitch detected in this audio segment.");
            }

            // --- Logic for handling key presses (single or continuous) ---
            match (new_key_to_press, current_active_key) {
                // Case 1: Same note/key is still being held
                (Some(new_key), Some(active_key)) if new_key == active_key => {
                    if let Some(start_time) = current_key_start_time {
                        // Check if the hold threshold has been met
                        if start_time.elapsed() >= Duration::from_millis(HOLD_THRESHOLD_MILLIS) {
                            // If it has, check if enough time has passed since the last continuous send
                            if let Some(last_send_time) = last_continuous_send_time {
                                if last_send_time.elapsed() >= Duration::from_millis(REPEAT_INTERVAL_MILLIS) {
                                    info!("Action: Repeating key '{:?}' (held).", active_key);
                                    enigo.key_click(active_key);
                                    last_continuous_send_time = Some(Instant::now());
                                }
                            } else {
                                // This path should be hit if last_continuous_send_time got reset somehow,
                                // but we're past the hold threshold. Send an initial repeat.
                                info!("Action: Repeating key '{:?}' (first repeat after hold threshold).", active_key);
                                enigo.key_click(active_key);
                                last_continuous_send_time = Some(Instant::now());
                            }
                        } else {
                            debug!(
                                "Info: Key '{:?}' held, but still within hold threshold ({}ms remaining).",
                                active_key,
                                (Duration::from_millis(HOLD_THRESHOLD_MILLIS) - start_time.elapsed()).as_millis()
                            );
                        }
                    }
                },
                // Case 2: A new key is detected (either different from active, or no active key was present)
                (Some(new_key), _) => {
                    info!("Action: New key '{:?}' detected. Sending initial press!", new_key);
                    enigo.key_click(new_key);
                    current_active_key = Some(new_key);
                    current_key_start_time = Some(Instant::now());
                    last_continuous_send_time = Some(Instant::now()); // Record time of this first press
                },
                // Case 3: No valid pitch detected, but a key was previously active (note released/lost)
                (None, Some(active_key)) => {
                    info!("Info: Pitch lost. Releasing key '{:?}' state.", active_key);
                    current_active_key = None;
                    current_key_start_time = None;
                    last_continuous_send_time = None;
                },
                // Case 4: No valid pitch detected, and no key was active. Do nothing.
                (None, None) => {
                    // This can be very verbose, only uncomment for specific debugging:
                    // debug!("Input: No clear pitch and no active key.");
                }
            }
        }
        // IMPORTANT: Explicitly drop the mutex lock before sleeping.
        drop(audio_queue);

        thread::sleep(Duration::from_millis(50)); // Main loop polling rate
    }
}

// Your existing map_frequency_to_key function
fn map_frequency_to_key(freq: f32) -> Option<Key> {
    match freq {
        100.0..=115.0 => Some(Key::DownArrow),
        115.1..=130.0 => Some(Key::LeftArrow),
        130.1..=145.0 => Some(Key::RightArrow),
        145.1..=160.0 => Some(Key::UpArrow),
        160.1..=175.0 => Some(Key::Backspace),
        175.1..=200.0 => Some(Key::Layout('x')),
        200.1..=230.0 => Some(Key::Layout('z')),
        230.1..=270.0 => Some(Key::Layout('a')),
        270.1..=305.0 => Some(Key::Layout('s')),
        305.1..=338.0 => Some(Key::Return),
        _ => None,
    }
}
