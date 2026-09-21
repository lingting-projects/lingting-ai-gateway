//! Web 路由包装器：请求线程完成鉴权，AI 接口转入工作线程以便感知客户端取消。

use std::sync::Arc;

use anyhow::{Result, anyhow};
use framework_web::{
    WebError, WebResponse, WebRoute, catch_panic, scope_web, use_web, web_api_iter,
};
use framework_web_axum::{WebRouteWrapper, scope_axum, use_axum};
use lib_db::{DbContext, scope_db};
use lib_web_core::{
    AppContext, Authorization, WebCancelGuard, WebCancelToken, scope_app, scope_authorization,
    scope_web_cancel,
};
use service_admin::manager::WebManager;
use tokio::sync::oneshot;

/// 管理接口的路径前缀；这类接口耗时短，直接在请求线程执行。
const ADMIN_PREFIX: &str = "___/";

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

/// 请求入口：先鉴权，再按链路类型分派。
///
/// 鉴权放在请求线程，鉴权失败不必付出线程切换的代价；AI 接口转入工作线程，
/// 使客户端断开时请求线程的 future 被 drop，从而能通知工作线程取消。
async fn invoke(db: Arc<DbContext>, route: Arc<WebRoute>) -> Result<WebResponse> {
    let path = route.path.clone();
    let web_context = use_web()?;
    let axum_context = use_axum()?;

    #[cfg(debug_assertions)]
    tracing::debug!("[MOCKTEST]  request-in path={path}");

    let (authorization, app) = scope_db(Arc::clone(&db), authorize(&route)).await?;

    // 管理接口：直接在请求线程执行，无需取消令牌。
    if route.path.starts_with(ADMIN_PREFIX) {
        let response = scope_db(
            db,
            scope_app(
                Arc::new(app),
                scope_authorization(authorization, async move {
                    catch_panic(async move { route.invoke().await }).await
                }),
            ),
        )
        .await;

        #[cfg(debug_assertions)]
        tracing::debug!("[MOCKTEST]  request-out path={path}");
        return response;
    }

    // AI 接口：工作线程继承不到请求线程的 task_local，需逐个重新安装。
    let token = WebCancelToken::new();
    let (result_tx, result_rx) = oneshot::channel();

    tokio::spawn(scope_axum(
        axum_context,
        scope_web(
            web_context,
            scope_db(
                db,
                scope_web_cancel(
                    Arc::clone(&token),
                    scope_app(
                        Arc::new(app),
                        scope_authorization(authorization, async move {
                            let result = catch_panic(async move { route.invoke().await }).await;
                            let _ = result_tx.send(result);
                        }),
                    ),
                ),
            ),
        ),
    ));

    // 请求线程：等待结果；若客户端在此前断开，本 future 被 drop，守卫触发取消。
    let mut guard = RequestCancelGuard::new(token, &path);
    match result_rx.await {
        Ok(result) => {
            guard.disarm();
            #[cfg(debug_assertions)]
            tracing::debug!("[MOCKTEST]  request-out path={path}");
            result
        }
        Err(_) => Err(anyhow!("客户端已断开")),
    }
}

/// 请求线程守卫：客户端断开（本 future 被 drop）时记录日志并触发取消。
struct RequestCancelGuard {
    cancel: WebCancelGuard,
    path: String,
    armed: bool,
}

impl RequestCancelGuard {
    fn new(token: Arc<WebCancelToken>, path: &str) -> Self {
        Self {
            cancel: WebCancelGuard::new(token),
            path: path.to_string(),
            armed: true,
        }
    }

    /// 正常返回后解除，避免误报与误发取消。
    fn disarm(&mut self) {
        self.armed = false;
        self.cancel.disarm();
    }
}

impl Drop for RequestCancelGuard {
    fn drop(&mut self) {
        if self.armed {
            #[cfg(debug_assertions)]
            tracing::debug!("[MOCKTEST]  request-cancel path={}", self.path);
        }
    }
}

/// 解析请求身份并校验接口访问权限；同时返回应用上下文，供后续作用域注入。
async fn authorize(route: &WebRoute) -> Result<(Authorization, AppContext)> {
    let manager = WebManager::new()?;
    let app = manager.build_app().await?;

    let token = bearer_token()?;
    let authorization = manager.build_authorization(&token).await?;

    if route.path.starts_with(ADMIN_PREFIX) {
        if !authorization.is_admin() {
            return Err(WebError::forbidden("当前用户没有此接口访问权限").into());
        }
    } else if !app.allow_anonymous() && !authorization.is_api() {
        return Err(WebError::forbidden("当前用户没有此接口访问权限").into());
    }

    Ok((authorization, app))
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
