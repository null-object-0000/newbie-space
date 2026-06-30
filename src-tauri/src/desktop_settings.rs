use chrono::Utc;
use reqwest::{Client, Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesktopSettings {
    #[serde(default = "default_startup_path")]
    pub startup_path: String,
    #[serde(default)]
    pub s3_sync: S3SyncConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S3SyncConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default = "default_s3_region")]
    pub region: String,
    #[serde(default)]
    pub bucket: String,
    #[serde(default = "default_s3_object_key")]
    pub object_key: String,
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default)]
    pub secret_access_key: String,
    #[serde(default = "default_true")]
    pub path_style: bool,
    #[serde(default)]
    pub auto_pull_on_start: bool,
    #[serde(default)]
    pub auto_push_on_save: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct S3SyncResult {
    pub ok: bool,
    pub message: String,
    pub status: Option<u16>,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            startup_path: default_startup_path(),
            s3_sync: S3SyncConfig::default(),
        }
    }
}

impl Default for S3SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: String::new(),
            region: default_s3_region(),
            bucket: String::new(),
            object_key: default_s3_object_key(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            path_style: true,
            auto_pull_on_start: false,
            auto_push_on_save: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_startup_path() -> String {
    "/".to_string()
}

fn default_s3_region() -> String {
    "auto".to_string()
}

fn default_s3_object_key() -> String {
    "newbie-space/llm-gateway.json".to_string()
}

#[tauri::command]
pub fn get_desktop_settings(app: tauri::AppHandle) -> Result<DesktopSettings, String> {
    load_desktop_settings(&app)
}

#[tauri::command]
pub fn save_desktop_settings(
    app: tauri::AppHandle,
    settings: DesktopSettings,
) -> Result<(), String> {
    validate_desktop_settings(&settings)?;
    save_desktop_settings_to_disk(&app, &settings)
}

pub fn load_desktop_settings(app: &tauri::AppHandle) -> Result<DesktopSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(DesktopSettings::default());
    }
    let text = fs::read_to_string(path).map_err(|e| format!("读取桌面设置失败：{}", e))?;
    if text.trim().is_empty() {
        return Ok(DesktopSettings::default());
    }
    let settings: DesktopSettings =
        serde_json::from_str(&text).map_err(|e| format!("桌面设置 JSON 无效：{}", e))?;
    validate_desktop_settings(&settings)?;
    Ok(settings)
}

fn save_desktop_settings_to_disk(
    app: &tauri::AppHandle,
    settings: &DesktopSettings,
) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败：{}", e))?;
    }
    let text =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化桌面设置失败：{}", e))?;
    fs::write(path, text).map_err(|e| format!("写入桌面设置失败：{}", e))
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法获取配置目录：{}", e))?
        .join("desktop-settings.json"))
}

fn validate_desktop_settings(settings: &DesktopSettings) -> Result<(), String> {
    validate_startup_path(&settings.startup_path)?;
    if settings.s3_sync.enabled {
        validate_s3_config(&settings.s3_sync)?;
    }
    Ok(())
}

fn validate_startup_path(path: &str) -> Result<(), String> {
    let allowed = [
        "/",
        "/nav/",
        "/posts",
        "/projects",
        "/tools",
        "/tools/llm-gateway",
        "/tools/port-check",
        "/tools/file-usage-check",
        "/tools/process-manager",
        "/tools/http-client",
        "/tools/json-formatter",
    ];
    if allowed.contains(&path) || path.starts_with("/posts/") {
        Ok(())
    } else {
        Err(format!("不支持的启动页面：{}", path))
    }
}

pub fn validate_s3_config(config: &S3SyncConfig) -> Result<(), String> {
    if config.endpoint.trim().is_empty() {
        return Err("S3 endpoint 不能为空".to_string());
    }
    if config.bucket.trim().is_empty() {
        return Err("S3 bucket 不能为空".to_string());
    }
    if config.object_key.trim().is_empty() {
        return Err("S3 object key 不能为空".to_string());
    }
    if config.access_key_id.trim().is_empty() || config.secret_access_key.trim().is_empty() {
        return Err("S3 access key 和 secret key 不能为空".to_string());
    }
    Url::parse(config.endpoint.trim()).map_err(|e| format!("S3 endpoint 无效：{}", e))?;
    Ok(())
}

pub async fn pull_json_from_s3(config: &S3SyncConfig) -> Result<Value, String> {
    validate_s3_config(config)?;
    let request = build_signed_s3_request(config, Method::GET, Vec::new())?;
    let response = Client::new()
        .execute(request)
        .await
        .map_err(|e| format!("S3 下载失败：{}", e))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取 S3 响应失败：{}", e))?;
    if !status.is_success() {
        return Err(format!("S3 下载返回 HTTP {}：{}", status.as_u16(), text));
    }
    serde_json::from_str(&text).map_err(|e| format!("S3 配置 JSON 无效：{}", e))
}

pub async fn push_json_to_s3(config: &S3SyncConfig, value: &Value) -> Result<S3SyncResult, String> {
    validate_s3_config(config)?;
    let body =
        serde_json::to_vec_pretty(value).map_err(|e| format!("序列化同步配置失败：{}", e))?;
    let request = build_signed_s3_request(config, Method::PUT, body)?;
    let response = Client::new()
        .execute(request)
        .await
        .map_err(|e| format!("S3 上传失败：{}", e))?;
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    if (200..300).contains(&status) {
        Ok(S3SyncResult {
            ok: true,
            message: "S3 配置已上传".to_string(),
            status: Some(status),
        })
    } else {
        Err(format!("S3 上传返回 HTTP {}：{}", status, text))
    }
}

fn build_signed_s3_request(
    config: &S3SyncConfig,
    method: Method,
    body: Vec<u8>,
) -> Result<reqwest::Request, String> {
    let endpoint = config.endpoint.trim().trim_end_matches('/');
    let bucket = config.bucket.trim();
    let key = config.object_key.trim().trim_start_matches('/');
    let encoded_key = key
        .split('/')
        .map(aws_uri_encode)
        .collect::<Vec<_>>()
        .join("/");
    let url = if config.path_style {
        format!("{}/{}/{}", endpoint, aws_uri_encode(bucket), encoded_key)
    } else {
        let mut parsed = Url::parse(endpoint).map_err(|e| format!("S3 endpoint 无效：{}", e))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| "S3 endpoint 缺少 host".to_string())?
            .to_string();
        parsed
            .set_host(Some(&format!("{}.{}", bucket, host)))
            .map_err(|_| "设置 S3 virtual host 失败".to_string())?;
        parsed.set_path(&encoded_key);
        parsed.to_string()
    };

    let parsed_url = Url::parse(&url).map_err(|e| format!("S3 URL 无效：{}", e))?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| "S3 URL 缺少 host".to_string())?
        .to_string();
    let host = if let Some(port) = parsed_url.port() {
        format!("{}:{}", host, port)
    } else {
        host
    };
    let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let date = &amz_date[..8];
    let payload_hash = hex::encode(Sha256::digest(&body));
    let canonical_uri = parsed_url.path();
    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host, payload_hash, amz_date
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!(
        "{}\n{}\n\n{}\n{}\n{}",
        method.as_str(),
        canonical_uri,
        canonical_headers,
        signed_headers,
        payload_hash
    );
    let credential_scope = format!("{}/{}/s3/aws4_request", date, config.region.trim());
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        hex::encode(Sha256::digest(canonical_request.as_bytes()))
    );
    let signing_key = signing_key(config.secret_access_key.trim(), date, config.region.trim());
    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        config.access_key_id.trim(),
        credential_scope,
        signed_headers,
        signature
    );

    Client::new()
        .request(method, parsed_url)
        .header("host", host)
        .header("x-amz-content-sha256", payload_hash)
        .header("x-amz-date", amz_date)
        .header("authorization", authorization)
        .header("content-type", "application/json")
        .body(body)
        .build()
        .map_err(|e| format!("创建 S3 请求失败：{}", e))
}

fn signing_key(secret: &str, date: &str, region: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{}", secret).as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    hmac_sha256(&k_service, b"aws4_request")
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut normalized_key = if key.len() > 64 {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    normalized_key.resize(64, 0);

    let mut ipad = [0x36_u8; 64];
    let mut opad = [0x5c_u8; 64];
    for (idx, byte) in normalized_key.iter().enumerate() {
        ipad[idx] ^= byte;
        opad[idx] ^= byte;
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    outer.finalize().to_vec()
}

fn aws_uri_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}
