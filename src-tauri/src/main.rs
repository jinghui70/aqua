// Release 模式下 Windows 不弹控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    aqua::run()
}
