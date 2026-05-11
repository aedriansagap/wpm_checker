use evdev::{Device, Key};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::time::sleep;

#[derive(Clone, serde::Serialize)]
struct WpmPayload {
    current_wpm: u32,
    raw_wpm: u32,
}

struct WpmState {
    keystrokes: VecDeque<Instant>,
}

impl WpmState {
    fn new() -> Self {
        Self {
            keystrokes: VecDeque::new(),
        }
    }

    fn add_keystroke(&mut self) {
        let now = Instant::now();
        self.keystrokes.push_back(now);
        self.cleanup(now);
    }

    fn cleanup(&mut self, now: Instant) {
        let window = Duration::from_secs(15);
        while let Some(&time) = self.keystrokes.front() {
            if now.duration_since(time) > window {
                self.keystrokes.pop_front();
            } else {
                break;
            }
        }
    }

    fn calculate_wpm(&mut self) -> (u32, u32) {
        let now = Instant::now();
        self.cleanup(now);
        
        let keys_in_window = self.keystrokes.len() as u32;
        // WPM = (keys / 5) / time_in_minutes
        // Window is 15 seconds, so time_in_minutes = 0.25
        // WPM = (keys / 5) * 4 = keys * 0.8
        let current_wpm = (keys_in_window as f32 * 0.8) as u32;
        
        let raw_wpm = current_wpm; 
        
        (current_wpm, raw_wpm)
    }
}

fn find_keyboard() -> Option<Device> {
    for (_, device) in evdev::enumerate() {
        if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_A) && keys.contains(Key::KEY_ENTER)) {
            return Some(device);
        }
    }
    None
}

fn start_keylogger(state: Arc<Mutex<WpmState>>) {
    std::thread::spawn(move || {
        let mut device = match find_keyboard() {
            Some(d) => d,
            None => {
                println!("No keyboard found! You might need to run with sudo.");
                return;
            }
        };

        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for ev in events {
                        if ev.event_type() == evdev::EventType::KEY && ev.value() == 1 {
                            // Key press
                            if let Ok(mut state_lock) = state.lock() {
                                state_lock.add_keystroke();
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading events: {:?}", e);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let wpm_state = Arc::new(Mutex::new(WpmState::new()));
    
    let state_clone = wpm_state.clone();
    start_keylogger(state_clone);

    tauri::Builder::default()
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state = wpm_state.clone();
            
            tauri::async_runtime::spawn(async move {
                loop {
                    sleep(Duration::from_millis(500)).await;
                    let (current_wpm, raw_wpm) = {
                        let mut state_lock = state.lock().unwrap();
                        state_lock.calculate_wpm()
                    };
                    
                    app_handle.emit("wpm-update", WpmPayload { current_wpm, raw_wpm }).unwrap();
                }
            });
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
