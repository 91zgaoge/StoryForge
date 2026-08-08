// v0.33.5：根因已定位修复（窗口推迟创建，消除 State 未就绪竞态），
// 恢复 GUI 子系统（v0.33.4 诊断期曾临时移除以便在控制台观察 panic）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 最早诊断：panic hook 覆盖 setup() 之前（tauri builder/build()
    // 窗口创建）的窗口期
    storymoss_lib::install_early_diag();
    storymoss_lib::run();
}
