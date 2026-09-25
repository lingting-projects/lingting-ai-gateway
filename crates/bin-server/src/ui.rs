//! 内嵌控制台前端资源。
//!
//! `build.rs` 仅在 release 打包或启用 `ui` feature 时，把资源与路由生成出来：前端文件写入
//! 编译目录 `target/ui`（文本文件为 gzip 结果，其余原样，每个文件对应一条 Web 路由），
//! 路由代码写入当前构建单元的 `OUT_DIR`；其他构建只生成空的路由代码。

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
