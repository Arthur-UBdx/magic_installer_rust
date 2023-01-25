
use native_windows_derive::NwgUi;
use native_windows_gui::NativeUi;
use native_windows_gui as nwg;

use std::io::prelude::*;
use std::fs::File;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;


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

// ---- UI ---- //

// Main window
#[derive(Default, NwgUi)]
pub struct MainWindow{
    #[nwg_control(size: (500, 350), position: (300, 300), title: "Installateur magique", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [MainWindow::close_window()] )]
    window: nwg::Window,
}

impl MainWindow {
    fn close_window() {
        nwg::stop_thread_dispatch();
    }
}

// Download window
#[derive(Default, NwgUi)]
pub struct DownloadWindow {
    #[nwg_control(size: (500, 100), position: (300, 300), title: "Téléchargement", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [DownloadWindow::close_window()] )]
    window: nwg::Window,

    #[nwg_control(text: "Téléchargement en cours...", size: (300, 30), position: (25, 20))]
    #[nwg_layout_item(layout: window_layout)]
    label: nwg::Label,

    //add a progress bar
    #[nwg_control(size: (450, 25), position: (25, 50))]
    #[nwg_layout_item(layout: window_layout)]
    progress_bar: nwg::ProgressBar,
}

impl DownloadWindow {
    fn close_window() {
        nwg::stop_thread_dispatch();
    }
}


// ---- File handling ---- //

#[derive(Debug)]
pub enum DownloadStatus{
    Downloading,
    Finished,
}

#[derive(Debug)]
pub struct DownloadInfo{
    status: DownloadStatus,
    downloaded_size: u64,
    total_size: u64,
}

fn download_file(url: &str) -> Vec<u8> {
    let mut response = reqwest::blocking::get(url).unwrap();
    let mut buffer = Vec::new();
    response.copy_to(&mut buffer).unwrap();
    buffer
}

async fn download_stream(url: String, tx: mpsc::Sender<DownloadInfo>) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut resp = client.get(&url).send().await?;
    let mut downloaded_data = Vec::new();
    let mut downloaded_size: u64 = 0;
    let total_size: u64 = resp.content_length().unwrap_or(0);

    while let Some(chunk) = resp.chunk().await? {
        downloaded_size += chunk.len() as u64;
        downloaded_data.extend_from_slice(&chunk);
        let debug_tx = tx.send(DownloadInfo{
            status: DownloadStatus::Downloading,
            downloaded_size,
            total_size,
        }).unwrap();
    }
    tx.send(DownloadInfo{
            status: DownloadStatus::Finished,
            downloaded_size: 0,
            total_size: 0,
        }
    ).unwrap();
    Ok(downloaded_data)
}

fn save_file(data: Vec<u8>, path: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(&data).unwrap();
}

fn extract_archive(data: Vec<u8>, path: &str) {
    let path = std::path::Path::new(path);
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = path.join(file.name());
        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            std::fs::create_dir_all(outpath.parent().unwrap()).unwrap();
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}

// ---- Main Functions (File Handling + UI) ---- //

pub fn download_mods(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    nwg::init().expect("Failed to init Native Windows GUI :(");
    let download_window = DownloadWindow::build_ui(Default::default()).expect("Failed to build UI");

    let data = tokio::spawn(async move {   
        download_stream(config.mod_files_url, tx).await;
    });

    download_window.progress_bar.set_pos(50);

    loop {
        let download_info: DownloadInfo = rx.recv().unwrap();
        match download_info.status {
            DownloadStatus::Downloading => {
                let percentage_downloaded: u32 = (download_info.downloaded_size as f32 / download_info.total_size as f32 * 100.0) as u32;
                download_window.progress_bar.set_pos(percentage_downloaded);
                println!("{}%, {}/{}", percentage_downloaded, download_info.downloaded_size, download_info.total_size); //debug
            },
            DownloadStatus::Finished => {
                println!("finished"); //debug
                std::thread::sleep(std::time::Duration::from_secs(1));
                break;
            }
        }
    }


    nwg::dispatch_thread_events();
    Ok(())
}

