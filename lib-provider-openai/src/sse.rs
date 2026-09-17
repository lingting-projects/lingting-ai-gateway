/// SSE 分片解码器：按字节缓冲，逐个吐出完整数据行。
///
/// 网络分片不保证按行或按 UTF-8 边界切分，因此先按 `\n` 切字节，
/// 再对完整行做 UTF-8 解码。
#[derive(Default)]
pub struct SseDecoder {
    buffer: Vec<u8>,
}

impl SseDecoder {
    /// 创建解码器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一段字节，返回其中已完整的数据行载荷（已去掉 `data:` 前缀）。
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(chunk);

        let mut payloads = Vec::new();
        while let Some(index) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=index).collect();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim();
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                payloads.push(data.trim().to_string());
            }
        }

        payloads
    }
}
