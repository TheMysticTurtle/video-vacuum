// Release builds are GUI apps: no console window behind the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    video_vacuum_lib::run()
}
