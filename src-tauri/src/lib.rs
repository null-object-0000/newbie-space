mod commands;

use commands::{check_file_usage, check_port, find_port_process, kill_process, list_processes};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_port,
            find_port_process,
            check_file_usage,
            list_processes,
            kill_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
