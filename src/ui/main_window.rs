use slint::{self, ComponentHandle, Model, ModelRc, SharedString, Weak};
use std::sync::{Arc, Mutex};
use std::rc::Rc;  // 添加这一行
use crate::features::markdown_editor::{MarkdownEditor, OpenFile};
use pulldown_cmark::{Parser, html};

// 修改这一行
use crate::ui::{AppWindow, Callbacks, OpenFileData};

pub struct MainWindow {
    window: Arc<AppWindow>,  // 改为 Arc<AppWindow>
    markdown_editor: Arc<Mutex<MarkdownEditor>>,
}

impl MainWindow {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = Arc::new(AppWindow::new()?);  // 使用 Arc::new
        let markdown_editor = Arc::new(Mutex::new(MarkdownEditor::new()));

        let md_editor = markdown_editor.clone();
        let window_clone = window.clone();
        window.global::<Callbacks>().on_create_file(move |name: SharedString| {
            if let Err(e) = md_editor.lock().unwrap().create_file(&name) {
                eprintln!("Failed to create file: {}", e);
            }
        });

        let md_editor = markdown_editor.clone();
        let window_clone = window.clone();
        window.global::<Callbacks>().on_open_file(move |path: SharedString| {
            if let Err(e) = md_editor.lock().unwrap().open_file(std::path::Path::new(path.as_str())) {
                eprintln!("Failed to open file: {}", e);
            }
        });

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_save_file(move || {
            if let Err(e) = md_editor.lock().unwrap().save_file() {
                eprintln!("Failed to save file: {}", e);
            }
        });

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_update_content(move |content: SharedString| {
            md_editor.lock().unwrap().update_content(content.to_string());
        });

        let md_editor = markdown_editor.clone();
        let weak_window = window_clone.as_weak();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let content = {
                    let editor = md_editor.lock().unwrap();
                    editor.get_content().to_string() // 克隆内容
                }; // 锁在这里被释放
                let parser = Parser::new(&content);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);
                weak_window.upgrade_in_event_loop(move |handle| {
                    handle.set_preview_content(html_output.into());
                }).ok();
            }
        });

        println!("MainWindow created successfully");
        Ok(Self { window, markdown_editor })  // 不再需要 as_weak()
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        println!("Updating file tree");
        self.update_file_tree();
        println!("Updating open files");
        self.update_open_files();
        println!("Attempting to show window");
        
        self.window.show()?;
        println!("Window shown, starting event loop");
        slint::run_event_loop()?;
        
        println!("Event loop finished");
        Ok(())
    }

    fn update_file_tree(&self) {
        let files = self.markdown_editor.lock().unwrap().get_file_tree();
        let file_model: Rc<slint::VecModel<SharedString>> = Rc::new(slint::VecModel::from(
            files.into_iter().map(|p| p.to_string_lossy().to_string().into()).collect::<Vec<SharedString>>()
        ));
        self.window.set_file_tree(ModelRc::new(file_model));
    }

    fn update_open_files(&self) {
        let markdown_editor = self.markdown_editor.lock().unwrap();
        let open_files = markdown_editor.get_open_files();
        let open_files_vec: Vec<OpenFileData> = open_files.iter().map(|f| OpenFileData {
            path: f.path.to_string_lossy().to_string().into(),
            is_modified: f.is_modified,
        }).collect();
        drop(markdown_editor); // 释放锁

        let open_files_model: Rc<slint::VecModel<OpenFileData>> = Rc::new(slint::VecModel::from(open_files_vec));
        self.window.set_open_files(ModelRc::new(open_files_model));
    }
}