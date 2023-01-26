use reqwest;
use std::sync::mpsc;


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

// fn download_file(url: &str) -> Vec<u8> {
//     let mut response = reqwest::blocking::get(url).unwrap();
//     let mut buffer = Vec::new();
//     response.copy_to(&mut buffer).unwrap();
//     buffer
// }

pub async fn download_stream(url: String, tx: mpsc::Sender<DownloadInfo>) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut resp = client.get(&url).send().await?;
    let mut downloaded_size: u64 = 0;
    let total_size: u64 = resp.content_length().unwrap_or(0);
    let mut downloaded_data = Vec::with_capacity(total_size as usize);

    while let Some(chunk) = resp.chunk().await? {
        downloaded_size += chunk.len() as u64;
        downloaded_data.extend_from_slice(&chunk);
        println!("{:?}",chunk); // debug
        tx.send(DownloadInfo{
            status: DownloadStatus::Downloading,
            downloaded_size,
            total_size,
        });
    }
    tx.send(DownloadInfo{
            status: DownloadStatus::Finished(downloaded_data),
            downloaded_size: 0,
            total_size: 0,
        }
    ).unwrap();
    Ok(())
}

// fn save_file(data: Vec<u8>, path: &str) -> Result<(), std::io::Error> {
//     let mut file = File::create(path).unwrap();
//     file.write_all(&data).unwrap();
//     Ok(())
// }

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