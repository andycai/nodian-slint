mod ui;
mod db;

use ui::MainWindow;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        let main_window = MainWindow::new()?;
        println!("About to run the main window");
        main_window.run().await
    })?;
    Ok(())
}
