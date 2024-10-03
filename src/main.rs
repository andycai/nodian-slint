mod ui;
mod features;
mod db;

use ui::MainWindow;

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    println!("About to run the main window");
    main_window.run()
}
