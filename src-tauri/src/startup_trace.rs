//! 启动诊断面包屑（v0.33.3 诊断版）
//!
//! 背景：Windows 版启动几秒后进程消失，WER 记录 c0000409 / P9=7
//! （CRT abort / __fastfail FAST_FAIL_FATAL_APP_EXIT），不产生 panic 日志、
//! 不留任何现场。tracing 的 non-blocking writer 在崩溃前来不及 flush，
//! 启动早期日志全部丢失。
//!
//! 本模块在每个启动里程碑直接向文件追加一行并立即 flush（绕过 tracing），
//! 崩溃后最后一行即崩溃点前最后到达的位置。文件写入用户临时目录：
//! Windows: `%TEMP%\storymoss-startup-trace.log`
//! macOS:   `$TMPDIR/storymoss-startup-trace.log`
//! Linux:   `/tmp/storymoss-startup-trace.log`

use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

/// 单文件大小上限：超过则在下次进程启动时截断重写（append 会无限增长）
const TRACE_FILE_MAX_SIZE: u64 = 512 * 1024; // 512KB

static FIRST_TRACE_OF_PROCESS: AtomicBool = AtomicBool::new(true);

fn trace_path() -> PathBuf {
    std::env::temp_dir().join("storymoss-startup-trace.log")
}

/// 双写兜底目标：Windows 上除 %TEMP% 外同时写 C:\Users\Public。
/// v0.33.3 复现时 %TEMP% 无文件——排除 temp_dir 解析/用户上下文差异的误判，
/// Public 目录一定可写且位置固定。
fn public_trace_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let dir = PathBuf::from(r"C:\Users\Public");
        if dir.is_dir() {
            return Some(dir.join("storymoss-startup-trace.log"));
        }
    }
    None
}

/// 追加一行里程碑记录（时间戳 + pid + 描述），立即 flush。
/// 同时写 stderr——用户用 `storymoss.exe 2> file` 重定向后可捕获
/// Rust 运行时的栈溢出/分配失败消息（这些消息不走 panic hook）。
pub fn trace(milestone: &str) {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("{} [pid {}] {}\n", ts, std::process::id(), milestone);

    let path = trace_path();
    // 进程首次记录时若文件超限则截断，保留诊断价值同时防止无限增长
    let truncate = FIRST_TRACE_OF_PROCESS.swap(false, Ordering::SeqCst)
        && std::fs::metadata(&path)
            .map(|m| m.len() > TRACE_FILE_MAX_SIZE)
            .unwrap_or(false);

    let open_result = if truncate {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
    } else {
        OpenOptions::new().create(true).append(true).open(&path)
    };
    if let Ok(mut f) = open_result {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
    if let Some(public) = public_trace_path() {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(public) {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }
    eprint!("{}", line);
}

/// v0.33.4 诊断：main() 入口第一行安装的超早期 panic hook。
/// 覆盖 setup() 的 install_panic_hook 之前的窗口期（tauri builder 组装、
/// build() 中窗口/WebView2 创建）——v0.33.3 证实崩溃无任何面包屑，说明崩溃
/// 在该窗口或更早；配合控制台子系统，panic 消息（含文件:行号）直接打在终端。
/// setup() 的 install_panic_hook 生效后会覆盖此 hook，属预期交接。
pub fn install_early_diag() {
    std::panic::set_hook(Box::new(|info| {
        let report = crate::logging::format_panic_report(info);
        trace(&format!(
            "EARLY PANIC: {}",
            report.lines().next().unwrap_or("APPLICATION PANIC")
        ));
        eprintln!("{}", report);
    }));
    trace("early diag hook installed (main entry)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_writes_line_to_file() {
        trace("test milestone");
        let content = std::fs::read_to_string(trace_path()).expect("trace file should exist");
        assert!(content.contains("test milestone"), "content: {}", content);
    }
}
