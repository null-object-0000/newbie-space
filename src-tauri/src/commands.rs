use serde::Serialize;
use std::net::TcpStream;
use std::time::Duration;
use sysinfo::{Pid, System};

// --- 数据结构 ---

#[derive(Serialize)]
pub struct PortCheckResult {
    pub port: u16,
    pub is_open: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct FileHandleInfo {
    pub pid: u32,
    pub process_name: String,
    pub file_path: String,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub status: String,
    pub exe_path: Option<String>,
}

#[derive(Serialize)]
pub struct KillProcessResult {
    pub pid: u32,
    pub success: bool,
    pub message: String,
}

// --- 命令实现 ---

#[tauri::command]
pub fn check_port(port: u16) -> PortCheckResult {
    match TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_secs(2),
    ) {
        Ok(_) => PortCheckResult {
            port,
            is_open: true,
            message: format!("端口 {} 开放（连接成功）", port),
        },
        Err(e) => PortCheckResult {
            port,
            is_open: false,
            message: format!("端口 {} 关闭或不可达：{}", port, e),
        },
    }
}

#[tauri::command]
pub fn check_file_handles(pid: u32) -> Vec<FileHandleInfo> {
    let mut results = Vec::new();
    let sys_pid = Pid::from(pid as usize);
    let mut system = System::new_all();
    system.refresh_all();

    let process_name = system
        .process(sys_pid)
        .map(|p| p.name().to_string_lossy().to_string())
        .unwrap_or_else(|| format!("PID {}", pid));

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::fs;
        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(entries) = fs::read_dir(&fd_dir) {
            for entry in entries.flatten() {
                if let Ok(target) = fs::read_link(entry.path()) {
                    results.push(FileHandleInfo {
                        pid,
                        process_name: process_name.clone(),
                        file_path: target.to_string_lossy().to_string(),
                    });
                }
            }
        }
        if results.is_empty() {
            results.push(FileHandleInfo {
                pid,
                process_name,
                file_path: "该进程未打开任何文件句柄，或无法读取 /proc 目录".to_string(),
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        results.push(FileHandleInfo {
            pid,
            process_name,
            file_path: "[Windows 文件句柄枚举需要额外 API 调用，暂未支持]".to_string(),
        });
    }

    results
}

#[tauri::command]
pub fn list_processes(search: Option<String>) -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_all();

    let mut processes: Vec<ProcessInfo> = system
        .processes()
        .iter()
        .filter(|(_, p)| match &search {
            Some(query) => p
                .name()
                .to_string_lossy()
                .to_lowercase()
                .contains(&query.to_lowercase()),
            None => true,
        })
        .map(|(pid, p)| ProcessInfo {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu_usage: p.cpu_usage(),
            memory_mb: p.memory() as f64 / 1_048_576.0,
            status: format!("{:?}", p.status()),
            exe_path: p.exe().map(|p| p.to_string_lossy().to_string()),
        })
        .collect();

    processes.sort_by_key(|p| p.pid);
    processes
}

#[tauri::command]
pub fn kill_process(pid: u32) -> KillProcessResult {
    let sys_pid = Pid::from(pid as usize);
    let mut system = System::new_all();
    system.refresh_all();

    match system.process(sys_pid) {
        Some(process) => {
            if process.kill() {
                KillProcessResult {
                    pid,
                    success: true,
                    message: format!("进程 {} 已终止", pid),
                }
            } else {
                KillProcessResult {
                    pid,
                    success: false,
                    message: format!("无法终止进程 {}（权限不足？）", pid),
                }
            }
        }
        None => KillProcessResult {
            pid,
            success: false,
            message: format!("未找到进程 {}", pid),
        },
    }
}
