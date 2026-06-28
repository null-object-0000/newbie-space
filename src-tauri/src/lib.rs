mod commands;

use commands::{check_port, check_file_handles, kill_process, list_processes};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_port,
            check_file_handles,
            list_processes,
            kill_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
