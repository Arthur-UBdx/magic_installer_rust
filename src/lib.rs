mod file_handling;
mod ui;
mod test;

use native_windows_gui::NativeUi;
use native_windows_gui as nwg;
use std::sync::mpsc;
use winapi::um::winuser::{PeekMessageA, PM_REMOVE};

use crate::file_handling::{DownloadStatus, download_stream, remove_mods, get_env_path};
use crate::ui::{DownloadWindow};

const DEFAULT_MOD_FILES_URL: &str = "http://serveur-arthur.tk/mod_files/mod_files.zip";
const DEFAULT_MINECRAFT_FOLDER_PATH: &str = "%appdata%\\.minecraft";
const DEFAULT_MODIFIED_FOLDERS: &[&str; 2] = &["mods", "config"];


#[derive(Debug)]
pub struct Config<> {
    pub mod_files_url: String,
    pub minecraft_folder_path: String,
    pub modified_folders: Vec<String>,
}

impl Config {
    pub fn new() -> Config {
        Config{
            mod_files_url: String::from(DEFAULT_MOD_FILES_URL),
            minecraft_folder_path: String::from(DEFAULT_MINECRAFT_FOLDER_PATH),
            modified_folders: DEFAULT_MODIFIED_FOLDERS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn default(mut self) {
        self.mod_files_url = String::from(DEFAULT_MOD_FILES_URL);
        self.minecraft_folder_path = String::from(DEFAULT_MINECRAFT_FOLDER_PATH);
        self.modified_folders = DEFAULT_MODIFIED_FOLDERS.iter().map(|s| s.to_string()).collect();
    }

    pub fn from(args: (String, String, Vec<String>)) -> Config {
        Config{
            mod_files_url: args.0,
            minecraft_folder_path: args.1,
            modified_folders: args.2,
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
        match download_stream(config.mod_files_url, tx).await {
            Ok(_) => (),
            Err(e) => panic!("{}",e)
        };
    });

    // let (data_tx, data_rx) = mpsc::channel();
    let mut data: Vec<u8> = Vec::new();

    loop {
        match rx.try_recv() {
            Ok(d) => {
                match d.status {
                    DownloadStatus::Downloading => {
                        let percentage_downloaded: u32 = (d.downloaded_size as f32 / d.total_size as f32 * 100.0) as u32;
                        download_window.progress_bar.set_pos(percentage_downloaded);
                        println!("{}%", percentage_downloaded)
                    },
                    DownloadStatus::Finished(d) => {
                        data = d;
                        break;
                    },
                }
            },

            Err(std::sync::mpsc::TryRecvError::Empty) => {
                unsafe { //unsafe code I don't understand
                    PeekMessageA(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0, PM_REMOVE); //some shit to make the UI work <- (this was written by copilot, even he doesn't understand it)
                };
            },
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                break;
            },
        }
    }
    
    std::thread::sleep(std::time::Duration::from_secs(2));
    nwg::modal_message(&download_window.window,&nwg::MessageParams{title: "Téléchargement Terminé", content: "Téléchargement Terminé", buttons: nwg::MessageButtons::Ok, icons: nwg::MessageIcons::Info});
    DownloadWindow::close();
    Ok(data)
}