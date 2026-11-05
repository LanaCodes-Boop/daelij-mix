mod audio;

#[tauri::command]
async fn list_sessions() -> Result<Vec<audio::Session>, String> {
    audio::list_sessions()
}

#[tauri::command]
async fn set_session_volume(pid: u32, volume: f32) -> Result<(), String> {
    audio::set_session_volume(pid, volume)
}

#[tauri::command]
async fn restore_all() -> Result<(), String> {
    audio::restore_all()
}

#[tauri::command]
async fn list_outputs() -> Result<Vec<audio::Output>, String> {
    audio::list_outputs()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            set_session_volume,
            restore_all,
            list_outputs
        ])
        .build(tauri::generate_context!())
        .expect("error while building Daelij MiX")
        .run(|_app, event| {
            // Give every app its full volume back when Daelij MiX closes.
            if let tauri::RunEvent::Exit = event {
                let _ = audio::restore_all();
            }
        });
}
