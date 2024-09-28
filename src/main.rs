mod ui;
mod features;
mod db;

use ui::MainWindow;

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.run()
}
