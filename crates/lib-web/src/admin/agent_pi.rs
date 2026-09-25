//! pi agent 适配接口：按当前可用模型生成 pi 的 models.json。
//!
//! pi 的模型配置格式与网关的 [OI] 接口不同，相关逻辑全部集中在本文件。

use anyhow::{Context, Result};
use framework_core::MultiStringValue;
use framework_proc_auto::auto_type;
use framework_web::{Json, WebBody, WebError, WebResponse, web_api_post};
use framework_web_axum::use_axum;
use lib_core::APP_ID;
use serde_json::{Map, Value, json};
use service_admin::service::ProviderModelService;
use std::fs;
use std::path::{Path, PathBuf};
use types_admin::entity::ProviderModel;

/// pi 模型配置文件名。
const MODELS_FILE: &str = "models.json";

/// pi 读取 API Key 的默认环境变量名。
const DEFAULT_KEY_NAME: &str = "LINGTING_AI_GATEWAY_API_KEY";

/// pi 适配接口参数。
#[auto_type]
pub struct PiAgentPO {
    /// pi 目录；为空时使用 `~/.pi/agent`。
    dir: Option<String>,
    /// models.json 中的访问目标；为空时使用 `http://localhost:{当前服务端口}`。
    host: Option<String>,
    /// API Key 环境变量名；为空时使用 `LINGTING_AI_GATEWAY_API_KEY`。
    key_name: Option<String>,
}

impl PiAgentPO {
    /// pi 目录：默认 `~/.pi/agent`；传入目录不存在时创建，并自动识别其中的 agent 目录。
    pub fn dir(&self) -> Result<String> {
        let dir = match self
            .dir
            .as_deref()
            .map(str::trim)
            .filter(|dir| !dir.is_empty())
        {
            Some(dir) => PathBuf::from(dir),
            None => home_directory()?.join(".pi").join("agent"),
        };

        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("创建 pi 目录失败：{}", dir.display()))?;
        }
        if dir.file_name().is_some_and(|name| name == "agent") {
            return Ok(dir.to_string_lossy().into_owned());
        }

        let agent = dir.join("agent");
        if agent.is_dir() {
            return Ok(agent.to_string_lossy().into_owned());
        }
        Ok(dir.to_string_lossy().into_owned())
    }

    /// pi 访问当前网关的地址：默认取当前服务端口，非 http 开头时自动补全协议。
    pub fn host(&self) -> Result<String> {
        let host = match self
            .host
            .as_deref()
            .map(str::trim)
            .filter(|host| !host.is_empty())
        {
            Some(host) => host.to_string(),
            None => format!("http://localhost:{}", use_axum()?.port),
        };
        let host = host.trim_end_matches('/');
        if has_http_scheme(host) {
            return Ok(host.to_string());
        }
        Ok(format!("http://{host}"))
    }

    /// models.json 中 apiKey 使用的环境变量名。
    fn key_name(&self) -> &str {
        self.key_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(DEFAULT_KEY_NAME)
    }
}

/// 导出 pi 模型配置：按当前可用模型生成 models.json 内容。
#[web_api_post(path = "/___/agent/pi/export")]
pub async fn agent_pi_export(Json(params): Json<PiAgentPO>) -> Result<WebResponse> {
    let content = build_models(&params).await?;
    Ok(json_response(content))
}

/// 同步 pi 模型配置：备份已有的 models.json 后覆写，返回备份文件路径，无旧文件时返回空。
#[web_api_post(path = "/___/agent/pi/sync")]
pub async fn agent_pi_sync(Json(params): Json<PiAgentPO>) -> Result<Option<String>> {
    let content = build_models(&params).await?;
    let directory = PathBuf::from(params.dir()?);
    fs::create_dir_all(&directory)
        .with_context(|| format!("创建 pi 目录失败：{}", directory.display()))?;

    let path = directory.join(MODELS_FILE);
    let backup = backup_models(&path)?;
    fs::write(&path, content)
        .with_context(|| format!("写入 pi 模型配置失败：{}", path.display()))?;

    Ok(backup)
}

/// 组装 pi 的 models.json 内容：全部启用模型挂到网关供应商下。
async fn build_models(params: &PiAgentPO) -> Result<String> {
    let models = ProviderModelService::new()?.find_enabled_distinct().await?;
    if models.is_empty() {
        return Err(WebError::message("当前没有可用模型，无法生成 pi 模型配置").into());
    }

    let models = models.into_iter().map(pi_model).collect::<Vec<_>>();
    let mut providers = Map::new();
    providers.insert(
        APP_ID.to_string(),
        json!({
            "baseUrl": format!("{}/v1", params.host()?),
            "api": "openai-responses",
            "apiKey": format!("${}", params.key_name()),
            "models": models,
        }),
    );

    serde_json::to_string_pretty(&json!({ "providers": providers })).context("生成 pi 模型配置失败")
}

/// 单个网关模型转换为 pi 模型配置：推理级别按原名转小写作为思考等级键。
fn pi_model(model: ProviderModel) -> Value {
    let mut value = Map::new();
    value.insert("id".into(), Value::String(model.model));
    if !model.display_name.trim().is_empty() {
        value.insert("name".into(), Value::String(model.display_name));
    }
    if model.reasoning {
        value.insert("reasoning".into(), Value::Bool(true));
    }

    let input = if model.support_vision {
        json!(["text", "image"])
    } else {
        json!(["text"])
    };
    value.insert("input".into(), input);

    if model.context_window > 0 {
        value.insert("contextWindow".into(), json!(model.context_window));
    }
    if model.max_tokens > 0 {
        value.insert("maxTokens".into(), json!(model.max_tokens));
    }
    if !model.levels.is_empty() {
        let levels = model
            .levels
            .iter()
            .map(|level| (level.to_ascii_lowercase(), Value::String(level.clone())))
            .collect::<Map<String, Value>>();
        value.insert("thinkingLevelMap".into(), Value::Object(levels));
    }

    Value::Object(value)
}

/// 备份 models.json 到 `models.json.年月日时分秒.bak`，返回备份文件路径；无旧文件时返回空。
fn backup_models(path: &Path) -> Result<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }

    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let backup = path.with_file_name(format!("{MODELS_FILE}.{timestamp}.bak"));
    fs::copy(path, &backup)
        .with_context(|| format!("备份 pi 模型配置失败：{}", backup.display()))?;

    Ok(Some(backup.to_string_lossy().into_owned()))
}

/// 构造原始 JSON 响应，不套用网关统一响应体。
fn json_response(content: String) -> WebResponse {
    let mut headers = MultiStringValue::default();
    headers.set_content_type("application/json; charset=utf-8");
    headers.set_content_length(content.len());

    WebResponse {
        status: 200,
        headers,
        body: WebBody::from(content.into_bytes()),
    }
}

/// 当前用户目录：Windows 取 USERPROFILE，其余平台取 HOME。
fn home_directory() -> Result<PathBuf> {
    let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(variable)
        .map(PathBuf::from)
        .with_context(|| format!("未设置 {variable} 环境变量"))
}

/// 是否已带 http/https 协议前缀。
fn has_http_scheme(host: &str) -> bool {
    host.split_once("://").is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    })
}
