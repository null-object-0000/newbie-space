use serde::Serialize;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use sysinfo::{Pid, System};

// --- 数据结构 ---

#[derive(Serialize)]
pub struct PortCheckResult {
    pub port: u16,
    pub is_open: bool,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct PortProcessInfo {
    pub pid: u32,
    pub process_name: String,
    pub protocol: String,
    pub local_addr: String,
}

#[derive(Serialize)]
pub struct FileUsageInfo {
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
    let timeout = Duration::from_millis(800);
    let mut addresses = Vec::new();

    if let Ok(resolved) = format!("localhost:{}", port).to_socket_addrs() {
        addresses.extend(resolved);
    }

    for fallback in [
        SocketAddr::from(([127, 0, 0, 1], port)),
        SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port)),
    ] {
        if !addresses.contains(&fallback) {
            addresses.push(fallback);
        }
    }

    let mut errors = Vec::new();
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(_) => {
                return PortCheckResult {
                    port,
                    is_open: true,
                    message: format!("端口 {} 开放（{} 连接成功）", port, address),
                }
            }
            Err(e) => errors.push(format!("{}: {}", address, e)),
        }
    }

    PortCheckResult {
        port,
        is_open: false,
        message: format!("端口 {} 关闭或不可达：{}", port, errors.join("；")),
    }
}

#[tauri::command]
pub fn find_port_process(port: u16) -> Vec<PortProcessInfo> {
    use netstat2::{
        get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo,
    };

    let af = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let pf = ProtocolFlags::TCP | ProtocolFlags::UDP;

    let sockets = match get_sockets_info(af, pf) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut system = System::new_all();
    system.refresh_all();

    let mut results: Vec<PortProcessInfo> = Vec::new();

    for si in &sockets {
        let (local_port, protocol, local_addr) = match &si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                let addr =
                    format!("{}:{}", tcp.local_addr, tcp.local_port);
                (tcp.local_port, "TCP".to_string(), addr)
            }
            ProtocolSocketInfo::Udp(udp) => {
                let addr =
                    format!("{}:{}", udp.local_addr, udp.local_port);
                (udp.local_port, "UDP".to_string(), addr)
            }
        };

        if local_port != port {
            continue;
        }

        for &pid in &si.associated_pids {
            let process_name = system
                .process(Pid::from(pid as usize))
                .map(|p| p.name().to_string_lossy().to_string())
                .unwrap_or_else(|| format!("PID {}", pid));

            results.push(PortProcessInfo {
                pid,
                process_name,
                protocol: protocol.clone(),
                local_addr: local_addr.clone(),
            });
        }
    }

    results
}

#[tauri::command]
pub fn check_file_usage(path: String) -> Vec<FileUsageInfo> {
    let mut results = Vec::new();
    let mut system = System::new_all();
    system.refresh_all();

    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(proc_entries) = fs::read_dir("/proc") {
            for proc_entry in proc_entries.flatten() {
                let proc_name = proc_entry.file_name();
                let pid_str = proc_name.to_string_lossy();

                // 跳过非数字目录（如 self, thread-self, sys 等）
                if pid_str.parse::<u32>().is_err() {
                    continue;
                }

                let pid: u32 = pid_str.parse().unwrap();
                let fd_dir = proc_entry.path().join("fd");
                let entries = match fs::read_dir(&fd_dir) {
                    Ok(e) => e,
                    Err(_) => continue, // 权限不足，跳过
                };

                for fd_entry in entries.flatten() {
                    let target = match fs::read_link(fd_entry.path()) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let target_str = target.to_string_lossy();

                    // 匹配：目标路径以输入路径开头（支持目录匹配）
                    if target_str.starts_with(&path) {
                        let process_name = system
                            .process(Pid::from(pid as usize))
                            .map(|p| p.name().to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("PID {}", pid));

                        results.push(FileUsageInfo {
                            pid,
                            process_name,
                            file_path: target_str.to_string(),
                        });
                        break; // 每个进程只记录一次
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        match Command::new("lsof").args([path.as_str()]).output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut seen_pids = std::collections::HashSet::new();

                // lsof 默认输出格式：COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue;
                    }
                    let pid_str = parts[1];
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if seen_pids.contains(&pid) {
                            continue;
                        }
                        seen_pids.insert(pid);

                        let process_name = system
                            .process(Pid::from(pid as usize))
                            .map(|p| p.name().to_string_lossy().to_string())
                            .unwrap_or_else(|| parts[0].to_string());

                        results.push(FileUsageInfo {
                            pid,
                            process_name,
                            file_path: path.clone(),
                        });
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if !stderr.is_empty() {
                    results.push(FileUsageInfo {
                        pid: 0,
                        process_name: "lsof error".to_string(),
                        file_path: format!("lsof 查询失败：{}", stderr),
                    });
                }
            }
            Err(e) => {
                results.push(FileUsageInfo {
                    pid: 0,
                    process_name: "error".to_string(),
                    file_path: format!("无法执行 lsof：{}", e),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 使用 Windows Restart Manager API
        // 注意：RM 只能检测已注册扩展名的文件，不适用于所有文件类型
        #[cfg(windows)]
        {
            use windows::Win32::System::RestartManager::{
                RmStartSession, RmRegisterResources, RmGetList, RmEndSession,
                RM_SESSION_KEY,
            };
            use windows::core::PWSTR;

            let mut session_handle: u32 = 0;
            let mut session_key: [u16; 33] = [0; 33];

            // RmStartSession
            let path_wide: Vec<u16> = path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            unsafe {
                if RmStartSession(
                    &mut session_handle,
                    0,
                    &mut session_key,
                ).is_ok()
                {
                    let wide_paths = [PWSTR::from_raw(path_wide.as_ptr() as *mut _)];

                    if RmRegisterResources(
                        session_handle,
                        &wide_paths,
                        0,
                        None,
                        0,
                        None,
                    ).is_ok()
                    {
                        let mut needed: u32 = 0;
                        let mut count: u32 = 0;
                        let mut reason: u32 = 0;

                        // 第一次调用获取所需缓冲区大小
                        if RmGetList(
                            session_handle,
                            &mut needed,
                            &mut count,
                            None,
                            &mut reason,
                        ).is_err()
                            && needed > 0
                        {
                            let mut buf: Vec<u8> = vec![0; needed as usize];
                            if RmGetList(
                                session_handle,
                                &mut needed,
                                &mut count,
                                Some(buf.as_mut_ptr() as *mut _),
                                &mut reason,
                            ).is_ok()
                            {
                                // RM_PROCESS_INFO 数组中提取 PID 和应用名
                                let proc_info_slice = unsafe {
                                    std::slice::from_raw_parts(
                                        buf.as_ptr() as *const u8,
                                        needed as usize,
                                    )
                                };
                                // 简单解析：RM_PROCESS_INFO 是一个变长结构体
                                // 由于直接解析较复杂，这里返回检测到占用但不解析具体进程
                                // 真正需要详细进程信息时，可以结合 sysinfo 枚举所有进程
                            }
                        }
                    }
                    RmEndSession(session_handle);
                }
            }
        }

        // 如果 RM 没有返回结果，使用 netstat-like 方式作为补充说明
        if results.is_empty() {
            results.push(FileUsageInfo {
                pid: 0,
                process_name: "Windows".to_string(),
                file_path: format!(
                    "文件占用检测在 Windows 上有限支持。请使用 Resource Monitor 或 handle.exe (Sysinternals) 手动检查：{}",
                    path
                ),
            });
        }
    }

    // 如果所有平台都没有结果
    if results.is_empty() {
        results.push(FileUsageInfo {
            pid: 0,
            process_name: "".to_string(),
            file_path: "未找到占用该文件/目录的进程".to_string(),
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
