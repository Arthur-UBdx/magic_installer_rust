mod file_handling;
mod ui;

use native_windows_derive as nwd;
use native_windows_gui as nwg;
use nwg::NativeUi;

const IMAGE: &str = "src/assets/kanye.bmp";

#[tokio::main]
async fn main() {
    nwg::init().expect("Failed to init Native Windows GUI :(");
    let main_window = ui::MainWindow::build_ui(Default::default()).expect("Failed to build UI");
    ui::MainWindow::init_image_frame(&main_window.image_frame);
    nwg::dispatch_thread_events();
}

//TODO 
// - Add a way to change config.
// - Resize the image
