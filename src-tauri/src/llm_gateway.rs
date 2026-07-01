use crate::desktop_settings::{
    load_desktop_settings, pull_json_from_s3, push_json_to_s3, S3SyncResult,
};
use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use reqwest::Client;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    fs::OpenOptions,
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{async_runtime::JoinHandle, Manager};
use tokio::{net::TcpListener, sync::oneshot};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStrategy {
    RoundRobin,
    Random,
}

impl Default for KeyStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

fn default_true() -> bool {
    true
}
fn default_weight() -> u32 {
    1
}
fn default_priority() -> u32 {
    1
}
fn default_timeout_ms() -> u64 {
    60_000
}
fn default_retry_count() -> u32 {
    0
}
fn default_max_concurrent_requests() -> u32 {
    8
}
fn default_requests_per_minute() -> u32 {
    120
}
fn default_false() -> bool {
    false
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub id: String,
    pub name: String,
    pub api_key: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default)]
    pub key_strategy: KeyStrategy,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    #[serde(default)]
    pub keys: Vec<ApiKeyConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelRouteConfig {
    pub id: String,
    pub public_model: String,
    pub upstream_model: String,
    pub channel_id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default)]
    pub prompt_cost_per_1k: f64,
    #[serde(default)]
    pub completion_cost_per_1k: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub listen_host: String,
    pub listen_port: u16,
    #[serde(default = "default_false")]
    pub inbound_auth_enabled: bool,
    #[serde(default)]
    pub inbound_api_key: String,
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
    #[serde(default)]
    pub model_routes: Vec<ModelRouteConfig>,
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: u32,
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_host: "127.0.0.1".to_string(),
            listen_port: 11434,
            inbound_auth_enabled: false,
            inbound_api_key: String::new(),
            channels: Vec::new(),
            model_routes: Vec::new(),
            max_concurrent_requests: default_max_concurrent_requests(),
            requests_per_minute: default_requests_per_minute(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GatewayStatus {
    pub running: bool,
    pub listen_url: String,
    pub channel_count: usize,
    pub enabled_channel_count: usize,
    pub route_count: usize,
    pub request_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub public_model: String,
    pub upstream_model: Option<String>,
    pub channel_id: Option<String>,
    pub key_id: Option<String>,
    pub status: u16,
    pub latency_ms: u128,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
    pub error: Option<String>,
    pub input: Value,
    pub output: Option<Value>,
    #[serde(default)]
    pub attempts: Vec<RequestAttemptLog>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestAttemptLog {
    pub attempt: u32,
    pub channel_id: String,
    pub channel_name: String,
    pub key_id: Option<String>,
    pub upstream_model: String,
    pub endpoint: String,
    pub status: Option<u16>,
    pub latency_ms: u128,
    pub upstream_request_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UsageSummary {
    pub requests: usize,
    pub success: usize,
    pub failed: usize,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
    pub by_model: Vec<UsageBreakdown>,
    pub by_channel: Vec<UsageBreakdown>,
    pub by_key: Vec<UsageBreakdown>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UsageBreakdown {
    pub id: String,
    pub label: String,
    pub requests: usize,
    pub success: usize,
    pub failed: usize,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChannelTestResult {
    pub channel_id: String,
    pub channel_name: String,
    pub ok: bool,
    pub status: Option<u16>,
    pub latency_ms: u128,
    pub message: String,
    pub models: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ConfigProfile {
    pub name: String,
    pub active: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModelDiscoveryResult {
    pub channel_id: String,
    pub channel_name: String,
    pub ok: bool,
    pub status: Option<u16>,
    pub latency_ms: u128,
    pub message: String,
    pub models: Vec<String>,
}

#[derive(Default)]
struct ServerHandle {
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

#[derive(Clone)]
struct GatewayHttpState {
    config: Arc<RwLock<GatewayConfig>>,
    logs: Arc<Mutex<Vec<RequestLog>>>,
    log_path: Arc<RwLock<Option<PathBuf>>>,
    key_cursors: Arc<Mutex<HashMap<String, usize>>>,
    limiter: Arc<Mutex<RequestLimiter>>,
    client: Client,
}

#[derive(Default)]
struct RequestLimiter {
    active_requests: u32,
    recent_requests: Vec<u128>,
}

struct RequestPermit {
    limiter: Arc<Mutex<RequestLimiter>>,
}

impl Drop for RequestPermit {
    fn drop(&mut self) {
        if let Ok(mut limiter) = self.limiter.lock() {
            limiter.active_requests = limiter.active_requests.saturating_sub(1);
        }
    }
}

pub struct LlmGatewayState {
    config: Arc<RwLock<GatewayConfig>>,
    logs: Arc<Mutex<Vec<RequestLog>>>,
    log_path: Arc<RwLock<Option<PathBuf>>>,
    key_cursors: Arc<Mutex<HashMap<String, usize>>>,
    limiter: Arc<Mutex<RequestLimiter>>,
    server: Mutex<ServerHandle>,
}

impl Default for LlmGatewayState {
    fn default() -> Self {
        Self {
            config: Arc::new(RwLock::new(GatewayConfig::default())),
            logs: Arc::new(Mutex::new(Vec::new())),
            log_path: Arc::new(RwLock::new(None)),
            key_cursors: Arc::new(Mutex::new(HashMap::new())),
            limiter: Arc::new(Mutex::new(RequestLimiter::default())),
            server: Mutex::new(ServerHandle::default()),
        }
    }
}

impl LlmGatewayState {
    fn http_state(&self) -> GatewayHttpState {
        GatewayHttpState {
            config: Arc::clone(&self.config),
            logs: Arc::clone(&self.logs),
            log_path: Arc::clone(&self.log_path),
            key_cursors: Arc::clone(&self.key_cursors),
            limiter: Arc::clone(&self.limiter),
            client: Client::new(),
        }
    }

    fn is_running(&self) -> bool {
        self.server
            .lock()
            .map(|server| server.shutdown.is_some())
            .unwrap_or(false)
    }

    fn set_log_path(&self, path: PathBuf) -> Result<(), String> {
        *self.log_path.write().map_err(|e| e.to_string())? = Some(path);
        Ok(())
    }
}

#[tauri::command]
pub async fn get_llm_gateway_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<GatewayConfig, String> {
    let settings = load_desktop_settings(&app).unwrap_or_default();
    let config = if settings.s3_sync.enabled && settings.s3_sync.auto_pull_on_start {
        match pull_gateway_config_from_s3(&settings.s3_sync).await {
            Ok(config) => {
                save_config_to_disk(&app, &config)?;
                config
            }
            Err(_) => load_config_from_disk(&app)?,
        }
    } else {
        load_config_from_disk(&app)?
    };
    *state.config.write().map_err(|e| e.to_string())? = config.clone();
    Ok(config)
}

#[tauri::command]
pub async fn save_llm_gateway_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
    config: GatewayConfig,
) -> Result<(), String> {
    if state.is_running() {
        return Err("请先停止本地 LLM API 服务，再修改配置".to_string());
    }
    validate_config(&config)?;
    save_config_to_disk(&app, &config)?;
    *state.config.write().map_err(|e| e.to_string())? = config.clone();
    state.key_cursors.lock().map_err(|e| e.to_string())?.clear();

    let settings = load_desktop_settings(&app).unwrap_or_default();
    if settings.s3_sync.enabled && settings.s3_sync.auto_push_on_save {
        push_gateway_config_to_s3(&settings.s3_sync, &config).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pull_llm_gateway_config_from_s3(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<GatewayConfig, String> {
    if state.is_running() {
        return Err("请先停止本地 LLM API 服务，再从 S3 拉取配置".to_string());
    }
    let settings = load_desktop_settings(&app)?;
    let config = pull_gateway_config_from_s3(&settings.s3_sync).await?;
    save_config_to_disk(&app, &config)?;
    *state.config.write().map_err(|e| e.to_string())? = config.clone();
    state.key_cursors.lock().map_err(|e| e.to_string())?.clear();
    Ok(config)
}

#[tauri::command]
pub async fn push_llm_gateway_config_to_s3(
    app: tauri::AppHandle,
    config: GatewayConfig,
) -> Result<S3SyncResult, String> {
    validate_config(&config)?;
    let settings = load_desktop_settings(&app)?;
    push_gateway_config_to_s3(&settings.s3_sync, &config).await
}
#[tauri::command]
pub async fn start_llm_gateway(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<GatewayStatus, String> {
    if state.is_running() {
        return Ok(build_status(&state)?);
    }

    let config = state.config.read().map_err(|e| e.to_string())?.clone();
    validate_config(&config)?;
    let addr: SocketAddr = format!("{}:{}", config.listen_host, config.listen_port)
        .parse()
        .map_err(|e| format!("监听地址无效：{}", e))?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("启动监听失败：{}", e))?;
    state.set_log_path(request_logs_path(&app)?)?;

    let (tx, rx) = oneshot::channel::<()>();
    let router = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/responses", post(responses))
        .route("/v1/messages", post(anthropic_messages))
        .route("/v1/messages/count_tokens", post(anthropic_count_tokens))
        .with_state(state.http_state());

    let task = tauri::async_runtime::spawn(async move {
        let server = axum::serve(listener, router).with_graceful_shutdown(async {
            let _ = rx.await;
        });
        if let Err(err) = server.await {
            eprintln!("LLM gateway server stopped with error: {}", err);
        }
    });

    let mut server = state.server.lock().map_err(|e| e.to_string())?;
    server.shutdown = Some(tx);
    server.task = Some(task);
    drop(server);
    build_status(&state)
}

#[tauri::command]
pub async fn stop_llm_gateway(
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<GatewayStatus, String> {
    let task = {
        let mut server = state.server.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = server.shutdown.take() {
            let _ = tx.send(());
        }
        server.task.take()
    };
    if let Some(task) = task {
        let _ = task.await;
    }
    build_status(&state)
}

#[tauri::command]
pub fn get_llm_gateway_status(
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<GatewayStatus, String> {
    build_status(&state)
}

#[tauri::command]
pub fn list_llm_gateway_logs(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
    limit: Option<usize>,
) -> Result<Vec<RequestLog>, String> {
    let take = limit.unwrap_or(100).min(500);
    let persisted = read_request_logs_from_disk(&request_logs_path(&app)?, Some(take))?;
    if !persisted.is_empty() {
        return Ok(persisted);
    }
    let logs = state.logs.lock().map_err(|e| e.to_string())?;
    Ok(logs.iter().rev().take(take).cloned().collect())
}

#[tauri::command]
pub fn get_llm_gateway_usage(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<UsageSummary, String> {
    let persisted = read_request_logs_from_disk(&request_logs_path(&app)?, None)?;
    let memory_logs;
    let logs: Vec<RequestLog> = if persisted.is_empty() {
        memory_logs = state.logs.lock().map_err(|e| e.to_string())?.clone();
        memory_logs
    } else {
        persisted
    };
    let mut summary = UsageSummary {
        requests: logs.len(),
        success: 0,
        failed: 0,
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
        estimated_cost: 0.0,
        by_model: Vec::new(),
        by_channel: Vec::new(),
        by_key: Vec::new(),
    };
    let mut by_model = HashMap::<String, UsageBreakdown>::new();
    let mut by_channel = HashMap::<String, UsageBreakdown>::new();
    let mut by_key = HashMap::<String, UsageBreakdown>::new();
    for log in logs.iter() {
        let success = (200..300).contains(&log.status);
        if success {
            summary.success += 1;
        } else {
            summary.failed += 1;
        }
        summary.prompt_tokens += log.prompt_tokens;
        summary.completion_tokens += log.completion_tokens;
        summary.total_tokens += log.total_tokens;
        summary.estimated_cost += log.estimated_cost;
        add_usage_breakdown(
            &mut by_model,
            &log.public_model,
            &log.public_model,
            log,
            success,
        );
        let channel_id = log.channel_id.as_deref().unwrap_or("unrouted");
        add_usage_breakdown(&mut by_channel, channel_id, channel_id, log, success);
        let key_id = log.key_id.as_deref().unwrap_or("none");
        add_usage_breakdown(&mut by_key, key_id, key_id, log, success);
    }
    summary.by_model = sorted_usage_breakdowns(by_model);
    summary.by_channel = sorted_usage_breakdowns(by_channel);
    summary.by_key = sorted_usage_breakdowns(by_key);
    Ok(summary)
}

fn add_usage_breakdown(
    groups: &mut HashMap<String, UsageBreakdown>,
    id: &str,
    label: &str,
    log: &RequestLog,
    success: bool,
) {
    let item = groups
        .entry(id.to_string())
        .or_insert_with(|| UsageBreakdown {
            id: id.to_string(),
            label: label.to_string(),
            ..UsageBreakdown::default()
        });
    item.requests += 1;
    if success {
        item.success += 1;
    } else {
        item.failed += 1;
    }
    item.prompt_tokens += log.prompt_tokens;
    item.completion_tokens += log.completion_tokens;
    item.total_tokens += log.total_tokens;
    item.estimated_cost += log.estimated_cost;
}

fn sorted_usage_breakdowns(groups: HashMap<String, UsageBreakdown>) -> Vec<UsageBreakdown> {
    let mut items = groups.into_values().collect::<Vec<_>>();
    items.sort_by(|a, b| {
        b.total_tokens
            .cmp(&a.total_tokens)
            .then_with(|| b.requests.cmp(&a.requests))
            .then_with(|| a.label.cmp(&b.label))
    });
    items.truncate(20);
    items
}

#[tauri::command]
pub fn clear_llm_gateway_logs(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<(), String> {
    state.logs.lock().map_err(|e| e.to_string())?.clear();
    let path = request_logs_path(&app)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("删除请求日志失败：{}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn test_llm_gateway_config(
    config: GatewayConfig,
) -> Result<Vec<ChannelTestResult>, String> {
    validate_config(&config)?;
    let client = Client::new();
    let mut results = Vec::new();

    for channel in config.channels.iter().filter(|channel| channel.enabled) {
        let result = discover_channel_models(&client, channel).await;
        results.push(ChannelTestResult {
            channel_id: result.channel_id,
            channel_name: result.channel_name,
            ok: result.ok,
            status: result.status,
            latency_ms: result.latency_ms,
            message: result.message,
            models: result.models,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn discover_llm_gateway_models(
    config: GatewayConfig,
    channel_id: String,
) -> Result<ModelDiscoveryResult, String> {
    let channel = config
        .channels
        .iter()
        .find(|channel| channel.id == channel_id)
        .cloned()
        .ok_or_else(|| format!("找不到渠道：{}", channel_id))?;
    validate_channel_for_discovery(&channel)?;
    Ok(discover_channel_models(&Client::new(), &channel).await)
}

async fn discover_channel_models(client: &Client, channel: &ChannelConfig) -> ModelDiscoveryResult {
    let started = std::time::Instant::now();
    let endpoint = format!("{}/v1/models", channel.base_url.trim_end_matches('/'));
    let api_key = match channel
        .keys
        .iter()
        .find(|key| key.enabled && !key.api_key.trim().is_empty())
    {
        Some(key) => key.api_key.trim(),
        None => {
            return ModelDiscoveryResult {
                channel_id: channel.id.clone(),
                channel_name: channel.name.clone(),
                ok: false,
                status: None,
                latency_ms: started.elapsed().as_millis(),
                message: "没有可用 API Key".to_string(),
                models: Vec::new(),
            };
        }
    };

    let response = client
        .get(&endpoint)
        .timeout(Duration::from_millis(channel.timeout_ms.max(1000)))
        .bearer_auth(api_key)
        .send()
        .await;

    match response {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.json::<Value>().await.unwrap_or(Value::Null);
            let models = extract_model_ids(&body);
            let ok = (200..300).contains(&status);
            ModelDiscoveryResult {
                channel_id: channel.id.clone(),
                channel_name: channel.name.clone(),
                ok,
                status: Some(status),
                latency_ms: started.elapsed().as_millis(),
                message: if ok {
                    format!("连接成功，发现 {} 个模型", models.len())
                } else {
                    extract_error_message(&body)
                        .unwrap_or_else(|| format!("上游返回 HTTP {}", status))
                },
                models,
            }
        }
        Err(err) => ModelDiscoveryResult {
            channel_id: channel.id.clone(),
            channel_name: channel.name.clone(),
            ok: false,
            status: None,
            latency_ms: started.elapsed().as_millis(),
            message: err.to_string(),
            models: Vec::new(),
        },
    }
}
#[tauri::command]
pub fn list_llm_gateway_profiles(app: tauri::AppHandle) -> Result<Vec<ConfigProfile>, String> {
    let dir = profiles_dir(&app)?;
    let active = load_active_profile_name(&app).ok().flatten();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("读取 Profile 目录失败：{}", e))? {
        let entry = entry.map_err(|e| format!("读取 Profile 失败：{}", e))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        let updated_at = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(DateTime::<Utc>::from);
        profiles.push(ConfigProfile {
            name: name.to_string(),
            active: active.as_deref() == Some(name),
            updated_at,
        });
    }
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

#[tauri::command]
pub fn save_llm_gateway_profile(
    app: tauri::AppHandle,
    name: String,
    config: GatewayConfig,
) -> Result<(), String> {
    validate_profile_name(&name)?;
    validate_config(&config)?;
    let path = profile_path(&app, &name)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 Profile 目录失败：{}", e))?;
    }
    let text =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败：{}", e))?;
    fs::write(path, text).map_err(|e| format!("保存 Profile 失败：{}", e))
}

#[tauri::command]
pub fn load_llm_gateway_profile(
    app: tauri::AppHandle,
    state: tauri::State<'_, LlmGatewayState>,
    name: String,
) -> Result<GatewayConfig, String> {
    if state.is_running() {
        return Err("请先停止本地 LLM API 服务，再切换 Profile".to_string());
    }
    validate_profile_name(&name)?;
    let path = profile_path(&app, &name)?;
    let text = fs::read_to_string(path).map_err(|e| format!("读取 Profile 失败：{}", e))?;
    let config: GatewayConfig =
        serde_json::from_str(&text).map_err(|e| format!("Profile JSON 无效：{}", e))?;
    validate_config(&config)?;
    save_config_to_disk(&app, &config)?;
    save_active_profile_name(&app, &name)?;
    *state.config.write().map_err(|e| e.to_string())? = config.clone();
    state.key_cursors.lock().map_err(|e| e.to_string())?.clear();
    Ok(config)
}

#[tauri::command]
pub fn delete_llm_gateway_profile(app: tauri::AppHandle, name: String) -> Result<(), String> {
    validate_profile_name(&name)?;
    let path = profile_path(&app, &name)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("删除 Profile 失败：{}", e))?;
    }
    if load_active_profile_name(&app)?.as_deref() == Some(name.trim()) {
        let active_path = active_profile_path(&app)?;
        if active_path.exists() {
            fs::remove_file(active_path).map_err(|e| format!("清理活动 Profile 失败：{}", e))?;
        }
    }
    Ok(())
}
async fn pull_gateway_config_from_s3(
    sync: &crate::desktop_settings::S3SyncConfig,
) -> Result<GatewayConfig, String> {
    let value = pull_json_from_s3(sync).await?;
    let config: GatewayConfig =
        serde_json::from_value(value).map_err(|e| format!("S3 配置结构无效：{}", e))?;
    validate_config(&config)?;
    Ok(config)
}

async fn push_gateway_config_to_s3(
    sync: &crate::desktop_settings::S3SyncConfig,
    config: &GatewayConfig,
) -> Result<S3SyncResult, String> {
    let value = serde_json::to_value(config).map_err(|e| format!("序列化配置失败：{}", e))?;
    push_json_to_s3(sync, &value).await
}
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn models(State(state): State<GatewayHttpState>, headers: HeaderMap) -> Response {
    let config = match state.config.read() {
        Ok(config) => config.clone(),
        Err(_) => GatewayConfig::default(),
    };
    if let Err(response) = validate_inbound_auth(&headers, &config) {
        return response;
    }
    let mut models: Vec<String> = config
        .model_routes
        .iter()
        .filter(|route| route.enabled)
        .map(|route| route.public_model.clone())
        .collect();
    models.sort();
    models.dedup();
    json_response(
        StatusCode::OK,
        json!({
            "object": "list",
            "data": models.into_iter().map(|id| json!({
                "id": id,
                "object": "model",
                "created": 0,
                "owned_by": "newbie-space"
            })).collect::<Vec<_>>()
        }),
    )
}

async fn chat_completions(
    State(state): State<GatewayHttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let started = std::time::Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                &format!("请求体不是合法 JSON：{}", err),
            );
        }
    };

    proxy_chat_completion(
        state,
        headers,
        payload,
        started,
        request_id,
        ClientWire::ChatCompletions,
    )
    .await
}

async fn responses(
    State(state): State<GatewayHttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let started = std::time::Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                &format!("请求体不是合法 JSON：{}", err),
            );
        }
    };
    let response_id = format!("resp_{}", request_id.replace('-', ""));
    let chat_payload = match responses_request_to_chat_payload(&payload) {
        Ok(payload) => payload,
        Err(message) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_responses_request",
                &message,
            )
        }
    };

    proxy_chat_completion(
        state,
        headers,
        chat_payload,
        started,
        request_id,
        ClientWire::Responses {
            response_id,
            original_request: payload,
        },
    )
    .await
}

async fn anthropic_messages(
    State(state): State<GatewayHttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let started = std::time::Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                &format!("请求体不是合法 JSON：{}", err),
            );
        }
    };
    let message_id = format!("msg_{}", request_id.replace('-', ""));
    let chat_payload = match anthropic_request_to_chat_payload(&payload) {
        Ok(payload) => payload,
        Err(message) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_anthropic_request",
                &message,
            )
        }
    };

    proxy_chat_completion(
        state,
        headers,
        chat_payload,
        started,
        request_id,
        ClientWire::AnthropicMessages {
            message_id,
            original_request: payload,
        },
    )
    .await
}

async fn anthropic_count_tokens(
    State(state): State<GatewayHttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                &format!("请求体不是合法 JSON：{}", err),
            );
        }
    };
    let config = match state.config.read() {
        Ok(config) => config.clone(),
        Err(err) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_failed",
                &err.to_string(),
            );
        }
    };
    if let Err(response) = validate_inbound_auth(&headers, &config) {
        return response;
    }
    match estimate_anthropic_input_tokens(&payload) {
        Ok(input_tokens) => json_response(
            StatusCode::OK,
            json!({
                "input_tokens": input_tokens
            }),
        ),
        Err(message) => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_anthropic_count_tokens_request",
            &message,
        ),
    }
}

async fn proxy_chat_completion(
    state: GatewayHttpState,
    headers: HeaderMap,
    mut payload: Value,
    started: std::time::Instant,
    request_id: String,
    client_wire: ClientWire,
) -> Response {
    let public_model = match payload.get("model").and_then(Value::as_str) {
        Some(model) => model.to_string(),
        None => {
            return error_response(StatusCode::BAD_REQUEST, "missing_model", "请求体缺少 model")
        }
    };

    let config = match state.config.read() {
        Ok(config) => config.clone(),
        Err(err) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_failed",
                &err.to_string(),
            );
        }
    };
    if let Err(response) = validate_inbound_auth(&headers, &config) {
        append_log(
            &state,
            RequestLog {
                id: request_id,
                created_at: Utc::now(),
                public_model,
                upstream_model: None,
                channel_id: None,
                key_id: None,
                status: StatusCode::UNAUTHORIZED.as_u16(),
                latency_ms: started.elapsed().as_millis(),
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                estimated_cost: 0.0,
                error: Some("本地网关鉴权失败".to_string()),
                input: sanitize_payload_for_log(&payload),
                output: None,
                attempts: Vec::new(),
            },
        );
        return response;
    }

    let _permit = match acquire_request_permit(&state, &config) {
        Ok(permit) => permit,
        Err((status, code, message)) => {
            append_log(
                &state,
                RequestLog {
                    id: request_id,
                    created_at: Utc::now(),
                    public_model,
                    upstream_model: None,
                    channel_id: None,
                    key_id: None,
                    status: status.as_u16(),
                    latency_ms: started.elapsed().as_millis(),
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                    estimated_cost: 0.0,
                    error: Some(message.clone()),
                    input: sanitize_payload_for_log(&payload),
                    output: None,
                    attempts: Vec::new(),
                },
            );
            return error_response(status, code, &message);
        }
    };

    let mut candidates = find_candidates(&config, &public_model);
    if candidates.is_empty() {
        append_log(
            &state,
            RequestLog {
                id: request_id,
                created_at: Utc::now(),
                public_model,
                upstream_model: None,
                channel_id: None,
                key_id: None,
                status: StatusCode::NOT_FOUND.as_u16(),
                latency_ms: started.elapsed().as_millis(),
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                estimated_cost: 0.0,
                error: Some("没有可用的模型映射或渠道".to_string()),
                input: sanitize_payload_for_log(&payload),
                output: None,
                attempts: Vec::new(),
            },
        );
        return error_response(
            StatusCode::NOT_FOUND,
            "model_not_found",
            "没有可用的模型映射或渠道",
        );
    }

    let mut last_error = String::new();
    let mut attempts: Vec<RequestAttemptLog> = Vec::new();
    for (route, channel) in candidates.drain(..) {
        let api_key = match choose_key(&state, &channel) {
            Some(key) => key,
            None => {
                last_error = format!("渠道 {} 没有可用 API Key", channel.name);
                attempts.push(RequestAttemptLog {
                    attempt: 0,
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    key_id: None,
                    upstream_model: route.upstream_model.clone(),
                    endpoint: format!(
                        "{}/v1/chat/completions",
                        channel.base_url.trim_end_matches('/')
                    ),
                    status: None,
                    latency_ms: started.elapsed().as_millis(),
                    upstream_request_id: None,
                    error: Some(last_error.clone()),
                });
                continue;
            }
        };

        payload["model"] = Value::String(route.upstream_model.clone());
        let endpoint = format!(
            "{}/v1/chat/completions",
            channel.base_url.trim_end_matches('/')
        );
        let is_stream = payload
            .get("stream")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if is_stream {
            ensure_stream_usage_options(&mut payload);
        }

        for attempt in 0..=channel.retry_count {
            let response = send_upstream_raw(
                &state.client,
                &endpoint,
                &headers,
                &api_key.api_key,
                &payload,
                channel.timeout_ms,
                &request_id,
            )
            .await;

            match response {
                Ok(response) if response.status().is_success() && is_stream => {
                    let status = response.status().as_u16();
                    let upstream_request_id = extract_upstream_request_id(response.headers());
                    attempts.push(RequestAttemptLog {
                        attempt: attempt + 1,
                        channel_id: channel.id.clone(),
                        channel_name: channel.name.clone(),
                        key_id: Some(api_key.id.clone()),
                        upstream_model: route.upstream_model.clone(),
                        endpoint: endpoint.clone(),
                        status: Some(status),
                        latency_ms: started.elapsed().as_millis(),
                        upstream_request_id,
                        error: None,
                    });
                    append_log(
                        &state,
                        RequestLog {
                            id: request_id.clone(),
                            created_at: Utc::now(),
                            public_model: public_model.clone(),
                            upstream_model: Some(route.upstream_model.clone()),
                            channel_id: Some(channel.id.clone()),
                            key_id: Some(api_key.id.clone()),
                            status,
                            latency_ms: started.elapsed().as_millis(),
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                            estimated_cost: 0.0,
                            error: None,
                            input: sanitize_payload_for_log(&payload),
                            output: Some(json!({ "stream": true })),
                            attempts: attempts.clone(),
                        },
                    );
                    return stream_response(
                        state.clone(),
                        response,
                        request_id.clone(),
                        route.clone(),
                        started,
                        client_wire.clone(),
                    );
                }
                Ok(response) => {
                    let upstream = parse_upstream_response(response).await;
                    match upstream {
                        Ok(upstream) if (200..300).contains(&upstream.status) => {
                            let usage = extract_usage(&upstream.body);
                            let cost = estimate_cost(&route, usage.0, usage.1);
                            let response_body =
                                adapt_chat_response_for_client(&upstream.body, &client_wire);
                            attempts.push(RequestAttemptLog {
                                attempt: attempt + 1,
                                channel_id: channel.id.clone(),
                                channel_name: channel.name.clone(),
                                key_id: Some(api_key.id.clone()),
                                upstream_model: route.upstream_model.clone(),
                                endpoint: endpoint.clone(),
                                status: Some(upstream.status),
                                latency_ms: started.elapsed().as_millis(),
                                upstream_request_id: upstream.request_id.clone(),
                                error: None,
                            });
                            append_log(
                                &state,
                                RequestLog {
                                    id: request_id.clone(),
                                    created_at: Utc::now(),
                                    public_model: public_model.clone(),
                                    upstream_model: Some(route.upstream_model.clone()),
                                    channel_id: Some(channel.id.clone()),
                                    key_id: Some(api_key.id.clone()),
                                    status: upstream.status,
                                    latency_ms: started.elapsed().as_millis(),
                                    prompt_tokens: usage.0,
                                    completion_tokens: usage.1,
                                    total_tokens: usage.2,
                                    estimated_cost: cost,
                                    error: None,
                                    input: sanitize_payload_for_log(&payload),
                                    output: Some(sanitize_output_for_log(&response_body)),
                                    attempts: attempts.clone(),
                                },
                            );
                            return json_response(
                                StatusCode::from_u16(upstream.status).unwrap_or(StatusCode::OK),
                                response_body,
                            );
                        }
                        Ok(upstream) => {
                            last_error =
                                extract_error_message(&upstream.body).unwrap_or_else(|| {
                                    format!("{} 返回 HTTP {}", channel.name, upstream.status)
                                });
                            attempts.push(RequestAttemptLog {
                                attempt: attempt + 1,
                                channel_id: channel.id.clone(),
                                channel_name: channel.name.clone(),
                                key_id: Some(api_key.id.clone()),
                                upstream_model: route.upstream_model.clone(),
                                endpoint: endpoint.clone(),
                                status: Some(upstream.status),
                                latency_ms: started.elapsed().as_millis(),
                                upstream_request_id: upstream.request_id,
                                error: Some(last_error.clone()),
                            });
                        }
                        Err(err) => {
                            last_error = format!("{} 响应解析失败：{}", channel.name, err);
                            attempts.push(RequestAttemptLog {
                                attempt: attempt + 1,
                                channel_id: channel.id.clone(),
                                channel_name: channel.name.clone(),
                                key_id: Some(api_key.id.clone()),
                                upstream_model: route.upstream_model.clone(),
                                endpoint: endpoint.clone(),
                                status: None,
                                latency_ms: started.elapsed().as_millis(),
                                upstream_request_id: None,
                                error: Some(last_error.clone()),
                            });
                        }
                    }
                }
                Err(err) => {
                    last_error = format!("{} 请求失败：{}", channel.name, err);
                    attempts.push(RequestAttemptLog {
                        attempt: attempt + 1,
                        channel_id: channel.id.clone(),
                        channel_name: channel.name.clone(),
                        key_id: Some(api_key.id.clone()),
                        upstream_model: route.upstream_model.clone(),
                        endpoint: endpoint.clone(),
                        status: None,
                        latency_ms: started.elapsed().as_millis(),
                        upstream_request_id: None,
                        error: Some(last_error.clone()),
                    });
                }
            }

            if attempt < channel.retry_count {
                continue;
            }
        }
    }

    append_log(
        &state,
        RequestLog {
            id: request_id,
            created_at: Utc::now(),
            public_model,
            upstream_model: None,
            channel_id: None,
            key_id: None,
            status: StatusCode::BAD_GATEWAY.as_u16(),
            latency_ms: started.elapsed().as_millis(),
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            estimated_cost: 0.0,
            error: Some(last_error.clone()),
            input: sanitize_payload_for_log(&payload),
            output: None,
            attempts,
        },
    );
    error_response(StatusCode::BAD_GATEWAY, "upstream_failed", &last_error)
}

#[derive(Clone)]
enum ClientWire {
    ChatCompletions,
    Responses {
        response_id: String,
        original_request: Value,
    },
    AnthropicMessages {
        message_id: String,
        original_request: Value,
    },
}

fn responses_request_to_chat_payload(request: &Value) -> Result<Value, String> {
    let model = request
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| "Responses 请求缺少 model".to_string())?;

    let mut messages = Vec::new();
    if let Some(instructions) = request.get("instructions").and_then(Value::as_str) {
        if !instructions.trim().is_empty() {
            messages.push(json!({
                "role": "system",
                "content": instructions
            }));
        }
    }

    let input = request
        .get("input")
        .or_else(|| request.get("messages"))
        .ok_or_else(|| "Responses 请求缺少 input".to_string())?;
    append_responses_input_messages(input, &mut messages)?;

    let mut payload = serde_json::Map::new();
    payload.insert("model".to_string(), Value::String(model.to_string()));
    payload.insert("messages".to_string(), Value::Array(messages));

    for key in [
        "temperature",
        "top_p",
        "stream",
        "tool_choice",
        "response_format",
        "parallel_tool_calls",
        "metadata",
        "user",
    ] {
        if let Some(value) = request.get(key) {
            payload.insert(key.to_string(), value.clone());
        }
    }
    if let Some(tools) = request.get("tools") {
        payload.insert("tools".to_string(), responses_tools_to_openai_tools(tools));
    }

    if let Some(value) = request.get("max_output_tokens") {
        payload.insert("max_tokens".to_string(), value.clone());
    }
    if let Some(value) = request.get("max_completion_tokens") {
        payload.insert("max_completion_tokens".to_string(), value.clone());
    }

    Ok(Value::Object(payload))
}

fn append_responses_input_messages(input: &Value, messages: &mut Vec<Value>) -> Result<(), String> {
    match input {
        Value::String(text) => {
            messages.push(json!({
                "role": "user",
                "content": text
            }));
        }
        Value::Array(items) => {
            for item in items {
                append_responses_input_item(item, messages)?;
            }
        }
        Value::Object(_) => append_responses_input_item(input, messages)?,
        _ => return Err("Responses input 必须是字符串、对象或数组".to_string()),
    }
    Ok(())
}

fn append_responses_input_item(item: &Value, messages: &mut Vec<Value>) -> Result<(), String> {
    let Some(map) = item.as_object() else {
        if let Some(text) = item.as_str() {
            messages.push(json!({ "role": "user", "content": text }));
        }
        return Ok(());
    };

    let item_type = map.get("type").and_then(Value::as_str).unwrap_or("");
    if item_type == "function_call" {
        let call_id = map
            .get("call_id")
            .or_else(|| map.get("id"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let name = map.get("name").cloned().unwrap_or(Value::Null);
        let arguments = map
            .get("arguments")
            .map(arguments_value_to_string)
            .unwrap_or_else(|| "{}".to_string());
        messages.push(json!({
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": call_id,
                "type": "function",
                "function": {
                    "name": name,
                    "arguments": arguments
                }
            }]
        }));
        return Ok(());
    }
    if item_type == "function_call_output" {
        let call_id = map.get("call_id").and_then(Value::as_str).unwrap_or("");
        let output = map
            .get("output")
            .map(content_value_to_text)
            .unwrap_or_default();
        messages.push(json!({
            "role": "tool",
            "tool_call_id": call_id,
            "content": output
        }));
        return Ok(());
    }

    let role = map
        .get("role")
        .and_then(Value::as_str)
        .map(normalize_chat_role)
        .unwrap_or("user");
    let content = map
        .get("content")
        .or_else(|| map.get("text"))
        .or_else(|| map.get("input_text"))
        .map(content_value_to_text)
        .unwrap_or_default();

    let tool_calls = map
        .get("tool_calls")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty());
    if role == "assistant" && tool_calls.is_some() {
        messages.push(json!({
            "role": role,
            "content": if content.trim().is_empty() { Value::Null } else { Value::String(content) },
            "tool_calls": tool_calls.cloned().unwrap_or_default()
        }));
    } else if !content.trim().is_empty() {
        messages.push(json!({
            "role": role,
            "content": content
        }));
    }
    Ok(())
}

fn normalize_chat_role(role: &str) -> &'static str {
    match role {
        "system" | "developer" => "system",
        "assistant" => "assistant",
        "tool" => "tool",
        _ => "user",
    }
}

fn content_value_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(content_value_to_text)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(map) => map
            .get("text")
            .or_else(|| map.get("input_text"))
            .or_else(|| map.get("output_text"))
            .or_else(|| map.get("content"))
            .map(content_value_to_text)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn arguments_value_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "{}".to_string(),
        _ => value.to_string(),
    }
}

fn responses_tools_to_openai_tools(value: &Value) -> Value {
    let tools = value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|tool| {
                    let map = tool.as_object()?;
                    if map.get("function").is_some() {
                        return Some(Value::Object(map.clone()));
                    }
                    match map.get("type").and_then(Value::as_str).unwrap_or("function") {
                        "function" => Some(json!({
                            "type": "function",
                            "function": {
                                "name": map.get("name").cloned().unwrap_or(Value::String("tool".to_string())),
                                "description": map.get("description").cloned().unwrap_or(Value::String(String::new())),
                                "parameters": map.get("parameters").cloned().unwrap_or(json!({ "type": "object", "properties": {} }))
                            }
                        })),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Value::Array(tools)
}

fn adapt_chat_response_for_client(chat_body: &Value, wire: &ClientWire) -> Value {
    match wire {
        ClientWire::ChatCompletions => chat_body.clone(),
        ClientWire::Responses {
            response_id,
            original_request,
        } => chat_response_to_responses_response(chat_body, response_id, original_request),
        ClientWire::AnthropicMessages {
            message_id,
            original_request,
        } => chat_response_to_anthropic_message(chat_body, message_id, original_request),
    }
}

fn anthropic_request_to_chat_payload(request: &Value) -> Result<Value, String> {
    let model = request
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| "Anthropic Messages 请求缺少 model".to_string())?;

    let mut messages = Vec::new();
    if let Some(system) = request.get("system") {
        let system_text = anthropic_content_to_text(system);
        if !system_text.trim().is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system_text
            }));
        }
    }

    let items = request
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| "Anthropic Messages 请求缺少 messages 数组".to_string())?;
    for item in items {
        append_anthropic_message_item(item, &mut messages);
    }

    let mut payload = serde_json::Map::new();
    payload.insert("model".to_string(), Value::String(model.to_string()));
    payload.insert("messages".to_string(), Value::Array(messages));

    if let Some(max_tokens) = request.get("max_tokens") {
        payload.insert("max_tokens".to_string(), max_tokens.clone());
    }
    for key in ["temperature", "top_p", "stream", "metadata"] {
        if let Some(value) = request.get(key) {
            payload.insert(key.to_string(), value.clone());
        }
    }
    if let Some(tools) = request.get("tools") {
        payload.insert("tools".to_string(), anthropic_tools_to_openai_tools(tools));
    }
    if let Some(tool_choice) = request.get("tool_choice") {
        payload.insert(
            "tool_choice".to_string(),
            anthropic_tool_choice_to_openai(tool_choice),
        );
    }

    Ok(Value::Object(payload))
}

fn estimate_anthropic_input_tokens(request: &Value) -> Result<u64, String> {
    let mut chars = 0_u64;
    if let Some(system) = request.get("system") {
        chars += anthropic_content_to_text(system).chars().count() as u64;
    }

    let items = request
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| "Anthropic token 统计请求缺少 messages 数组".to_string())?;
    for item in items {
        if let Some(content) = item.get("content") {
            chars += anthropic_content_to_text(content).chars().count() as u64;
        }
        if let Some(role) = item.get("role").and_then(Value::as_str) {
            chars += role.chars().count() as u64;
        }
    }
    if let Some(tools) = request.get("tools") {
        chars += tools.to_string().chars().count() as u64;
    }
    Ok(estimate_tokens_from_chars(chars))
}

fn append_anthropic_message_item(item: &Value, messages: &mut Vec<Value>) {
    let Some(map) = item.as_object() else {
        return;
    };
    let role = match map.get("role").and_then(Value::as_str) {
        Some("assistant") => "assistant",
        Some("tool") => "tool",
        _ => "user",
    };

    if let Some(content) = map.get("content") {
        let mut tool_results = Vec::new();
        let content_text = anthropic_content_to_text_with_tool_results(content, &mut tool_results);
        let tool_calls = if role == "assistant" {
            anthropic_tool_uses_to_openai_tool_calls(content)
        } else {
            Vec::new()
        };
        if !tool_calls.is_empty() {
            messages.push(json!({
                "role": role,
                "content": if content_text.trim().is_empty() { Value::Null } else { Value::String(content_text) },
                "tool_calls": tool_calls
            }));
        } else if !content_text.trim().is_empty() {
            messages.push(json!({
                "role": role,
                "content": content_text
            }));
        }
        messages.extend(tool_results);
    }
}

fn anthropic_content_to_text(value: &Value) -> String {
    let mut tool_results = Vec::new();
    anthropic_content_to_text_with_tool_results(value, &mut tool_results)
}

fn anthropic_content_to_text_with_tool_results(
    value: &Value,
    tool_results: &mut Vec<Value>,
) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| anthropic_content_block_to_text(item, tool_results))
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(_) => {
            anthropic_content_block_to_text(value, tool_results).unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn anthropic_content_block_to_text(block: &Value, tool_results: &mut Vec<Value>) -> Option<String> {
    let map = block.as_object()?;
    match map.get("type").and_then(Value::as_str).unwrap_or("text") {
        "text" => map
            .get("text")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        "tool_result" => {
            let tool_use_id = map
                .get("tool_use_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let content = map
                .get("content")
                .map(anthropic_content_to_text)
                .unwrap_or_default();
            tool_results.push(json!({
                "role": "tool",
                "tool_call_id": tool_use_id,
                "content": content
            }));
            None
        }
        "image" => Some("[image input omitted by local gateway]".to_string()),
        _ => map
            .get("text")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

fn anthropic_tool_uses_to_openai_tool_calls(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(anthropic_tool_use_to_openai_tool_call)
            .collect(),
        Value::Object(_) => anthropic_tool_use_to_openai_tool_call(value)
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn anthropic_tool_use_to_openai_tool_call(value: &Value) -> Option<Value> {
    let map = value.as_object()?;
    if map.get("type").and_then(Value::as_str)? != "tool_use" {
        return None;
    }
    let id = map
        .get("id")
        .cloned()
        .unwrap_or(Value::String(format!("call_{}", Uuid::new_v4().simple())));
    Some(json!({
        "id": id,
        "type": "function",
        "function": {
            "name": map.get("name").cloned().unwrap_or(Value::String("tool".to_string())),
            "arguments": map.get("input").map(arguments_value_to_string).unwrap_or_else(|| "{}".to_string())
        }
    }))
}

fn anthropic_tools_to_openai_tools(value: &Value) -> Value {
    let tools = value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|tool| {
                    let map = tool.as_object()?;
                    Some(json!({
                        "type": "function",
                        "function": {
                            "name": map.get("name").cloned().unwrap_or(Value::String("tool".to_string())),
                            "description": map.get("description").cloned().unwrap_or(Value::String(String::new())),
                            "parameters": map.get("input_schema").cloned().unwrap_or(json!({ "type": "object", "properties": {} }))
                        }
                    }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Value::Array(tools)
}

fn anthropic_tool_choice_to_openai(value: &Value) -> Value {
    let Some(map) = value.as_object() else {
        return Value::String("auto".to_string());
    };
    match map.get("type").and_then(Value::as_str).unwrap_or("auto") {
        "any" | "auto" => Value::String("auto".to_string()),
        "none" => Value::String("none".to_string()),
        "tool" => {
            let name = map.get("name").cloned().unwrap_or(Value::Null);
            json!({
                "type": "function",
                "function": { "name": name }
            })
        }
        _ => Value::String("auto".to_string()),
    }
}

fn chat_response_to_anthropic_message(
    chat_body: &Value,
    message_id: &str,
    original_request: &Value,
) -> Value {
    let model = original_request
        .get("model")
        .and_then(Value::as_str)
        .or_else(|| chat_body.get("model").and_then(Value::as_str))
        .unwrap_or("");
    let choice = chat_body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .cloned()
        .unwrap_or(Value::Null);
    let message = choice.get("message").unwrap_or(&Value::Null);
    let mut content = Vec::new();
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        if !text.is_empty() {
            content.push(json!({
                "type": "text",
                "text": text
            }));
        }
    }
    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in tool_calls {
            let function = call.get("function").unwrap_or(&Value::Null);
            let arguments = function
                .get("arguments")
                .and_then(Value::as_str)
                .and_then(|text| serde_json::from_str::<Value>(text).ok())
                .unwrap_or_else(|| {
                    function
                        .get("arguments")
                        .cloned()
                        .unwrap_or_else(|| json!({}))
                });
            content.push(json!({
                "type": "tool_use",
                "id": call.get("id").cloned().unwrap_or(Value::Null),
                "name": function.get("name").cloned().unwrap_or(Value::Null),
                "input": arguments
            }));
        }
    }
    if content.is_empty() {
        content.push(json!({
            "type": "text",
            "text": ""
        }));
    }
    let finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .unwrap_or("stop");
    let stop_reason = match finish_reason {
        "length" => "max_tokens",
        "tool_calls" => "tool_use",
        _ => "end_turn",
    };
    let usage = extract_usage(chat_body);

    json!({
        "id": message_id,
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": content,
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": {
            "input_tokens": usage.0,
            "output_tokens": usage.1
        }
    })
}

fn chat_response_to_responses_response(
    chat_body: &Value,
    response_id: &str,
    original_request: &Value,
) -> Value {
    let model = original_request
        .get("model")
        .and_then(Value::as_str)
        .or_else(|| chat_body.get("model").and_then(Value::as_str))
        .unwrap_or("");
    let created_at = now_millis() / 1000;
    let choice = chat_body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .cloned()
        .unwrap_or(Value::Null);
    let message = choice.get("message").unwrap_or(&Value::Null);
    let output_text = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .unwrap_or("stop");
    let usage = extract_usage(chat_body);
    let mut output = vec![json!({
        "id": format!("msg_{}", response_id.trim_start_matches("resp_")),
        "type": "message",
        "status": "completed",
        "role": "assistant",
        "content": [{
            "type": "output_text",
            "text": output_text.clone(),
            "annotations": []
        }]
    })];

    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in tool_calls {
            let function = call.get("function").unwrap_or(&Value::Null);
            output.push(json!({
                "type": "function_call",
                "id": call.get("id").cloned().unwrap_or(Value::Null),
                "call_id": call.get("id").cloned().unwrap_or(Value::Null),
                "name": function.get("name").cloned().unwrap_or(Value::Null),
                "arguments": function.get("arguments").cloned().unwrap_or(Value::String("{}".to_string())),
                "status": "completed"
            }));
        }
    }

    json!({
        "id": response_id,
        "object": "response",
        "created_at": created_at,
        "status": "completed",
        "background": false,
        "error": null,
        "incomplete_details": null,
        "instructions": original_request.get("instructions").cloned().unwrap_or(Value::Null),
        "max_output_tokens": original_request.get("max_output_tokens").cloned().unwrap_or(Value::Null),
        "model": model,
        "output": output,
        "output_text": output_text,
        "parallel_tool_calls": original_request.get("parallel_tool_calls").cloned().unwrap_or(Value::Bool(true)),
        "previous_response_id": original_request.get("previous_response_id").cloned().unwrap_or(Value::Null),
        "reasoning": original_request.get("reasoning").cloned().unwrap_or(json!({})),
        "store": original_request.get("store").cloned().unwrap_or(Value::Bool(false)),
        "temperature": original_request.get("temperature").cloned().unwrap_or(Value::Null),
        "tool_choice": original_request.get("tool_choice").cloned().unwrap_or(Value::String("auto".to_string())),
        "tools": original_request.get("tools").cloned().unwrap_or(json!([])),
        "top_p": original_request.get("top_p").cloned().unwrap_or(Value::Null),
        "truncation": original_request.get("truncation").cloned().unwrap_or(Value::String("disabled".to_string())),
        "usage": {
            "input_tokens": usage.0,
            "output_tokens": usage.1,
            "total_tokens": usage.2
        },
        "metadata": original_request.get("metadata").cloned().unwrap_or(json!({})),
        "finish_reason": finish_reason
    })
}

fn chat_stream_bytes_to_responses_bytes(bytes: &Bytes, response_id: &str) -> Bytes {
    let text = String::from_utf8_lossy(bytes);
    let mut out = String::new();
    for line in text.lines() {
        let Some(data) = line.trim().strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" {
            push_responses_sse_event(
                &mut out,
                "response.completed",
                json!({
                    "type": "response.completed",
                    "response": {
                        "id": response_id,
                        "status": "completed"
                    }
                }),
            );
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if let Some(choices) = value.get("choices").and_then(Value::as_array) {
            for choice in choices {
                if let Some(delta) = choice.get("delta") {
                    if let Some(content) = delta.get("content").and_then(Value::as_str) {
                        push_responses_sse_event(
                            &mut out,
                            "response.output_text.delta",
                            json!({
                                "type": "response.output_text.delta",
                                "response_id": response_id,
                                "output_index": 0,
                                "content_index": 0,
                                "delta": content
                            }),
                        );
                    }
                }
                if choice
                    .get("finish_reason")
                    .and_then(Value::as_str)
                    .is_some()
                {
                    push_responses_sse_event(
                        &mut out,
                        "response.output_text.done",
                        json!({
                            "type": "response.output_text.done",
                            "response_id": response_id,
                            "output_index": 0,
                            "content_index": 0
                        }),
                    );
                }
            }
        }
    }
    Bytes::from(out)
}

fn push_responses_sse_event(out: &mut String, event: &str, data: Value) {
    out.push_str("event: ");
    out.push_str(event);
    out.push('\n');
    out.push_str("data: ");
    out.push_str(&data.to_string());
    out.push_str("\n\n");
}

fn chat_stream_bytes_to_anthropic_bytes(
    bytes: &Bytes,
    message_id: &str,
    original_request: &Value,
) -> Bytes {
    let text = String::from_utf8_lossy(bytes);
    let model = original_request
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut out = String::new();
    for line in text.lines() {
        let Some(data) = line.trim().strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" {
            push_anthropic_sse_event(
                &mut out,
                "content_block_stop",
                json!({
                    "type": "content_block_stop",
                    "index": 0
                }),
            );
            push_anthropic_sse_event(
                &mut out,
                "message_stop",
                json!({
                    "type": "message_stop"
                }),
            );
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if let Some(choices) = value.get("choices").and_then(Value::as_array) {
            for choice in choices {
                if let Some(delta) = choice.get("delta") {
                    if delta.get("role").and_then(Value::as_str).is_some() {
                        push_anthropic_sse_event(
                            &mut out,
                            "message_start",
                            json!({
                                "type": "message_start",
                                "message": {
                                    "id": message_id,
                                    "type": "message",
                                    "role": "assistant",
                                    "model": model,
                                    "content": [],
                                    "stop_reason": null,
                                    "stop_sequence": null,
                                    "usage": {
                                        "input_tokens": 0,
                                        "output_tokens": 0
                                    }
                                }
                            }),
                        );
                        push_anthropic_sse_event(
                            &mut out,
                            "content_block_start",
                            json!({
                                "type": "content_block_start",
                                "index": 0,
                                "content_block": {
                                    "type": "text",
                                    "text": ""
                                }
                            }),
                        );
                    }
                    if let Some(content) = delta.get("content").and_then(Value::as_str) {
                        push_anthropic_sse_event(
                            &mut out,
                            "content_block_delta",
                            json!({
                                "type": "content_block_delta",
                                "index": 0,
                                "delta": {
                                    "type": "text_delta",
                                    "text": content
                                }
                            }),
                        );
                    }
                }
                if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                    let stop_reason = match reason {
                        "length" => "max_tokens",
                        "tool_calls" => "tool_use",
                        _ => "end_turn",
                    };
                    push_anthropic_sse_event(
                        &mut out,
                        "message_delta",
                        json!({
                            "type": "message_delta",
                            "delta": {
                                "stop_reason": stop_reason,
                                "stop_sequence": null
                            },
                            "usage": {
                                "output_tokens": 0
                            }
                        }),
                    );
                }
            }
        }
        let usage = extract_usage(&value);
        if usage.2 > 0 {
            push_anthropic_sse_event(
                &mut out,
                "message_delta",
                json!({
                    "type": "message_delta",
                    "delta": {},
                    "usage": {
                        "output_tokens": usage.1
                    }
                }),
            );
        }
    }
    Bytes::from(out)
}

fn push_anthropic_sse_event(out: &mut String, event: &str, data: Value) {
    out.push_str("event: ");
    out.push_str(event);
    out.push('\n');
    out.push_str("data: ");
    out.push_str(&data.to_string());
    out.push_str("\n\n");
}

struct UpstreamResponse {
    status: u16,
    body: Value,
    request_id: Option<String>,
}

async fn send_upstream_raw(
    client: &Client,
    endpoint: &str,
    incoming_headers: &HeaderMap,
    api_key: &str,
    payload: &Value,
    timeout_ms: u64,
    request_id: &str,
) -> Result<reqwest::Response, String> {
    let mut request = client
        .request(Method::POST, endpoint)
        .timeout(Duration::from_millis(timeout_ms.max(1000)))
        .bearer_auth(api_key)
        .header("x-request-id", request_id)
        .header("x-gateway-request-id", request_id)
        .json(payload);

    if let Some(value) = incoming_headers.get(header::ACCEPT) {
        request = request.header(header::ACCEPT, value.clone());
    }

    request.send().await.map_err(|e| e.to_string())
}

async fn parse_upstream_response(response: reqwest::Response) -> Result<UpstreamResponse, String> {
    let status = response.status().as_u16();
    let request_id = extract_upstream_request_id(response.headers());
    let body = response.json::<Value>().await.unwrap_or_else(|_| {
        json!({
            "error": {
                "message": "上游返回了非 JSON 响应",
                "type": "invalid_upstream_response"
            }
        })
    });
    Ok(UpstreamResponse {
        status,
        body,
        request_id,
    })
}

fn extract_upstream_request_id(headers: &reqwest::header::HeaderMap) -> Option<String> {
    for name in [
        "x-request-id",
        "x-openai-request-id",
        "request-id",
        "x-amzn-requestid",
        "cf-ray",
    ] {
        if let Some(value) = headers.get(name).and_then(|value| value.to_str().ok()) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn stream_response(
    state: GatewayHttpState,
    response: reqwest::Response,
    log_id: String,
    route: ModelRouteConfig,
    started: std::time::Instant,
    client_wire: ClientWire,
) -> Response {
    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::OK);
    let accumulator = Arc::new(Mutex::new(StreamLogAccumulator::default()));
    let stream_state = state.clone();
    let stream_log_id = log_id.clone();
    let stream_route = route.clone();
    let stream_accumulator = Arc::clone(&accumulator);
    let response_wire = client_wire.clone();

    let stream = response
        .bytes_stream()
        .map(move |chunk_result| match chunk_result {
            Ok(bytes) => {
                if let Ok(mut acc) = stream_accumulator.lock() {
                    acc.ingest_bytes(&bytes);
                    update_stream_log(
                        &stream_state,
                        &stream_log_id,
                        &stream_route,
                        &acc,
                        started.elapsed().as_millis(),
                        None,
                    );
                }
                let outgoing = match &response_wire {
                    ClientWire::ChatCompletions => bytes,
                    ClientWire::Responses { response_id, .. } => {
                        chat_stream_bytes_to_responses_bytes(&bytes, response_id)
                    }
                    ClientWire::AnthropicMessages {
                        message_id,
                        original_request,
                    } => chat_stream_bytes_to_anthropic_bytes(&bytes, message_id, original_request),
                };
                Ok::<Bytes, reqwest::Error>(outgoing)
            }
            Err(err) => {
                if let Ok(acc) = stream_accumulator.lock() {
                    update_stream_log(
                        &stream_state,
                        &stream_log_id,
                        &stream_route,
                        &acc,
                        started.elapsed().as_millis(),
                        Some(err.to_string()),
                    );
                }
                Err(err)
            }
        });

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/event-stream; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "stream_failed",
                "创建流式响应失败",
            )
        })
}

#[derive(Default)]
struct StreamLogAccumulator {
    chunks: u64,
    content_chars: u64,
    output_text: String,
    reasoning_text: String,
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    finish_reason: Option<String>,
    completed: bool,
    buffer: String,
}

impl StreamLogAccumulator {
    fn ingest_bytes(&mut self, bytes: &Bytes) {
        let text = String::from_utf8_lossy(bytes);
        self.buffer.push_str(&text);
        while let Some(pos) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=pos).collect();
            self.ingest_sse_line(line.trim_end_matches(['\r', '\n']));
        }
    }

    fn ingest_sse_line(&mut self, line: &str) {
        let Some(data) = line.strip_prefix("data:") else {
            return;
        };
        let data = data.trim();
        if data.is_empty() {
            return;
        }
        if data == "[DONE]" {
            self.completed = true;
            return;
        }
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            return;
        };
        self.chunks += 1;
        let usage = extract_usage(&value);
        if usage.2 > 0 {
            self.prompt_tokens = usage.0;
            self.completion_tokens = usage.1;
            self.total_tokens = usage.2;
        }
        if let Some(choices) = value.get("choices").and_then(Value::as_array) {
            for choice in choices {
                if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                    self.finish_reason = Some(reason.to_string());
                }
                if let Some(delta) = choice.get("delta") {
                    if let Some(content) = delta.get("content").and_then(Value::as_str) {
                        self.content_chars += content.chars().count() as u64;
                        self.output_text.push_str(content);
                    }
                    if let Some(reasoning) = delta.get("reasoning_content").and_then(Value::as_str)
                    {
                        self.reasoning_text.push_str(reasoning);
                    }
                }
            }
        }
    }

    fn effective_usage(&self) -> (u64, u64, u64) {
        if self.total_tokens > 0 {
            return (
                self.prompt_tokens,
                self.completion_tokens,
                self.total_tokens,
            );
        }
        let estimated_completion = estimate_tokens_from_chars(self.content_chars);
        (0, estimated_completion, estimated_completion)
    }
}

fn update_stream_log(
    state: &GatewayHttpState,
    log_id: &str,
    route: &ModelRouteConfig,
    acc: &StreamLogAccumulator,
    latency_ms: u128,
    error: Option<String>,
) {
    let usage = acc.effective_usage();
    let estimated_cost = estimate_cost(route, usage.0, usage.1);
    let mut updated_log = None;
    if let Ok(mut logs) = state.logs.lock() {
        if let Some(log) = logs.iter_mut().find(|log| log.id == log_id) {
            log.latency_ms = latency_ms;
            log.prompt_tokens = usage.0;
            log.completion_tokens = usage.1;
            log.total_tokens = usage.2;
            log.estimated_cost = estimated_cost;
            if let Some(error) = error {
                log.error = Some(error);
            }
            log.output = Some(json!({
                "stream": true,
                "chunks": acc.chunks,
                "text": acc.output_text,
                "reasoning_text": acc.reasoning_text,
                "content_chars": acc.content_chars,
                "finish_reason": acc.finish_reason,
                "usage": {
                    "prompt_tokens": usage.0,
                    "completion_tokens": usage.1,
                    "total_tokens": usage.2,
                    "estimated": acc.total_tokens == 0
                }
            }));
            if acc.completed || log.error.is_some() {
                updated_log = Some(log.clone());
            }
        }
    }
    if let Some(log) = updated_log {
        if let Ok(path_guard) = state.log_path.read() {
            if let Some(path) = path_guard.as_ref() {
                let _ = replace_request_log_on_disk(path, &log);
            }
        }
    }
}

fn estimate_tokens_from_chars(chars: u64) -> u64 {
    if chars == 0 {
        0
    } else {
        ((chars as f64) / 4.0).ceil() as u64
    }
}

fn ensure_stream_usage_options(payload: &mut Value) {
    let Some(map) = payload.as_object_mut() else {
        return;
    };
    let options = map
        .entry("stream_options".to_string())
        .or_insert_with(|| json!({}));
    if let Some(options) = options.as_object_mut() {
        options
            .entry("include_usage".to_string())
            .or_insert(Value::Bool(true));
    }
}
fn validate_inbound_auth(headers: &HeaderMap, config: &GatewayConfig) -> Result<(), Response> {
    if !config.inbound_auth_enabled {
        return Ok(());
    }

    let expected = config.inbound_api_key.trim();
    if expected.is_empty() {
        return Err(error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "gateway_auth_misconfigured",
            "本地网关已启用鉴权，但未配置入站 API Key",
        ));
    }

    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ").or(Some(value)))
        .map(str::trim);
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim);

    if bearer == Some(expected) || api_key == Some(expected) {
        Ok(())
    } else {
        Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_gateway_api_key",
            "缺少或无效的本地网关 API Key",
        ))
    }
}
fn acquire_request_permit(
    state: &GatewayHttpState,
    config: &GatewayConfig,
) -> Result<RequestPermit, (StatusCode, &'static str, String)> {
    let now = now_millis();
    let window_start = now.saturating_sub(60_000);
    let mut limiter = state.limiter.lock().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "limiter_lock_failed",
            err.to_string(),
        )
    })?;
    limiter
        .recent_requests
        .retain(|timestamp| *timestamp >= window_start);

    if config.max_concurrent_requests > 0
        && limiter.active_requests >= config.max_concurrent_requests
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "concurrency_limit_exceeded",
            format!(
                "本地代理并发请求已达到上限 {}",
                config.max_concurrent_requests
            ),
        ));
    }
    if config.requests_per_minute > 0
        && limiter.recent_requests.len() as u32 >= config.requests_per_minute
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_exceeded",
            format!(
                "本地代理每分钟请求数已达到上限 {}",
                config.requests_per_minute
            ),
        ));
    }

    limiter.active_requests += 1;
    limiter.recent_requests.push(now);
    Ok(RequestPermit {
        limiter: Arc::clone(&state.limiter),
    })
}
fn find_candidates(
    config: &GatewayConfig,
    public_model: &str,
) -> Vec<(ModelRouteConfig, ChannelConfig)> {
    let channel_map: HashMap<String, ChannelConfig> = config
        .channels
        .iter()
        .filter(|channel| channel.enabled)
        .map(|channel| (channel.id.clone(), channel.clone()))
        .collect();

    let mut routes: Vec<(ModelRouteConfig, ChannelConfig)> = config
        .model_routes
        .iter()
        .filter(|route| route.enabled && route.public_model == public_model)
        .filter_map(|route| {
            channel_map
                .get(&route.channel_id)
                .map(|channel| (route.clone(), channel.clone()))
        })
        .collect();

    routes.sort_by_key(|(route, channel)| (route.priority, channel.priority));
    routes
}

fn choose_key(state: &GatewayHttpState, channel: &ChannelConfig) -> Option<ApiKeyConfig> {
    let keys: Vec<ApiKeyConfig> = channel
        .keys
        .iter()
        .filter(|key| key.enabled && !key.api_key.trim().is_empty())
        .cloned()
        .collect();
    if keys.is_empty() {
        return None;
    }

    match channel.key_strategy {
        KeyStrategy::Random => {
            let idx = pseudo_random_index(keys.len());
            keys.get(idx).cloned()
        }
        KeyStrategy::RoundRobin => {
            let mut cursors = state.key_cursors.lock().ok()?;
            let cursor = cursors.entry(channel.id.clone()).or_insert(0);
            let key = keys.get(*cursor % keys.len()).cloned();
            *cursor = (*cursor + 1) % keys.len();
            key
        }
    }
}

fn pseudo_random_index(len: usize) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    (nanos as usize) % len
}

fn extract_usage(body: &Value) -> (u64, u64, u64) {
    let usage = body.get("usage").unwrap_or(&Value::Null);
    let prompt = usage
        .get("prompt_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let completion = usage
        .get("completion_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total = usage
        .get("total_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(prompt + completion);
    (prompt, completion, total)
}

fn extract_model_ids(body: &Value) -> Vec<String> {
    let mut models: Vec<String> = body
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();
    models
}

fn extract_error_message(body: &Value) -> Option<String> {
    body.get("error")
        .and_then(|error| error.get("message").or_else(|| error.get("code")))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}
fn estimate_cost(route: &ModelRouteConfig, prompt_tokens: u64, completion_tokens: u64) -> f64 {
    (prompt_tokens as f64 / 1000.0 * route.prompt_cost_per_1k)
        + (completion_tokens as f64 / 1000.0 * route.completion_cost_per_1k)
}

fn sanitize_payload_for_log(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, value) in map {
                let lower = key.to_lowercase();
                if matches!(
                    lower.as_str(),
                    "api_key" | "authorization" | "access_token" | "refresh_token"
                ) {
                    sanitized.insert(key.clone(), Value::String("[redacted]".to_string()));
                } else if matches!(lower.as_str(), "messages" | "input" | "prompt" | "choices") {
                    sanitized.insert(key.clone(), summarize_sensitive_value(value));
                } else {
                    sanitized.insert(key.clone(), sanitize_payload_for_log(value));
                }
            }
            Value::Object(sanitized)
        }
        Value::Array(items) => Value::Array(items.iter().map(sanitize_payload_for_log).collect()),
        _ => value.clone(),
    }
}

fn sanitize_output_for_log(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, value) in map {
                let lower = key.to_lowercase();
                if matches!(
                    lower.as_str(),
                    "api_key" | "authorization" | "access_token" | "refresh_token"
                ) {
                    sanitized.insert(key.clone(), Value::String("[redacted]".to_string()));
                } else {
                    sanitized.insert(key.clone(), sanitize_output_for_log(value));
                }
            }
            Value::Object(sanitized)
        }
        Value::Array(items) => Value::Array(items.iter().map(sanitize_output_for_log).collect()),
        _ => value.clone(),
    }
}

fn summarize_sensitive_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => json!({
            "redacted": true,
            "items": items.len()
        }),
        Value::String(text) => json!({
            "redacted": true,
            "chars": text.chars().count()
        }),
        Value::Object(map) => json!({
            "redacted": true,
            "fields": map.len()
        }),
        _ => Value::String("[redacted]".to_string()),
    }
}
fn append_log(state: &GatewayHttpState, log: RequestLog) {
    if let Ok(mut logs) = state.logs.lock() {
        logs.push(log.clone());
        if logs.len() > 1000 {
            let overflow = logs.len() - 1000;
            logs.drain(0..overflow);
        }
    }
    if let Ok(path_guard) = state.log_path.read() {
        if let Some(path) = path_guard.as_ref() {
            let _ = append_request_log_to_disk(path, &log);
        }
    }
}

fn request_logs_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法获取配置目录：{}", e))?
        .join("llm-gateway-requests.jsonl"))
}

fn append_request_log_to_disk(path: &PathBuf, log: &RequestLog) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建日志目录失败：{}", e))?;
    }
    let line = serde_json::to_string(log).map_err(|e| format!("序列化请求日志失败：{}", e))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开请求日志失败：{}", e))?;
    writeln!(file, "{}", line).map_err(|e| format!("写入请求日志失败：{}", e))
}

fn replace_request_log_on_disk(path: &PathBuf, updated_log: &RequestLog) -> Result<(), String> {
    if !path.exists() {
        return append_request_log_to_disk(path, updated_log);
    }
    let text = fs::read_to_string(path).map_err(|e| format!("读取请求日志失败：{}", e))?;
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<RequestLog>(trimmed) {
            Ok(log) if log.id == updated_log.id => {
                lines.push(
                    serde_json::to_string(updated_log)
                        .map_err(|e| format!("序列化请求日志失败：{}", e))?,
                );
                replaced = true;
            }
            _ => lines.push(trimmed.to_string()),
        }
    }
    if !replaced {
        lines.push(
            serde_json::to_string(updated_log).map_err(|e| format!("序列化请求日志失败：{}", e))?,
        );
    }
    fs::write(path, lines.join("\n")).map_err(|e| format!("写入请求日志失败：{}", e))
}

fn read_request_logs_from_disk(
    path: &PathBuf,
    limit: Option<usize>,
) -> Result<Vec<RequestLog>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(|e| format!("读取请求日志失败：{}", e))?;
    let mut logs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(log) = serde_json::from_str::<RequestLog>(line) {
            logs.push(log);
        }
    }
    logs.reverse();
    if let Some(limit) = limit {
        logs.truncate(limit);
    }
    Ok(logs)
}

fn profiles_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法获取配置目录：{}", e))?
        .join("llm-gateway-profiles"))
}

fn profile_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    validate_profile_name(name)?;
    Ok(profiles_dir(app)?.join(format!("{}.json", name.trim())))
}

fn active_profile_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法获取配置目录：{}", e))?
        .join("llm-gateway-active-profile.txt"))
}

fn load_active_profile_name(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let path = active_profile_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let name = fs::read_to_string(path).map_err(|e| format!("读取活动 Profile 失败：{}", e))?;
    let name = name.trim().to_string();
    if name.is_empty() {
        Ok(None)
    } else {
        Ok(Some(name))
    }
}

fn save_active_profile_name(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let path = active_profile_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败：{}", e))?;
    }
    fs::write(path, name.trim()).map_err(|e| format!("写入活动 Profile 失败：{}", e))
}

fn validate_profile_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Profile 名称不能为空".to_string());
    }
    if name.len() > 64 {
        return Err("Profile 名称不能超过 64 个字符".to_string());
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err("Profile 名称只能包含英文字母、数字、横线和下划线".to_string());
    }
    Ok(())
}
fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法获取配置目录：{}", e))?;
    Ok(dir.join("llm-gateway.json"))
}

fn load_config_from_disk(app: &tauri::AppHandle) -> Result<GatewayConfig, String> {
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(GatewayConfig::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("读取配置失败：{}", e))?;
    if text.trim().is_empty() {
        return Ok(GatewayConfig::default());
    }
    let config: GatewayConfig =
        serde_json::from_str(&text).map_err(|e| format!("配置 JSON 无效：{}", e))?;
    validate_config(&config)?;
    Ok(config)
}

fn save_config_to_disk(app: &tauri::AppHandle, config: &GatewayConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败：{}", e))?;
    }
    let text =
        serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败：{}", e))?;
    fs::write(&path, text).map_err(|e| format!("写入配置失败：{}", e))
}
fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn validate_channel_for_discovery(channel: &ChannelConfig) -> Result<(), String> {
    if channel.id.trim().is_empty() {
        return Err("渠道 ID 不能为空".to_string());
    }
    if channel.base_url.trim().is_empty() {
        return Err(format!("渠道 {} 的 base_url 不能为空", channel.name));
    }
    if channel.timeout_ms < 1000 {
        return Err(format!("渠道 {} 的超时时间不能小于 1000ms", channel.name));
    }
    if !channel
        .keys
        .iter()
        .any(|key| key.enabled && !key.api_key.trim().is_empty())
    {
        return Err(format!("渠道 {} 至少需要一个可用 API Key", channel.name));
    }
    Ok(())
}
fn validate_config(config: &GatewayConfig) -> Result<(), String> {
    let listen_host = config.listen_host.trim();
    if listen_host.is_empty() {
        return Err("监听地址不能为空".to_string());
    }
    if !matches!(listen_host, "127.0.0.1" | "localhost" | "::1" | "[::1]") {
        return Err("本地 LLM API 代理只能监听 127.0.0.1、localhost 或 ::1".to_string());
    }
    if config.listen_port == 0 {
        return Err("监听端口必须在 1-65535 之间".to_string());
    }
    if config.max_concurrent_requests > 256 {
        return Err("最大并发请求不能超过 256".to_string());
    }
    if config.requests_per_minute > 60_000 {
        return Err("每分钟请求数不能超过 60000".to_string());
    }
    if config.inbound_auth_enabled && config.inbound_api_key.trim().is_empty() {
        return Err("启用本地网关鉴权时，入站 API Key 不能为空".to_string());
    }

    let mut channel_ids = HashMap::new();
    let mut route_ids = HashMap::new();

    for channel in &config.channels {
        let channel_id = channel.id.trim();
        if channel_id.is_empty() {
            return Err("渠道 ID 不能为空".to_string());
        }
        if channel_ids.insert(channel_id.to_string(), true).is_some() {
            return Err(format!("渠道 ID 重复：{}", channel.id));
        }
        if channel.name.trim().is_empty() {
            return Err(format!("渠道 {} 的名称不能为空", channel.id));
        }
        if channel.base_url.trim().is_empty() {
            return Err(format!("渠道 {} 的 base_url 不能为空", channel.name));
        }
        if channel.timeout_ms < 1000 {
            return Err(format!("渠道 {} 的超时时间不能小于 1000ms", channel.name));
        }
        if channel.retry_count > 5 {
            return Err(format!("渠道 {} 的重试次数不能超过 5", channel.name));
        }
        if channel.enabled
            && !channel
                .keys
                .iter()
                .any(|key| key.enabled && !key.api_key.trim().is_empty())
        {
            return Err(format!(
                "启用的渠道 {} 至少需要一个可用 API Key",
                channel.name
            ));
        }

        let mut key_ids = HashMap::new();
        for key in &channel.keys {
            if key.id.trim().is_empty() {
                return Err(format!("渠道 {} 的 Key ID 不能为空", channel.name));
            }
            if key_ids.insert(key.id.trim().to_string(), true).is_some() {
                return Err(format!("渠道 {} 的 Key ID 重复：{}", channel.name, key.id));
            }
            if key.enabled && key.api_key.trim().is_empty() {
                return Err(format!("渠道 {} 中启用的 Key 不能为空", channel.name));
            }
        }
    }

    for route in &config.model_routes {
        if route.id.trim().is_empty() {
            return Err("模型映射 ID 不能为空".to_string());
        }
        if route_ids
            .insert(route.id.trim().to_string(), true)
            .is_some()
        {
            return Err(format!("模型映射 ID 重复：{}", route.id));
        }
        if route.public_model.trim().is_empty() || route.upstream_model.trim().is_empty() {
            return Err("模型映射的 public_model/upstream_model 不能为空".to_string());
        }
        if !channel_ids.contains_key(route.channel_id.trim()) {
            return Err(format!(
                "模型映射 {} 引用了不存在的渠道 {}",
                route.public_model, route.channel_id
            ));
        }
    }
    Ok(())
}
fn build_status(state: &LlmGatewayState) -> Result<GatewayStatus, String> {
    let config = state.config.read().map_err(|e| e.to_string())?;
    let logs = state.logs.lock().map_err(|e| e.to_string())?;
    Ok(GatewayStatus {
        running: state.is_running(),
        listen_url: format!("http://{}:{}", config.listen_host, config.listen_port),
        channel_count: config.channels.len(),
        enabled_channel_count: config
            .channels
            .iter()
            .filter(|channel| channel.enabled)
            .count(),
        route_count: config.model_routes.len(),
        request_count: logs.len(),
    })
}

fn json_response(status: StatusCode, body: Value) -> Response {
    let mut response = Json(body).into_response();
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    json_response(
        status,
        json!({
            "error": {
                "message": message,
                "type": code,
                "code": code
            }
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses_function_call_history_becomes_chat_tool_calls() {
        let payload = responses_request_to_chat_payload(&json!({
            "model": "agent-model",
            "tools": [{
                "type": "function",
                "name": "read_file",
                "description": "Read a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    }
                }
            }],
            "input": [
                {
                    "role": "user",
                    "content": "Open README"
                },
                {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "read_file",
                    "arguments": { "path": "README.md" }
                },
                {
                    "type": "function_call_output",
                    "call_id": "call_1",
                    "output": "hello"
                }
            ]
        }))
        .expect("responses payload should convert");

        let messages = payload
            .get("messages")
            .and_then(Value::as_array)
            .expect("messages array");
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["tool_calls"][0]["id"], "call_1");
        assert_eq!(
            messages[1]["tool_calls"][0]["function"]["name"],
            "read_file"
        );
        assert_eq!(
            messages[1]["tool_calls"][0]["function"]["arguments"],
            "{\"path\":\"README.md\"}"
        );
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "call_1");

        let tools = payload
            .get("tools")
            .and_then(Value::as_array)
            .expect("tools array");
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "read_file");
    }

    #[test]
    fn anthropic_tool_use_history_becomes_chat_tool_calls() {
        let payload = anthropic_request_to_chat_payload(&json!({
            "model": "claude-agent",
            "max_tokens": 128,
            "tools": [{
                "name": "shell",
                "description": "Run a shell command",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "cmd": { "type": "string" }
                    }
                }
            }],
            "messages": [
                {
                    "role": "user",
                    "content": "pwd"
                },
                {
                    "role": "assistant",
                    "content": [{
                        "type": "tool_use",
                        "id": "toolu_1",
                        "name": "shell",
                        "input": { "cmd": "pwd" }
                    }]
                },
                {
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": "toolu_1",
                        "content": "/tmp/work"
                    }]
                }
            ]
        }))
        .expect("anthropic payload should convert");

        let messages = payload
            .get("messages")
            .and_then(Value::as_array)
            .expect("messages array");
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["tool_calls"][0]["id"], "toolu_1");
        assert_eq!(messages[1]["tool_calls"][0]["function"]["name"], "shell");
        assert_eq!(
            messages[1]["tool_calls"][0]["function"]["arguments"],
            "{\"cmd\":\"pwd\"}"
        );
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "toolu_1");
        assert_eq!(messages[2]["content"], "/tmp/work");

        let tools = payload
            .get("tools")
            .and_then(Value::as_array)
            .expect("tools array");
        assert_eq!(tools[0]["function"]["name"], "shell");
        assert_eq!(tools[0]["function"]["parameters"]["type"], "object");
    }

    #[test]
    fn chat_tool_calls_convert_to_anthropic_and_responses_outputs() {
        let chat_body = json!({
            "model": "upstream-model",
            "choices": [{
                "finish_reason": "tool_calls",
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_abc",
                        "type": "function",
                        "function": {
                            "name": "read_file",
                            "arguments": "{\"path\":\"README.md\"}"
                        }
                    }]
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        });

        let anthropic = chat_response_to_anthropic_message(
            &chat_body,
            "msg_test",
            &json!({ "model": "claude-facing" }),
        );
        assert_eq!(anthropic["stop_reason"], "tool_use");
        assert_eq!(anthropic["content"][0]["type"], "tool_use");
        assert_eq!(anthropic["content"][0]["id"], "call_abc");
        assert_eq!(anthropic["content"][0]["input"]["path"], "README.md");

        let responses = chat_response_to_responses_response(
            &chat_body,
            "resp_test",
            &json!({ "model": "responses-facing" }),
        );
        assert_eq!(responses["output"][1]["type"], "function_call");
        assert_eq!(responses["output"][1]["call_id"], "call_abc");
        assert_eq!(responses["output"][1]["name"], "read_file");
        assert_eq!(
            responses["output"][1]["arguments"],
            "{\"path\":\"README.md\"}"
        );
    }

    #[test]
    fn usage_breakdowns_sort_by_total_tokens_then_requests() {
        let mut groups = HashMap::new();
        let log_a = RequestLog {
            id: "a".to_string(),
            created_at: Utc::now(),
            public_model: "model-a".to_string(),
            upstream_model: Some("up-a".to_string()),
            channel_id: Some("ch-a".to_string()),
            key_id: Some("key-a".to_string()),
            status: 200,
            latency_ms: 10,
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            estimated_cost: 0.3,
            error: None,
            input: json!({}),
            output: None,
            attempts: Vec::new(),
        };
        let log_b = RequestLog {
            total_tokens: 50,
            estimated_cost: 0.5,
            ..log_a.clone()
        };

        add_usage_breakdown(&mut groups, "model-a", "model-a", &log_a, true);
        add_usage_breakdown(&mut groups, "model-b", "model-b", &log_b, false);
        let items = sorted_usage_breakdowns(groups);

        assert_eq!(items[0].id, "model-b");
        assert_eq!(items[0].failed, 1);
        assert_eq!(items[0].total_tokens, 50);
        assert_eq!(items[1].id, "model-a");
        assert_eq!(items[1].success, 1);
        assert_eq!(items[1].estimated_cost, 0.3);
    }
}
