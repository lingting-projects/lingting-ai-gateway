//! 构建脚本：release 打包或启用 `ui` feature 时，把控制台前端内嵌进二进制。
//!
//! 流程：
//! 1. 在前端工程执行 `pnpm build:desktop` 产出 `dist`；
//! 2. `dist` 中的文本文件 gzip 压缩、其余文件原样复制到 `target/ui`，保持相对路径；
//! 3. 按相对路径为每个文件生成一条 Web 路由，写入 cargo 分配给当前构建单元的 `OUT_DIR`，
//!    由 `src/ui.rs` 引入；路由代码不写 `target/ui`，避免不同 feature 组合与并发构建互相覆盖。
//!
//! 未满足条件时只写一个空的 `generated.rs`，保证 `src/ui.rs` 的 `include!` 始终成立，
//! 也就不必要求本机存在前端构建产物。

use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;

/// 前端工程目录名（项目根目录下）。
const UI_PROJECT_DIR: &str = "lingting-ai-gateway-ui";
/// 前端构建产物目录名。
const DIST_DIR: &str = "dist";
/// 编译目录名（`target` 下），存放内嵌资源与生成的路由代码。
const OUTPUT_DIR: &str = "ui";
/// 生成的路由代码文件名（写入 `OUT_DIR`）。
const GENERATED_FILE: &str = "generated.rs";
/// 触发内嵌前端的 feature 名（对应 `ui`）。
const BUILD_FEATURE: &str = "UI";
/// 入口页面文件名，根路径与它共用同一份字节。
const ENTRY_FILE: &str = "index.html";
/// 按文本 gzip 压缩的扩展名，其余扩展名原样复制。
const TEXT_EXTENSIONS: &[&str] = &[
    "html",
    "htm",
    "css",
    "js",
    "mjs",
    "json",
    "svg",
    "txt",
    "xml",
    "map",
    "webmanifest",
];

/// 生成代码的头部：固定导入。
///
/// 说明用普通注释：本文件由 `include!` 引入到 `src/ui.rs` 模块内，模块内部注释位置不允许 `//!`。
const GENERATED_HEADER: &str = "\
// 由 bin-server/build.rs 生成的内嵌前端路由，请勿手动修改。

use std::sync::Arc;

use anyhow::Result;
use framework_core::MultiStringValue;
use framework_web::{
    AuthRule, WebBody, WebMethod, WebResponse, WebRoute, push_web_api, web_api_get,
};

";

/// 生成代码的尾部：响应组装。
const GENERATED_FOOTER: &str = "\
/// 组装 gzip 文本资源响应。
async fn build(bytes: &'static [u8], content_type: &str) -> Result<WebResponse> {
    let mut headers = MultiStringValue::default();
    headers.set(\"content-encoding\", \"gzip\");
    headers.set_content_type(content_type);
    headers.set_content_length(bytes.len());
    Ok(response(bytes, headers))
}

/// 组装原样返回的二进制资源响应。
async fn build_raw(bytes: &'static [u8], content_type: &str) -> Result<WebResponse> {
    let mut headers = MultiStringValue::default();
    headers.set_content_type(content_type);
    headers.set_content_length(bytes.len());
    Ok(response(bytes, headers))
}

fn response(bytes: &'static [u8], headers: MultiStringValue) -> WebResponse {
    WebResponse {
        status: 200,
        headers,
        body: WebBody::Bytes(bytes.into()),
    }
}
";

/// 不嵌入前端时写入的内容：只留一行说明，保证 `src/ui.rs` 的 `include!` 始终成立。
const EMPTY_GENERATED: &str =
    "// 未嵌入前端：release 打包或启用 ui feature 时才生成内嵌前端路由。\n";

/// 单个内嵌资源。
struct Resource {
    /// 相对 `dist` 的路径，统一使用 `/` 分隔。
    relative: String,
    /// 压缩或复制后的文件绝对路径。
    file: PathBuf,
    /// 响应使用的 `Content-Type`。
    content_type: &'static str,
    /// 是否为 gzip 压缩后的文本资源。
    compressed: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let project_dir = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or("无法推导项目根目录")?
        .to_path_buf();
    let ui_project_dir = project_dir.join(UI_PROJECT_DIR);
    let dist_dir = ui_project_dir.join(DIST_DIR);
    let output_dir = project_dir.join("target").join(OUTPUT_DIR);
    // 生成的路由代码写到 cargo 给当前构建单元分配的 OUT_DIR：不同 feature 组合与并发构建
    // 各有独立目录，不会互相覆盖；前端资源则集中放在 target/ui。
    let generated_dir = PathBuf::from(env::var("OUT_DIR")?);
    watch_changes(&manifest_dir, &ui_project_dir);

    if !should_embed_ui() {
        // 不嵌入前端：只生成空文件，保证 `src/ui.rs` 的 `include!` 始终成立。
        fs::write(generated_dir.join(GENERATED_FILE), EMPTY_GENERATED)?;

        return Ok(());
    }

    build_frontend(&ui_project_dir)?;

    if !dist_dir.is_dir() {
        return Err(format!(
            "未找到前端构建产物：{}；请先在 {} 执行 pnpm build:desktop",
            dist_dir.display(),
            ui_project_dir.display()
        )
        .into());
    }

    let resources = prepare(&dist_dir, &output_dir)?;
    write_generated(&resources, &generated_dir)?;

    Ok(())
}

/// 是否需要内嵌前端：release 打包，或显式启用 `ui` feature。
fn should_embed_ui() -> bool {
    env::var("PROFILE").is_ok_and(|profile| profile == "release")
        || env::var(format!("CARGO_FEATURE_{BUILD_FEATURE}")).is_ok()
}

/// 在前端工程执行桌面端构建。
fn build_frontend(ui_project_dir: &Path) -> Result<(), Box<dyn Error>> {
    if !ui_project_dir.is_dir() {
        return Err(format!("未找到前端工程目录：{}", ui_project_dir.display()).into());
    }

    // Windows 下 pnpm 只有 `.cmd` 与无扩展名的脚本，`Command` 不会补全这些扩展名，需要交给 cmd 执行。
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "pnpm build:desktop"])
            .current_dir(ui_project_dir)
            .status()?
    } else {
        Command::new("pnpm")
            .args(["build:desktop"])
            .current_dir(ui_project_dir)
            .status()?
    };
    if !status.success() {
        return Err(format!("前端构建失败，退出码：{status}").into());
    }

    Ok(())
}

/// 声明需要监视的路径：任一变化都重跑本脚本。
///
/// 一旦声明 `rerun-if-changed`，Cargo 就不再默认监视整个包，因此包内文件必须一并列出；
/// 前端则只监视会进入构建产物的输入：工程配置、`src`、`public`，以及文件名不固定的
/// `.env*` 与 `lri` 下的组件库源码（组件库以源码链接引入，其改动同样影响前端产物）。
fn watch_changes(manifest_dir: &Path, ui_project_dir: &Path) {
    for path in [
        manifest_dir.join("src"),
        manifest_dir.join("build.rs"),
        manifest_dir.join("Cargo.toml"),
        ui_project_dir.join("src"),
        ui_project_dir.join("public"),
        ui_project_dir.join("lri"),
        ui_project_dir.join("index.html"),
        ui_project_dir.join("package.json"),
        ui_project_dir.join("pnpm-lock.yaml"),
        ui_project_dir.join("vite.config.ts"),
    ] {
        watch_path(&path);
    }

    // `.env*` 与 `lri` 下的直接子项文件名不固定，无法写成常量列表，运行时枚举。
    watch_children(ui_project_dir, |name| name.starts_with(".env"));
    watch_children(&ui_project_dir.join("lri"), |_| true);
}

/// 监视目录下满足条件的直接子项。
fn watch_children(dir: &Path, filter: impl Fn(&str) -> bool) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if filter(&name) {
            watch_path(&entry.path());
        }
    }
}

/// 监视存在的路径；不存在的路径只会带来无意义的重跑，直接跳过。
fn watch_path(path: &Path) {
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
/// 压缩或复制全部构建产物，返回内嵌资源列表。
fn prepare(dist_dir: &Path, output_dir: &Path) -> Result<Vec<Resource>, Box<dyn Error>> {
    let mut files = Vec::new();
    collect_files(dist_dir, "", &mut files)?;
    if files.is_empty() {
        return Err(format!("前端构建产物为空：{}", dist_dir.display()).into());
    }

    // 前端重新构建后文件名可能变化，先清空编译目录，避免残留旧资源。
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    let mut resources = Vec::new();
    for (relative, source) in files {
        let bytes = fs::read(&source)?;
        let compressed = is_text(&relative);
        let file = if compressed {
            output_dir.join(format!("{relative}.gz"))
        } else {
            output_dir.join(&relative)
        };
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        if compressed {
            fs::write(&file, gzip(&bytes)?)?;
        } else {
            fs::write(&file, &bytes)?;
        }

        resources.push(Resource {
            content_type: content_type(&relative),
            relative,
            file,
            compressed,
        });
    }

    Ok(resources)
}

/// 收集目录下的全部文件，`relative` 为相对 `dist` 的路径。
fn collect_files(
    dir: &Path,
    prefix: &str,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), Box<dyn Error>> {
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let relative = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), &relative, files)?;
        } else {
            files.push((relative, entry.path()));
        }
    }

    Ok(())
}

/// gzip 压缩；不写入修改时间，保证同一输入产出相同结果。
fn gzip(bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    encoder.finish()
}

/// 生成全部路由代码并写入 `OUT_DIR`。
fn write_generated(resources: &[Resource], generated_dir: &Path) -> Result<(), Box<dyn Error>> {
    let mut code = String::from(GENERATED_HEADER);

    // 入口页面单列一段：根路径与 `/index.html` 共用同一份字节。
    if let Some(entry) = resources
        .iter()
        .find(|resource| resource.relative == ENTRY_FILE)
    {
        push_entry_code(&mut code, entry);
    }
    for resource in resources
        .iter()
        .filter(|resource| resource.relative != ENTRY_FILE)
    {
        push_resource_code(&mut code, resource);
    }

    code.push_str(GENERATED_FOOTER);
    fs::write(generated_dir.join(GENERATED_FILE), code)?;

    Ok(())
}

/// 生成入口页面代码：共用常量、`/index.html` 路由与根路径路由。
fn push_entry_code(code: &mut String, entry: &Resource) {
    code.push_str(&format!(
        "\
/// 入口页面压缩后的字节，根路径 `/` 与 `/index.html` 共用同一份。
const INDEX_HTML: &[u8] = include_bytes!({file});

/// 入口页面
#[web_api_get(path = \"/{relative}\")]
pub async fn ignore_index_html() -> Result<WebResponse> {{
    build(INDEX_HTML, \"{content_type}\").await
}}

/// 根路径：`web_api_get` 不接受空 path，因此手写路由注册；请求 `/` 规范化后为空串，正好命中。
pub fn build_ui_root_route() -> WebRoute {{
    WebRoute {{
        method: WebMethod::Get,
        path: String::new(),
        auth: AuthRule::login(),
        invoke: Arc::new(|| {{
            Box::pin(async {{
                ignore_index_html()
                    .await
                    .unwrap_or_else(|error| WebResponse::from_error(error, None))
            }})
        }}),
    }}
}}

push_web_api!(build_ui_root_route);

",
        relative = entry.relative,
        content_type = entry.content_type,
        file = include_path(&entry.file),
    ));
}

/// 生成单个资源的路由代码。
fn push_resource_code(code: &mut String, resource: &Resource) {
    let build = if resource.compressed {
        "build"
    } else {
        "build_raw"
    };

    code.push_str(&format!(
        "\
/// 资源 {relative}
#[web_api_get(path = \"/{relative}\")]
pub async fn {name}() -> Result<WebResponse> {{
    let bytes = include_bytes!({file});
    {build}(bytes, \"{content_type}\").await
}}

",
        relative = resource.relative,
        name = function_name(&resource.relative),
        content_type = resource.content_type,
        file = include_path(&resource.file),
    ));
}

/// 由相对路径推导路由函数名：非字母数字统一替换为下划线，并加 `ignore_` 前缀避免进入 TS 导出。
fn function_name(relative: &str) -> String {
    let mut name = String::from("ignore_");
    for value in relative.chars() {
        if value.is_ascii_alphanumeric() {
            name.push(value);
        } else {
            name.push('_');
        }
    }
    name
}

/// 生成 `include_bytes!` 使用的路径字面量；统一使用 `/` 分隔，避免反斜杠转义。
fn include_path(path: &Path) -> String {
    format!("r\"{}\"", path.to_string_lossy().replace('\\', "/"))
}

/// 是否为按文本压缩的扩展名。
fn is_text(relative: &str) -> bool {
    extension(relative).is_some_and(|extension| TEXT_EXTENSIONS.contains(&extension))
}

/// 扩展名对应的 `Content-Type`。
fn content_type(relative: &str) -> &'static str {
    match extension(relative).unwrap_or_default() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "webmanifest" => "application/manifest+json; charset=utf-8",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// 取扩展名（不含 `.`）。
fn extension(relative: &str) -> Option<&str> {
    relative.rsplit_once('.').map(|(_, extension)| extension)
}
