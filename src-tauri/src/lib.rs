mod commands;
mod llm_gateway;

use commands::{check_file_usage, check_port, find_port_process, kill_process, list_processes};
use llm_gateway::{
    clear_llm_gateway_logs, delete_llm_gateway_profile, discover_llm_gateway_models,
    get_llm_gateway_config, get_llm_gateway_status, get_llm_gateway_usage, list_llm_gateway_logs,
    list_llm_gateway_profiles, load_llm_gateway_profile, save_llm_gateway_config,
    save_llm_gateway_profile, start_llm_gateway, stop_llm_gateway, test_llm_gateway_config,
    LlmGatewayState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LlmGatewayState::default())
        .invoke_handler(tauri::generate_handler![
            check_port,
            find_port_process,
            check_file_usage,
            list_processes,
            kill_process,
            get_llm_gateway_config,
            save_llm_gateway_config,
            start_llm_gateway,
            stop_llm_gateway,
            get_llm_gateway_status,
            list_llm_gateway_logs,
            get_llm_gateway_usage,
            clear_llm_gateway_logs,
            test_llm_gateway_config,
            discover_llm_gateway_models,
            list_llm_gateway_profiles,
            save_llm_gateway_profile,
            load_llm_gateway_profile,
            delete_llm_gateway_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
