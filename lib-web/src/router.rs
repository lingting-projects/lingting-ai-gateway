use framework_web::{web_api_iter, WebContext, WebRoute, WebError, WebResponse, scope_web, use_web};
use std::sync::{Arc, LazyLock};
use lib_db::{scope_db, DbContext};
use anyhow::Result;

/// Web 路由器
pub struct WebRouter {
    routes: Vec<Arc<WebRoute>>,
}

impl WebRouter {
    pub fn new(routes: Vec<Arc<WebRoute>>) -> Self {
        Self { routes }
    }

    /// 全部已注册路由
    pub fn routes(&self) -> &[Arc<WebRoute>] {
        &self.routes
    }

    pub async fn invoke(
        &self,
        web_context: WebContext,
        db_context: DbContext,
    ) -> Result<WebResponse> {
        let web_context = Arc::new(web_context);
        let db_context = Arc::new(db_context);
        scope_db(
            db_context,
            scope_web(web_context, async {
                let route = self.find().await?;
                Ok(route.invoke().await)
            }),
        )
            .await
    }

    async fn find(&self) -> Result<&WebRoute> {
        let context = use_web()?;
        let request = context.request();
        let path = request.path.trim_matches('/');
        let route = self
            .routes
            .get(&request.method)
            .and_then(|routes| routes.get(path))
            .ok_or_else(|| WebError::not_found("请求接口不存在"))?;

        let rule = &route.auth;
        if rule.anonymous == Some(true) {
            return Ok(route);
        }

        let token = bearer_token(&context)?;
        let authorization = UserAuthorizationService::new()?
            .authorization(token)
            .await?;

        if !allows(rule, &authorization) {
            return Err(WebError::forbidden("当前用户没有此接口访问权限").into());
        }
        set_authorization(authorization)?;
        Ok(route)
    }
}

/// 全局路由表，由 web_api* 宏自动注册
static ROUTER: LazyLock<WebRouter> = LazyLock::new(|| {
    let routes = web_api_iter().map(Arc::new).collect();
    WebRouter::new(routes)
});


pub fn router() -> &'static WebRouter {
    &ROUTER
}

fn bearer_token(context: &WebContext) -> Result<&str> {
    let value = context
        .request()
        .headers
        .get_first("authorization")
        .ok_or_else(|| WebError::unauthorized("缺少 Authorization 请求头"))?;
    let (scheme, token) = value
        .split_once(' ')
        .ok_or_else(|| WebError::unauthorized("Authorization 请求头格式错误"))?;
    let token = token.trim();
    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() {
        return Err(WebError::unauthorized("Authorization 请求头格式错误").into());
    }
    Ok(token)
}
