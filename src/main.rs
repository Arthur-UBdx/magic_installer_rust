mod file_handling;

use magic_installer::{download_mods, Config};

#[tokio::main]
async fn main() -> ! {
    // let config: Config = Config::from(String::from("https://github.com/Arthur-UBdx/fabric_minecraft/zipball/main"),String::new());
    let config: Config = Config::new();
    // let _data: Vec<u8> = match download_mods(config) {
    //     Ok(data) => data,
    //     Err(_) => panic!("Erreur lors du téléchargement des mods"),
    // };

    let path = "%appdata%\\.minecraft";
    let path_var = crate::file_handling::get_env_path(path);
    println!("{}", path_var);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

