use std::process::Command;

use anyhow::{Context, Result, anyhow};

/// 执行外部命令；失败时把命令、退出码与完整输出一起报错，便于直接展示给用户。
pub(crate) fn run(program: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("执行 {program} 失败，请确认该命令存在且可用"))?;

    if output.status.success() {
        return Ok(());
    }

    let code = output
        .status
        .code()
        .map_or_else(|| "未知".to_string(), |code| code.to_string());

    Err(anyhow!(
        "{program} 执行失败\n  命令：{} {}\n  退出码：{code}\n  标准输出：{}\n  标准错误：{}",
        program,
        args.join(" "),
        output_text(&output.stdout),
        output_text(&output.stderr),
    ))
}

/// 外部命令输出转文本；空输出用占位符表示，避免错误信息里出现空白。
fn output_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim();
    if text.is_empty() {
        "(空)".to_string()
    } else {
        text.to_string()
    }
}
