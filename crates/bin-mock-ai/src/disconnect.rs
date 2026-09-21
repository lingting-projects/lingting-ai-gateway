//! 客户端断开检测：等待期或响应流被提前 drop，即说明对端已断开。

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::Stream;

use crate::note;

/// 等待期间被 drop 即说明客户端断开。
pub(crate) struct DisconnectGuard {
    label: &'static str,
    armed: bool,
}

impl DisconnectGuard {
    /// 以阶段名创建守卫。
    pub(crate) fn new(label: &'static str) -> Self {
        Self { label, armed: true }
    }

    /// 正常走完流程后解除，避免误报。
    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for DisconnectGuard {
    fn drop(&mut self) {
        if self.armed {
            note(&format!(
                "★★★ 上游检测到客户端断开：{}期间 handler 被 drop",
                self.label
            ));
        }
    }
}

/// 响应流包装：未被读完就 drop 即说明客户端断开。
///
/// 内部流装箱后固定，使本类型自身始终 `Unpin`，便于交给 axum 的响应体。
pub(crate) struct DisconnectStream<S> {
    inner: Pin<Box<S>>,
    completed: bool,
}

impl<S> DisconnectStream<S> {
    pub(crate) fn new(inner: S) -> Self {
        Self {
            inner: Box::pin(inner),
            completed: false,
        }
    }
}

impl<S> Stream for DisconnectStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(None) => {
                self.completed = true;
                note("响应流正常结束");
                Poll::Ready(None)
            }
            other => other,
        }
    }
}

impl<S> Drop for DisconnectStream<S> {
    fn drop(&mut self) {
        if !self.completed {
            note("★★★ 上游检测到客户端断开：响应流被提前 drop");
        }
    }
}
