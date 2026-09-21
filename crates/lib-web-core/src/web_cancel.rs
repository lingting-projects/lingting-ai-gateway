//! 请求级取消信号：客户端断开时由请求线程触发，供转发链路内部消费。
//!
//! 请求线程等待工作线程结果，若客户端在此前断开，请求线程的 future 会被运行时 drop，
//! [`WebCancelGuard`] 随之触发取消；工作线程通过 [`use_web_cancel`] 取到令牌，
//! 在 `select!` 中感知取消并及时收尾（取消上游请求、写回请求日志）。

use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio::sync::watch;

/// 请求取消令牌。
///
/// 用 `watch` 而非 `Notify`：`wait_for` 在取消已发生时会立即返回，
/// 不存在「先取消后等待」的竞态。
#[derive(Debug)]
pub struct WebCancelToken {
    sender: watch::Sender<bool>,
}

impl WebCancelToken {
    /// 创建令牌，初始为未取消。
    pub fn new() -> Arc<Self> {
        let (sender, _) = watch::channel(false);
        Arc::new(Self { sender })
    }

    /// 触发取消；重复调用无副作用。
    pub fn cancel(&self) {
        let _ = self.sender.send(true);
    }

    /// 是否已取消。
    pub fn is_cancelled(&self) -> bool {
        *self.sender.borrow()
    }

    /// 订阅取消状态，供 `select!` 等待。
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.sender.subscribe()
    }

    /// 等待取消信号；已取消时立即返回。
    ///
    /// 用 `borrow_and_update` + `changed` 而非 `wait_for`：后者返回的 `Ref` 读锁守卫不是
    /// `Send`，放进 `select!` 会让整个 future 无法跨线程，进而无法 `tokio::spawn`。
    pub async fn wait_cancelled(receiver: &mut watch::Receiver<bool>) {
        loop {
            if *receiver.borrow_and_update() {
                return;
            }
            if receiver.changed().await.is_err() {
                // 发送端已释放，视为不再会有取消信号。
                return;
            }
        }
    }
}

/// 取消守卫：请求线程持有，被 drop 时触发取消；正常拿到结果后调用 [`disarm`] 解除。
///
/// [`disarm`]: WebCancelGuard::disarm
pub struct WebCancelGuard {
    token: Arc<WebCancelToken>,
    armed: bool,
}

impl WebCancelGuard {
    /// 绑定令牌创建守卫。
    pub fn new(token: Arc<WebCancelToken>) -> Self {
        Self { token, armed: true }
    }

    /// 解除守卫，避免请求正常完成后误发取消。
    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for WebCancelGuard {
    fn drop(&mut self) {
        if self.armed {
            self.token.cancel();
        }
    }
}

tokio::task_local! {
    static WEB_CANCEL: Arc<WebCancelToken>;
}

/// 在取消令牌作用域内执行。
pub async fn scope_web_cancel<F>(token: Arc<WebCancelToken>, future: F) -> F::Output
where
    F: Future,
{
    WEB_CANCEL.scope(token, future).await
}

/// 取当前请求的取消令牌。
pub fn use_web_cancel() -> Result<Arc<WebCancelToken>> {
    WEB_CANCEL
        .try_with(Arc::clone)
        .map_err(|error| anyhow!("当前调用不在请求取消上下文作用域内：{error}"))
}
