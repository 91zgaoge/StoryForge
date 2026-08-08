// v0.33.4 诊断版：临时移除 #![cfg_attr(not(debug_assertions), windows_subsystem
// = "windows")]。 GUI 子系统下 stderr 不可见，而 Windows 启动崩溃的 panic
// 消息（含文件:行号） 正是经由 stderr 输出——v0.33.3 转储分析证实崩溃为非解退
// panic（console 下可直接看到）。 根因修复后恢复该行。

fn main() {
    // 最早诊断：panic hook 覆盖 setup() 之前（tauri builder/build()
    // 窗口创建）的窗口期
    storymoss_lib::install_early_diag();
    storymoss_lib::run();
}
