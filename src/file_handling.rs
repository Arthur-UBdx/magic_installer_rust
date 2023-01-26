use reqwest;
use std::sync::mpsc;
use crate::Config;

// ---- File handling ---- //

#[derive(Debug)]
pub enum DownloadStatus{
    Downloading,
    Finished(Vec<u8>),
}

#[derive(Debug)]
pub struct DownloadInfo{
    pub status: DownloadStatus,
    pub downloaded_size: u64,
    pub total_size: u64,
}

pub async fn download_stream(url: String, tx: mpsc::Sender<DownloadInfo>) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder().build()?;
    let mut resp = client.get(&url).send().await?;
    let mut downloaded_size: u64 = 0;
    let total_size: u64 = resp.content_length().unwrap_or(0);
    let mut downloaded_data = Vec::with_capacity(total_size as usize);

    while let Some(chunk) = resp.chunk().await? {
        downloaded_size += chunk.len() as u64;
        downloaded_data.extend_from_slice(&chunk);
        tx.send(DownloadInfo{
            status: DownloadStatus::Downloading,
            downloaded_size,
            total_size,
        }).unwrap();
    }
    tx.send(DownloadInfo{
            status: DownloadStatus::Finished(downloaded_data),
            downloaded_size: 0,
            total_size: 0,
        }).unwrap();
    Ok(())
}


// pub fn extract_archive(data: Vec<u8>, path: &str) -> Result<(), std::io::Error> {
//     let path = std::path::Path::new(path);
//     let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).unwrap();

//     for i in 0..archive.len() {
//         let mut file = archive.by_index(i).unwrap();
//         let outpath = path.join(file.name());
//         if (*file.name()).ends_with('/') {
//             std::fs::create_dir_all(&outpath).unwrap();
//         } else {
//             std::fs::create_dir_all(outpath.parent().unwrap()).unwrap();
//             let mut outfile = std::fs::File::create(&outpath).unwrap();
//             std::io::copy(&mut file, &mut outfile).unwrap();
//         }
//     }
//     Ok(())
// }

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