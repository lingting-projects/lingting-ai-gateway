//! 基于 framework-web 的项目级 Web 适配层。
//!
//! 提供统一鉴权、请求身份上下文与应用全局配置上下文；
//! 业务接口只需声明 `AuthRule`，鉴权由路由器统一完成。

pub mod auth;
pub mod context;

pub use auth::{
    AuthKind, AuthRuleExt, Authorization, AuthorizationService, ROLE_ADMIN, ROLE_API, allows,
    scope_authorization, sha1_hex, use_authorization,
};
pub use context::{
    AppContext, CONFIG_ADMIN_TOKEN, CONFIG_ALLOW_ANONYMOUS, CONFIG_DEBUG_MODE, scope_app, use_app,
};
pub use framework_web::{
    AuthRule, Json, Query, WebBody, WebContext, WebError, WebErrorExt, WebErrorKind, WebMethod,
    WebRequest, WebResponse, WebRoute, catch_panic, scope_web, use_web,
};
