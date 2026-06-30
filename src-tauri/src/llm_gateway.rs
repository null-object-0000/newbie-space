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

#[derive(Clone, Debug, Serialize)]
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
    key_cursors: Arc<Mutex<HashMap<String, usize>>>,
    limiter: Arc<Mutex<RequestLimiter>>,
    server: Mutex<ServerHandle>,
}

impl Default for LlmGatewayState {
    fn default() -> Self {
        Self {
            config: Arc::new(RwLock::new(GatewayConfig::default())),
            logs: Arc::new(Mutex::new(Vec::new())),
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

    let (tx, rx) = oneshot::channel::<()>();
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state.http_state());

    let task = tauri::async_runtime::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
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
    state: tauri::State<'_, LlmGatewayState>,
    limit: Option<usize>,
) -> Result<Vec<RequestLog>, String> {
    let logs = state.logs.lock().map_err(|e| e.to_string())?;
    let take = limit.unwrap_or(100).min(500);
    Ok(logs.iter().rev().take(take).cloned().collect())
}

#[tauri::command]
pub fn get_llm_gateway_usage(
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<UsageSummary, String> {
    let logs = state.logs.lock().map_err(|e| e.to_string())?;
    let mut summary = UsageSummary {
        requests: logs.len(),
        success: 0,
        failed: 0,
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
        estimated_cost: 0.0,
    };
    for log in logs.iter() {
        if (200..300).contains(&log.status) {
            summary.success += 1;
        } else {
            summary.failed += 1;
        }
        summary.prompt_tokens += log.prompt_tokens;
        summary.completion_tokens += log.completion_tokens;
        summary.total_tokens += log.total_tokens;
        summary.estimated_cost += log.estimated_cost;
    }
    Ok(summary)
}

#[tauri::command]
pub fn clear_llm_gateway_logs(state: tauri::State<'_, LlmGatewayState>) -> Result<(), String> {
    state.logs.lock().map_err(|e| e.to_string())?.clear();
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

async fn models(State(state): State<GatewayHttpState>) -> Json<Value> {
    let config = match state.config.read() {
        Ok(config) => config.clone(),
        Err(_) => GatewayConfig::default(),
    };
    let mut models: Vec<String> = config
        .model_routes
        .iter()
        .filter(|route| route.enabled)
        .map(|route| route.public_model.clone())
        .collect();
    models.sort();
    models.dedup();
    Json(json!({
        "object": "list",
        "data": models.into_iter().map(|id| json!({
            "id": id,
            "object": "model",
            "created": 0,
            "owned_by": "newbie-space"
        })).collect::<Vec<_>>()
    }))
}

async fn chat_completions(
    State(state): State<GatewayHttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let started = std::time::Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let mut payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                &format!("请求体不是合法 JSON：{}", err),
            );
        }
    };

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
            },
        );
        return error_response(
            StatusCode::NOT_FOUND,
            "model_not_found",
            "没有可用的模型映射或渠道",
        );
    }

    let mut last_error = String::new();
    for (route, channel) in candidates.drain(..) {
        let api_key = match choose_key(&state, &channel) {
            Some(key) => key,
            None => {
                last_error = format!("渠道 {} 没有可用 API Key", channel.name);
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
            )
            .await;

            match response {
                Ok(response) if response.status().is_success() && is_stream => {
                    let status = response.status().as_u16();
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
                        },
                    );
                    return stream_response(
                        state.clone(),
                        response,
                        request_id.clone(),
                        route.clone(),
                        started,
                    );
                }
                Ok(response) => {
                    let upstream = parse_upstream_response(response).await;
                    match upstream {
                        Ok(upstream) if (200..300).contains(&upstream.status) => {
                            let usage = extract_usage(&upstream.body);
                            let cost = estimate_cost(&route, usage.0, usage.1);
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
                                    output: Some(sanitize_payload_for_log(&upstream.body)),
                                },
                            );
                            return json_response(
                                StatusCode::from_u16(upstream.status).unwrap_or(StatusCode::OK),
                                upstream.body,
                            );
                        }
                        Ok(upstream) => {
                            last_error =
                                extract_error_message(&upstream.body).unwrap_or_else(|| {
                                    format!("{} 返回 HTTP {}", channel.name, upstream.status)
                                });
                        }
                        Err(err) => {
                            last_error = format!("{} 响应解析失败：{}", channel.name, err);
                        }
                    }
                }
                Err(err) => {
                    last_error = format!("{} 请求失败：{}", channel.name, err);
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
        },
    );
    error_response(StatusCode::BAD_GATEWAY, "upstream_failed", &last_error)
}

struct UpstreamResponse {
    status: u16,
    body: Value,
}

async fn send_upstream_raw(
    client: &Client,
    endpoint: &str,
    incoming_headers: &HeaderMap,
    api_key: &str,
    payload: &Value,
    timeout_ms: u64,
) -> Result<reqwest::Response, String> {
    let mut request = client
        .request(Method::POST, endpoint)
        .timeout(Duration::from_millis(timeout_ms.max(1000)))
        .bearer_auth(api_key)
        .json(payload);

    if let Some(value) = incoming_headers.get(header::ACCEPT) {
        request = request.header(header::ACCEPT, value.clone());
    }

    request.send().await.map_err(|e| e.to_string())
}

async fn parse_upstream_response(response: reqwest::Response) -> Result<UpstreamResponse, String> {
    let status = response.status().as_u16();
    let body = response.json::<Value>().await.unwrap_or_else(|_| {
        json!({
            "error": {
                "message": "上游返回了非 JSON 响应",
                "type": "invalid_upstream_response"
            }
        })
    });
    Ok(UpstreamResponse { status, body })
}

fn stream_response(
    state: GatewayHttpState,
    response: reqwest::Response,
    log_id: String,
    route: ModelRouteConfig,
    started: std::time::Instant,
) -> Response {
    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::OK);
    let accumulator = Arc::new(Mutex::new(StreamLogAccumulator::default()));
    let stream_state = state.clone();
    let stream_log_id = log_id.clone();
    let stream_route = route.clone();
    let stream_accumulator = Arc::clone(&accumulator);

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
                Ok::<Bytes, reqwest::Error>(bytes)
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
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    finish_reason: Option<String>,
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
        if data.is_empty() || data == "[DONE]" {
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
                    self.content_chars += delta
                        .get("content")
                        .or_else(|| delta.get("reasoning_content"))
                        .and_then(Value::as_str)
                        .map(|text| text.chars().count() as u64)
                        .unwrap_or(0);
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
                "content_chars": acc.content_chars,
                "finish_reason": acc.finish_reason,
                "usage": {
                    "prompt_tokens": usage.0,
                    "completion_tokens": usage.1,
                    "total_tokens": usage.2,
                    "estimated": acc.total_tokens == 0
                }
            }));
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
        logs.push(log);
        if logs.len() > 1000 {
            let overflow = logs.len() - 1000;
            logs.drain(0..overflow);
        }
    }
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
