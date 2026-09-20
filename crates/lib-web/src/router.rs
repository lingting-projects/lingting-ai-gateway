use anyhow::Result;
use framework_web::{WebError, WebResponse, WebRoute, use_web, web_api_iter};
use framework_web_axum::WebRouteWrapper;
use lib_db::{DbContext, scope_db};
use lib_web_core::scope_authorization;
use service_admin::manager::WebManager;
use std::sync::Arc;

/// 构造 Web 路由包装器：framework-web-axum 命中路由后，由本包装器建立请求级数据库上下文，
/// 完成授权解析与授权注入，再执行路由。
pub fn web_route_wrapper(db: Arc<DbContext>) -> WebRouteWrapper {
    Arc::new(move |route| {
        let db = Arc::clone(&db);
        Box::pin(async move { invoke(db, route).await })
    })
}

/// 收集当前已注册的 Web 路由，供 TS 导出与 Cloudflare 规则同步等外部工具使用。
pub fn web_routes() -> Vec<WebRoute> {
    web_api_iter().collect()
}

async fn invoke(db: Arc<DbContext>, route: Arc<WebRoute>) -> Result<WebResponse> {
    scope_db(db, async move {
        let manager = WebManager::new()?;
        let app = manager.build_app().await?;

        let token = bearer_token()?;
        let authorization = manager.build_authorization(&token).await?;

        // 管理接口；路由 path 由框架规范化，不带前导斜杠。
        if route.path.starts_with("___/") {
            if !authorization.is_admin() {
                return Err(WebError::forbidden("当前用户没有此接口访问权限").into());
            }
        // ai 接口
        } else {
            if !app.allow_anonymous() && !authorization.is_api() {
                return Err(WebError::forbidden("当前用户没有此接口访问权限").into());
            }
        }

        scope_authorization(authorization, async move { Ok(route.invoke().await) }).await
    })
    .await
}

/// 读取 Bearer 令牌；未携带 Authorization 或格式异常时返回空串，由授权解析决定身份。
fn bearer_token() -> Result<String> {
    let context = use_web()?;
    let Some(value) = context.request().headers.get_first("authorization") else {
        return Ok(String::new());
    };
    let Some((scheme, token)) = value.split_once(' ') else {
        return Ok(String::new());
    };
    let token = token.trim();
    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() {
        return Ok(String::new());
    }
    Ok(token.to_string())
}
