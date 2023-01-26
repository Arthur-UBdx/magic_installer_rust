use native_windows_gui as nwg;
use native_windows_derive::NwgUi;

// ---- UI ---- //

// Main window
#[derive(Default, NwgUi)]
pub struct MainWindow{
    #[nwg_control(size: (500, 350), position: (300, 300), title: "Installateur magique", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [MainWindow::close_window()] )]
    pub window: nwg::Window,
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
    #[nwg_events( OnWindowClose: [] )]
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