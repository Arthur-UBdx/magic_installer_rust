mod file_handling;
mod test;
mod ui;

use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use std::sync::mpsc;
use winapi::um::winuser::{PeekMessageA, PM_REMOVE};

use crate::file_handling::{download_stream, get_env_path, DownloadStatus};
use crate::ui::DownloadWindow;

const DEFAULT_MOD_FILES_URL: &str = "http://serveur-arthur.tk/mod_files/mod_files.zip";
const DEFAULT_MINECRAFT_FOLDER_PATH: &str = "%appdata%\\.minecraft";
const DEFAULT_MODIFIED_FOLDERS: &[&str; 2] = &["mods", "config"];

#[derive(Debug)]
pub struct Config {
    pub mod_files_url: String,
    pub minecraft_folder_path: String,
    pub modified_folders: Vec<String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            mod_files_url: String::from(DEFAULT_MOD_FILES_URL),
            minecraft_folder_path: String::from(DEFAULT_MINECRAFT_FOLDER_PATH),
            modified_folders: DEFAULT_MODIFIED_FOLDERS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn default(mut self) {
        self.mod_files_url = String::from(DEFAULT_MOD_FILES_URL);
        self.minecraft_folder_path = String::from(DEFAULT_MINECRAFT_FOLDER_PATH);
        self.modified_folders = DEFAULT_MODIFIED_FOLDERS
            .iter()
            .map(|s| s.to_string())
            .collect();
    }

    pub fn from(args: (String, String, Vec<String>)) -> Config {
        Config {
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

pub fn download_files(config: Config) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    nwg::init().expect("Failed to init Native Windows GUI :(");
    let download_window = DownloadWindow::build_ui(Default::default()).expect("Failed to build UI");

    tokio::spawn(async move {
        match download_stream(config.mod_files_url, tx).await {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
    });

    let mut data: Vec<u8> = Vec::new();
    loop {
        match rx.try_recv() {
            Ok(d) => match d.status {
                DownloadStatus::Downloading => {
                    let percentage_downloaded: u32 =
                        (d.downloaded_size as f32 / d.total_size as f32 * 100.0) as u32;
                    download_window.progress_bar.set_pos(percentage_downloaded);
                    println!("{}%", percentage_downloaded)
                }
                DownloadStatus::Finished(d) => {
                    data = d;
                    break;
                }
            },

            Err(std::sync::mpsc::TryRecvError::Empty) => {
                unsafe {
                    //unsafe code I don't understand
                    PeekMessageA(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0, PM_REMOVE);
                    //some shit to make the UI work <- this was written by copilot, even he doesn't understand it
                };
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                break;
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
    nwg::modal_message(
        &download_window.window,
        &nwg::MessageParams {
            title: "Téléchargement Terminé",
            content: "Téléchargement Terminé",
            buttons: nwg::MessageButtons::Ok,
            icons: nwg::MessageIcons::Info,
        },
    );
    DownloadWindow::close();
    Ok(data)
}


// ---- UI ---- //

// Main window
#[derive(Default, NwgUi)]
pub struct MainWindow {
    #[nwg_control(size: (500, 300), position: (300, 300), title: "Installateur magique", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [MainWindow::close()])]
    pub window: nwg::Window,

    #[nwg_resource(family: "Bahnschrift", size: 28, weight: 400)]
    pub font_title: nwg::Font,

    #[nwg_resource(family: "MS Shell Dlg 2", size: 16, weight: 400)]
    pub font_default: nwg::Font,

    #[nwg_control(text: "Installateur magique", size: (300, 30), position: (25, 10), font: Some(&data.font_title))]
    #[nwg_layout_item(layout: window_layout)]
    pub title_text: nwg::Label,

    #[nwg_control(size: (360, 200), position: (25, 50))]
    // #[nwg_events(OnInit: [MainWindow::init_image_frame(&data.image_frame)])]
    pub image_frame: nwg::ImageFrame,

    #[nwg_control(text: "Installer les mods", size: (225, 25), position: (25, 260), font: Some(&data.font_default))]
    #[nwg_events( OnButtonClick: [MainWindow::install_files()] )]
    pub install_button: nwg::Button,

    #[nwg_control(text: "Supprimer l'installation les mods", size: (225, 25), position: (300, 260), font: Some(&data.font_default))]
    #[nwg_events( OnButtonClick: [MainWindow::remove_mods()] )]
    pub remove_button: nwg::Button,
}

impl MainWindow {
    fn close() {
        nwg::stop_thread_dispatch();
    }

    pub fn init_image_frame(image_frame: &nwg::ImageFrame) {
        let image =
            nwg::Bitmap::from_file("src/assets/kanye.bmp", true).expect("Failed to load image");
        let kanye = Some(&image);
        image_frame.set_bitmap(kanye);
    }

    fn install_files() {
        let config: Config = Config::new();
        let dl_handle = std::thread::spawn(|| async move {
            let minecraft_path = get_env_path(&config.minecraft_folder_path);
            let data: Vec<u8> = download_files(config).unwrap();
            extract_archive(data, &minecraft_path);
        });
    }

    fn remove_mods() {
        let config: Config = Config::new();
        let rm_handle = std::thread::spawn(|| async move {
            let minecraft_path = get_env_path(&config.minecraft_folder_path);
            remove_mods(config);
        });
    }
}

// Download window
#[derive(Default, NwgUi)]
pub struct DownloadWindow {
    #[nwg_control(size: (500, 100), position: (300, 300), title: "Téléchargement", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [])]
    pub window: nwg::Window,

    #[nwg_control(text: "Téléchargement en cours...", size: (300, 30), position: (25, 20))]
    #[nwg_layout_item(layout: window_layout)]
    pub label: nwg::Label,

    //add a progress bar
    #[nwg_control(size: (450, 25), position: (25, 50))]
    #[nwg_layout_item(layout: window_layout)]
    pub progress_bar: nwg::ProgressBar,
}

impl DownloadWindow {
    pub fn close() {
        nwg::stop_thread_dispatch();
    }
}

// ---- File handling ---- //

#[derive(Debug)]
pub enum DownloadStatus {
    Downloading,
    Finished(Vec<u8>),
}

#[derive(Debug)]
pub struct DownloadInfo {
    pub status: DownloadStatus,
    pub downloaded_size: u64,
    pub total_size: u64,
}

pub async fn download_stream(
    url: String,
    tx: mpsc::Sender<DownloadInfo>,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder().build()?;
    let mut resp = client.get(&url).send().await?;
    let mut downloaded_size: u64 = 0;
    let total_size: u64 = resp.content_length().unwrap_or(0);
    let mut downloaded_data = Vec::with_capacity(total_size as usize);

    while let Some(chunk) = resp.chunk().await? {
        downloaded_size += chunk.len() as u64;
        downloaded_data.extend_from_slice(&chunk);
        tx.send(DownloadInfo {
            status: DownloadStatus::Downloading,
            downloaded_size,
            total_size,
        })
        .unwrap();
    }
    tx.send(DownloadInfo {
        status: DownloadStatus::Finished(downloaded_data),
        downloaded_size: 0,
        total_size: 0,
    })
    .unwrap();
    Ok(())
}

pub fn extract_archive(data: Vec<u8>, path: &str) -> Result<(), std::io::Error> {
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
    Ok(())
}

pub fn remove_mods(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let minecraft_path: &str = &config.minecraft_folder_path;
    for folder in config.modified_folders {
        let path = format!("{}\\{}", minecraft_path, folder);
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn get_env_path(path: &str) -> String {
    if path.starts_with("%") {
        let path_splitted: Vec<&str> = path.split("%").collect();
        let var: &str = &path_splitted[1];
        let path = match std::env::var(var.to_uppercase()) {
            Ok(path) => path,
            Err(_) => panic!("Variable d'environnement '{}' non trouvée", var),
        };
        return path.to_string() + &path_splitted[2].to_string();
    }
    path.to_string()
}
