//! 系统信息接口。

use anyhow::Result;
use framework_web::web_api_post;
use service_admin::service::SystemService;
use types_admin::dto::SystemUserVO;

/// 系统账户列表：仅返回在本机存在家目录的账户。
#[web_api_post(path = "/___/system/users")]
pub async fn users() -> Result<Vec<SystemUserVO>> {
    SystemService::users()
}
