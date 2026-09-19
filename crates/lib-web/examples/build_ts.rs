//! 导出 TypeScript SDK：收集路由、类型与枚举元数据，写入 npm ESM 包目录。
//!
//! 元数据由 `web_api_*` / `auto_type` / `auto_enum` 在编译期注册，因此必须启用
//! `ts-export` feature 才有内容可导出：
//!
//! ```text
//! cargo run -p lib-web --example build_ts --features ts-export
//! ```

use std::path::PathBuf;

use framework_proc_core::{
    PackageBuilder, TypescriptApiBuilder, TypescriptEnumBuilder, TypescriptTypeBuilder,
    api_metadata_iter, enum_metadata_iter, type_metadata_iter,
};

/// npm 包名。
const PACKAGE_NAME: &str = "@lingting/ai-gateway-sdk";

/// API 抽象类名，调用方继承后实现 `call` 即可获得全部接口方法。
const API_CLASS_NAME: &str = "BasicAiGatewayApi";

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 强制链接本 crate 的路由注册项，否则元数据会被链接器丢弃。
    let _ = lib_web::web_routes();

    let mut builder =
        PackageBuilder::new(PACKAGE_NAME, env!("CARGO_PKG_VERSION"), sdk_output_dir()?);
    builder.push(
        "enums",
        TypescriptEnumBuilder::new().enums(enum_metadata_iter()),
    );
    builder.push(
        "types",
        TypescriptTypeBuilder::new().types(type_metadata_iter()),
    );
    builder.push(
        "api",
        TypescriptApiBuilder::new()
            .apis(api_metadata_iter())
            .class_name(API_CLASS_NAME),
    );

    builder.write()?;
    Ok(())
}

/// SDK 输出目录：项目根目录下的 `packages/sdk`。
fn sdk_output_dir() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest_dir
        .parent()
        .ok_or_else(|| std::io::Error::other("无法推导 crates 目录"))?;
    let project_dir = crates_dir
        .parent()
        .ok_or_else(|| std::io::Error::other("无法推导项目根目录"))?;
    Ok(project_dir.join("packages").join("sdk"))
}
