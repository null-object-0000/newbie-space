use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStrategy {
    RoundRobin,
    Random,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub enabled: bool,
    pub weight: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
    pub priority: u32,
    pub key_strategy: KeyStrategy,
    pub timeout_ms: u64,
    pub keys: Vec<ApiKeyConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelRouteConfig {
    pub id: String,
    pub public_model: String,
    pub upstream_model: String,
    pub channel_id: String,
    pub enabled: bool,
    pub priority: u32,
    pub prompt_cost_per_1k: f64,
    pub completion_cost_per_1k: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub listen_host: String,
    pub listen_port: u16,
    pub channels: Vec<ChannelConfig>,
    pub model_routes: Vec<ModelRouteConfig>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_host: "127.0.0.1".to_string(),
            listen_port: 11434,
            channels: Vec::new(),
            model_routes: Vec::new(),
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
    client: Client,
}

pub struct LlmGatewayState {
    config: Arc<RwLock<GatewayConfig>>,
    logs: Arc<Mutex<Vec<RequestLog>>>,
    key_cursors: Arc<Mutex<HashMap<String, usize>>>,
    server: Mutex<ServerHandle>,
}

impl Default for LlmGatewayState {
    fn default() -> Self {
        Self {
            config: Arc::new(RwLock::new(GatewayConfig::default())),
            logs: Arc::new(Mutex::new(Vec::new())),
            key_cursors: Arc::new(Mutex::new(HashMap::new())),
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
pub fn get_llm_gateway_config(
    state: tauri::State<'_, LlmGatewayState>,
) -> Result<GatewayConfig, String> {
    state
        .config
        .read()
        .map(|config| config.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_llm_gateway_config(
    state: tauri::State<'_, LlmGatewayState>,
    config: GatewayConfig,
) -> Result<(), String> {
    if state.is_running() {
        return Err("请先停止本地 LLM API 服务，再修改配置".to_string());
    }
    validate_config(&config)?;
    *state.config.write().map_err(|e| e.to_string())? = config;
    state.key_cursors.lock().map_err(|e| e.to_string())?.clear();
    Ok(())
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
        let started = std::time::Instant::now();
        let endpoint = format!("{}/v1/models", channel.base_url.trim_end_matches('/'));
        let api_key = match channel
            .keys
            .iter()
            .find(|key| key.enabled && !key.api_key.trim().is_empty())
        {
            Some(key) => key.api_key.trim(),
            None => {
                results.push(ChannelTestResult {
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    ok: false,
                    status: None,
                    latency_ms: started.elapsed().as_millis(),
                    message: "没有可用 API Key".to_string(),
                    models: Vec::new(),
                });
                continue;
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
                results.push(ChannelTestResult {
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    ok: (200..300).contains(&status),
                    status: Some(status),
                    latency_ms: started.elapsed().as_millis(),
                    message: if (200..300).contains(&status) {
                        format!("连接成功，发现 {} 个模型", models.len())
                    } else {
                        extract_error_message(&body)
                            .unwrap_or_else(|| format!("上游返回 HTTP {}", status))
                    },
                    models,
                });
            }
            Err(err) => {
                results.push(ChannelTestResult {
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    ok: false,
                    status: None,
                    latency_ms: started.elapsed().as_millis(),
                    message: err.to_string(),
                    models: Vec::new(),
                });
            }
        }
    }

    Ok(results)
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
        let response = send_upstream(
            &state.client,
            &endpoint,
            &headers,
            &api_key.api_key,
            &payload,
            channel.timeout_ms,
        )
        .await;
        match response {
            Ok(upstream) if (200..300).contains(&upstream.status) => {
                let usage = extract_usage(&upstream.body);
                let cost = estimate_cost(&route, usage.0, usage.1);
                append_log(
                    &state,
                    RequestLog {
                        id: request_id,
                        created_at: Utc::now(),
                        public_model: public_model.clone(),
                        upstream_model: Some(route.upstream_model),
                        channel_id: Some(channel.id),
                        key_id: Some(api_key.id),
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
                last_error = format!("{} 返回 HTTP {}", channel.name, upstream.status);
            }
            Err(err) => {
                last_error = format!("{} 请求失败：{}", channel.name, err);
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

async fn send_upstream(
    client: &Client,
    endpoint: &str,
    incoming_headers: &HeaderMap,
    api_key: &str,
    payload: &Value,
    timeout_ms: u64,
) -> Result<UpstreamResponse, String> {
    let mut request = client
        .request(Method::POST, endpoint)
        .timeout(Duration::from_millis(timeout_ms.max(1000)))
        .bearer_auth(api_key)
        .json(payload);

    if let Some(value) = incoming_headers.get(header::ACCEPT) {
        request = request.header(header::ACCEPT, value.clone());
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
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

fn validate_config(config: &GatewayConfig) -> Result<(), String> {
    if config.listen_host.trim().is_empty() {
        return Err("监听地址不能为空".to_string());
    }
    if config.listen_port == 0 {
        return Err("监听端口必须在 1-65535 之间".to_string());
    }
    for channel in &config.channels {
        if channel.id.trim().is_empty() {
            return Err("渠道 ID 不能为空".to_string());
        }
        if channel.base_url.trim().is_empty() {
            return Err(format!("渠道 {} 的 base_url 不能为空", channel.name));
        }
        if channel.timeout_ms < 1000 {
            return Err(format!("渠道 {} 的超时时间不能小于 1000ms", channel.name));
        }
    }
    for route in &config.model_routes {
        if route.public_model.trim().is_empty() || route.upstream_model.trim().is_empty() {
            return Err("模型映射的 public_model/upstream_model 不能为空".to_string());
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
