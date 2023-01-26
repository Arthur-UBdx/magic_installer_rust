mod file_handling;
mod ui;

use native_windows_gui::NativeUi;
use native_windows_gui as nwg;
use std::sync::mpsc;

use crate::file_handling::{DownloadInfo, DownloadStatus, download_stream};
use crate::ui::{DownloadWindow};

const DEFAULT_MODLOADER_URL: &str = "https://github.com/Arthur-UBdx/fabric_minecraft/zipball/main";
const DEFAULT_MOD_FILES_URL: &str = "https://github.com/Arthur-UBdx/mods_minecraft/zipball/main";
const DEFAULT_MINECRAFT_FOLDER_PATH: &str = "%appdata%\\.minecraft";


#[derive(Debug)]
pub struct Config {
    modloader_url: String,
    mod_files_url: String,
    minecraft_folder_path: String,
}

impl Config {
    pub fn new() -> Config {
        Config{
            modloader_url: String::from(DEFAULT_MODLOADER_URL),
            mod_files_url: String::from(DEFAULT_MOD_FILES_URL),
            minecraft_folder_path: String::from(DEFAULT_MINECRAFT_FOLDER_PATH),
        }
    }

    pub fn default(mut self) {
        self.modloader_url = String::from(DEFAULT_MODLOADER_URL);
        self.mod_files_url = String::from(DEFAULT_MOD_FILES_URL);
        self.minecraft_folder_path = String::from(DEFAULT_MINECRAFT_FOLDER_PATH);
    }

    pub fn from(args: (String, String, String)) -> Config {
        Config{
            modloader_url: args.0,
            mod_files_url: args.1,
            minecraft_folder_path: args.2,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Main Functions (File Handling + UI) ---- //

pub fn download_mods(config: Config) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    nwg::init().expect("Failed to init Native Windows GUI :(");
    let download_window = DownloadWindow::build_ui(Default::default()).expect("Failed to build UI");

    tokio::spawn(async move {   
        download_stream(config.mod_files_url, tx).await.unwrap();
    });

    let mut data: Vec<u8> = Vec::new();
    loop {
        let download_info: DownloadInfo = rx.recv().unwrap();
        match download_info.status {
            DownloadStatus::Downloading => {
                let percentage_downloaded: u32 = (download_info.downloaded_size as f32 / download_info.total_size as f32 * 100.0) as u32;
                download_window.progress_bar.set_pos(percentage_downloaded);
            },
            DownloadStatus::Finished(d) => {
                data = d;
                break;
            }
        }
    }
    nwg::dispatch_thread_events();
    Ok(data)
}

