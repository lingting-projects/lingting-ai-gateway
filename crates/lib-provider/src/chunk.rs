use std::io;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use futures_util::stream::BoxStream;
use tokio::sync::mpsc;

use crate::forward::ForwardOutcome;

/// 流式分片出口。
///
/// 驱动把已序列化的分片写入这里，调用方消费 [`ChunkSink::channel`] 返回的流；
/// 同时它也是本次转发的累积容器，流中途失败时调用方仍能取到已累积的用量与内容。
#[derive(Clone)]
pub struct ChunkSink {
    sender: mpsc::UnboundedSender<Result<Bytes, io::Error>>,
    outcome: Arc<Mutex<ForwardOutcome>>,
}

impl ChunkSink {
    /// 创建出口与对应的响应流。
    pub fn channel() -> (Self, BoxStream<'static, Result<Bytes, io::Error>>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
            receiver.recv().await.map(|item| (item, receiver))
        });

        let sink = Self {
            sender,
            outcome: Arc::new(Mutex::new(ForwardOutcome::default())),
        };

        (sink, Box::pin(stream))
    }

    /// 写出一个分片；调用方已断开时静默忽略。
    pub fn send(&self, bytes: impl Into<Bytes>) {
        let _ = self.sender.send(Ok(bytes.into()));
    }

    /// 写出一个错误分片。
    pub fn send_error(&self, message: impl Into<String>) {
        let _ = self.sender.send(Err(io::Error::other(message.into())));
    }

    /// 调用方是否已断开。
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    /// 等待调用方断开；可放进 `select!` 与分片读取同时等待。
    pub async fn closed(&self) {
        self.sender.closed().await;
    }

    /// 更新本次转发的累积结果。
    pub fn record(&self, outcome: ForwardOutcome) {
        if let Ok(mut current) = self.outcome.lock() {
            *current = outcome;
        }
    }

    /// 追加累积内容的原始字节。
    pub fn append_content(&self, content: impl AsRef<[u8]>) {
        if let Ok(mut current) = self.outcome.lock() {
            let mut buffer = current.content.take().unwrap_or_default().to_vec();
            buffer.extend_from_slice(content.as_ref());
            current.content = Some(Bytes::from(buffer));
        }
    }

    /// 取当前累积结果。
    pub fn outcome(&self) -> ForwardOutcome {
        self.outcome
            .lock()
            .map(|current| current.clone())
            .unwrap_or_default()
    }
}
