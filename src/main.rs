use native_windows_gui::NativeUi;
use native_windows_gui as nwg;
use native_windows_derive as nwd;

use magic_installer::{download_mods, Config};
use std::sync::mpsc;
use std::thread;
// use std::thread::sleep;
// use std::time::Duration;

#[tokio::main]
async fn main() -> ! {
    let config: Config = Config::from((String::new(),String::from("https://github.com/Arthur-UBdx/fabric_minecraft/zipball/main"),String::new()));
    // let config: Config = Config::new();
    download_mods(config);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

