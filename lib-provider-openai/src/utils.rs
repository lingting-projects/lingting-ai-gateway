//! [OI] 接口使用的工具方法。

use types_admin::entity::Provider;

/// 供应商的对话补全地址。
pub fn chat_url(provider: &Provider) -> String {
    format!(
        "{}/chat/completions",
        provider.base_url.trim_end_matches('/')
    )
}
