use slint::{self, ComponentHandle, Model, ModelRc, SharedString, Weak, VecModel};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use tokio::sync::mpsc;
use crate::ui::markdown_editor::MarkdownEditor;
use pulldown_cmark::{Parser, html};
use crate::ui::{AppWindow, Callbacks, OpenFileData};
use std::path::{PathBuf, Path};
use std::fs;
use tokio::time::{sleep, Duration};

pub struct MainWindow {
    window: Rc<AppWindow>,
    markdown_editor: Arc<Mutex<MarkdownEditor>>,
    tx: mpsc::Sender<UIMessage>,
}

enum UIMessage {
    UpdateFileTree(Vec<String>),
    UpdateOpenFiles(Vec<OpenFileData>),
    UpdatePreview(String),
    CreateFile(String),
    OpenFile(String),
    CloseFile(String),
    SaveFile,
    UpdateEditorContent(String),
    UpdateEditorContentFromUI(String),
}

impl MainWindow {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = Rc::new(AppWindow::new()?);
        let markdown_editor = Arc::new(Mutex::new(MarkdownEditor::new()));
        let (tx, rx) = mpsc::channel(100);

        // 设置初始的 editor_content
        window.set_editor_content("".into());

        let tx_clone = tx.clone();
        window.global::<Callbacks>().on_create_file(move |name: SharedString| {
            let tx = tx_clone.clone();
            let name = name.to_string();
            tokio::spawn(async move {
                tx.send(UIMessage::CreateFile(name)).await.unwrap();
            });
        });

        let tx_clone = tx.clone();
        window.global::<Callbacks>().on_open_file(move |path: SharedString| {
            let tx = tx_clone.clone();
            let path = path.to_string();
            println!("on_open_file called with path: {}", path); // 保留这行调试输出
            tokio::spawn(async move {
                tx.send(UIMessage::OpenFile(path)).await.unwrap();
            });
        });

        let tx_clone = tx.clone();
        window.global::<Callbacks>().on_save_file(move || {
            let tx = tx_clone.clone();
            tokio::spawn(async move {
                tx.send(UIMessage::SaveFile).await.unwrap();
            });
        });

        let tx_clone = tx.clone();
        window.global::<Callbacks>().on_close_file(move |path: SharedString| {
            let tx = tx_clone.clone();
            let path = path.to_string();
            println!("Close file button clicked for path: {}", path); // 添加这行
            tokio::spawn(async move {
                tx.send(UIMessage::CloseFile(path)).await.unwrap();
            });
        });

        let tx_clone = tx.clone();
        window.on_save_shortcut(move || {
            let tx = tx_clone.clone();
            tokio::spawn(async move {
                tx.send(UIMessage::SaveFile).await.unwrap();
            });
        });

        // let window_weak = window.as_weak();
        // window.on_get_editor_content(move || {
        //     window_weak.upgrade_in_event_loop(|handle| {
        //         handle.get_editor_content()
        //     }).unwrap_or_default()
        // });

        let tx_clone = tx.clone();
        window.global::<Callbacks>().on_update_editor_content(move |content: SharedString| {
            let tx = tx_clone.clone();
            let content = content.to_string();
            tokio::spawn(async move {
                tx.send(UIMessage::UpdateEditorContentFromUI(content)).await.unwrap();
            });
        });

        let window_weak = window.as_weak();
        let md_editor = markdown_editor.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            Self::run_event_loop(window_weak, md_editor, rx, tx_clone).await;
        });

        println!("MainWindow created successfully");
        Ok(Self { window, markdown_editor, tx })
    }

    pub async fn run(&self) -> Result<(), slint::PlatformError> {
        println!("Updating file tree");
        self.tx.send(UIMessage::UpdateFileTree(Vec::new())).await.unwrap();
        println!("Updating open files");
        self.tx.send(UIMessage::UpdateOpenFiles(Vec::new())).await.unwrap();
        println!("Attempting to show window");
        
        self.window.show()?;
        println!("Window shown, starting event loop");
        slint::run_event_loop()?;
        
        println!("Event loop finished");
        Ok(())
    }

    async fn run_event_loop(
        window: Weak<AppWindow>,
        markdown_editor: Arc<Mutex<MarkdownEditor>>,
        mut rx: mpsc::Receiver<UIMessage>,
        tx: mpsc::Sender<UIMessage>,
    ) {
        // Load initial directory tree
        if let Ok(files) = Self::load_directory_tree(Path::new("nodian")) {
            tx.send(UIMessage::UpdateFileTree(files)).await.unwrap();
        }

        // Load initial open files
        let open_files_data = {
            let editor = markdown_editor.lock().unwrap();
            let open_files = editor.get_open_files();
            let current_file = editor.get_current_file();
            open_files.iter().map(|f| OpenFileData {
                path: f.path.to_string_lossy().to_string().into(),
                is_modified: f.is_modified,
                is_active: current_file.as_ref().map(|cf| cf == &f.path).unwrap_or(false),
            }).collect::<Vec<OpenFileData>>()
        };
        tx.send(UIMessage::UpdateOpenFiles(open_files_data)).await.unwrap();

        while let Some(msg) = rx.recv().await {
            let window = window.clone();
            match msg {
                UIMessage::UpdateFileTree(files) => {
                    let files_clone = files.clone();
                    window.upgrade_in_event_loop(move |handle| {
                        let file_model = Rc::new(VecModel::from(
                            files_clone.into_iter().map(SharedString::from).collect::<Vec<SharedString>>()
                        ));
                        handle.set_file_tree(ModelRc::new(file_model));
                    }).ok();
                },
                UIMessage::UpdateOpenFiles(open_files) => {
                    let open_files_clone = open_files.clone();
                    window.upgrade_in_event_loop(move |handle| {
                        let open_files_model = Rc::new(VecModel::from(open_files_clone));
                        handle.set_open_files(ModelRc::new(open_files_model));
                    }).ok();
                },
                UIMessage::UpdatePreview(html) => {
                    let html_clone = html.clone();
                    window.upgrade_in_event_loop(move |handle| {
                        handle.set_preview_content(html_clone.into());
                    }).ok();
                },
                UIMessage::CreateFile(name) => {
                    if let Err(e) = markdown_editor.lock().unwrap().create_file(&name) {
                        eprintln!("Failed to create file: {}", e);
                    }
                    // Update the file tree after creating a new file
                    if let Ok(files) = Self::load_directory_tree(Path::new("nodian")) {
                        tx.send(UIMessage::UpdateFileTree(files)).await.unwrap();
                    }
                },
                UIMessage::OpenFile(path) => {
                    // println!("Attempting to open file: {}", path);
                    let full_path = if Path::new(&path).is_absolute() {
                        PathBuf::from(&path)
                    } else {
                        Path::new("nodian").join(&path)
                    };
                    // println!("Full path: {:?}", full_path);
                    
                    let content = {
                        let mut editor = markdown_editor.lock().unwrap();
                        match editor.open_file(&full_path) {
                            Ok(()) => {
                                // println!("File opened successfully");
                                editor.get_content().to_string()
                            },
                            Err(e) => {
                                eprintln!("Failed to open file: {}", e);
                                String::new()
                            }
                        }
                    };

                    // Update UI outside of the lock
                    let content_clone = content.clone();
                    window.upgrade_in_event_loop(move |handle| {
                        handle.set_editor_content(content_clone.into());
                    }).ok();
                    
                    // Update preview
                    let parser = Parser::new(&content);
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    tx.send(UIMessage::UpdatePreview(html_output)).await.unwrap();

                    // Send a message to update open files
                    let open_files_data = {
                        let editor = markdown_editor.lock().unwrap();
                        let open_files = editor.get_open_files();
                        let current_file = editor.get_current_file();
                        open_files.iter().map(|f| OpenFileData {
                            path: f.path.strip_prefix(&editor.get_root_dir())
                                .unwrap_or(&f.path)
                                .to_string_lossy()
                                .to_string()
                                .into(),
                            is_modified: f.is_modified,
                            is_active: current_file.as_ref().map(|cf| cf == f.path.as_path()).unwrap_or(false),
                        }).collect::<Vec<OpenFileData>>()
                    };
                    tx.send(UIMessage::UpdateOpenFiles(open_files_data)).await.unwrap();
                },
                UIMessage::CloseFile(path) => {
                    println!("Attempting to close file: {}", path);
                    let result = markdown_editor.lock().unwrap().close_file(&path);
                    match result {
                        Ok(()) => {
                            println!("File closed successfully: {}", path);
                            
                            // Send a message to update open files
                            let open_files_data: Vec<OpenFileData> = {
                                let editor = markdown_editor.lock().unwrap();
                                let open_files = editor.get_open_files();
                                let current_file = editor.get_current_file();
                                open_files.iter().map(|f| OpenFileData {
                                    path: f.path.strip_prefix(&editor.get_root_dir())
                                        .unwrap_or(&f.path)
                                        .to_string_lossy()
                                        .to_string()
                                        .into(),
                                    is_modified: f.is_modified,
                                    is_active: current_file.as_ref().map(|cf| cf == f.path.as_path()).unwrap_or(false),
                                }).collect()
                            };
                            tx.send(UIMessage::UpdateOpenFiles(open_files_data)).await.unwrap();
                            
                            // Update the file tree
                            if let Ok(files) = Self::load_directory_tree(Path::new("nodian")) {
                                tx.send(UIMessage::UpdateFileTree(files)).await.unwrap();
                            }

                            // Clear editor content if the closed file was the current file
                            if markdown_editor.lock().unwrap().get_current_file().is_none() {
                                window.upgrade_in_event_loop(|handle| {
                                    handle.set_editor_content("".into());
                                    handle.set_preview_content("".into());
                                }).ok();
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to close file: {}", e);
                        }
                    }
                },
                UIMessage::SaveFile => {
                    // println!("Save file command received");

                    let content = {
                        let editor = markdown_editor.lock().unwrap();
                        editor.get_content().to_string()
                    };

                    let result = {
                        let mut editor = markdown_editor.lock().unwrap();
                        editor.save_file()
                    };

                    match result {
                        Ok(()) => {
                            println!("File saved successfully");
                            // 更新打开文件的状态
                            let open_files_data: Vec<OpenFileData> = {
                                let editor = markdown_editor.lock().unwrap();
                                let open_files = editor.get_open_files();
                                let current_file = editor.get_current_file();
                                open_files.iter().map(|f| OpenFileData {
                                    path: f.path.strip_prefix(&editor.get_root_dir())
                                        .unwrap_or(&f.path)
                                        .to_string_lossy()
                                        .to_string()
                                        .into(),
                                    is_modified: f.is_modified,
                                    is_active: current_file.as_ref().map(|cf| cf == f.path.as_path()).unwrap_or(false),
                                }).collect()
                            };
                            tx.send(UIMessage::UpdateOpenFiles(open_files_data)).await.unwrap();

                            // 更新 UI 以反映文件已保存
                            window.upgrade_in_event_loop(|handle| {
                                handle.window().request_redraw();
                            }).ok();
                        }
                        Err(e) => {
                            eprintln!("Failed to save file: {}", e);
                            // 可以在这里添加错误提示的 UI 更新
                        }
                    }
                },
                UIMessage::UpdateEditorContent(content) => {
                    // println!("Updating editor content, length: {}", content.len());
                    {
                        let mut editor = markdown_editor.lock().unwrap();
                        editor.update_content(content.clone());
                    }
                    window.upgrade_in_event_loop(move |handle| {
                        handle.set_editor_content(content.into());
                    }).ok();

                    // 更新打开文件的状态
                    let open_files_data: Vec<OpenFileData> = {
                        let editor = markdown_editor.lock().unwrap();
                        let open_files = editor.get_open_files();
                        let current_file = editor.get_current_file();
                        open_files.iter().map(|f| OpenFileData {
                            path: f.path.strip_prefix(&editor.get_root_dir())
                                .unwrap_or(&f.path)
                                .to_string_lossy()
                                .to_string()
                                .into(),
                            is_modified: f.is_modified,
                            is_active: current_file.as_ref().map(|cf| cf == f.path.as_path()).unwrap_or(false),
                        }).collect()
                    };
                    tx.send(UIMessage::UpdateOpenFiles(open_files_data)).await.unwrap();
                },
                UIMessage::UpdateEditorContentFromUI(content) => {
                    {
                        let mut editor = markdown_editor.lock().unwrap();
                        editor.update_content(content.clone());
                    }

                    // 更新预览
                    let parser = Parser::new(&content);
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    tx.send(UIMessage::UpdatePreview(html_output)).await.unwrap();

                    // 更新打开文件的状态
                    let open_files_data = {
                        let editor = markdown_editor.lock().unwrap();
                        editor.get_open_files().iter().map(|f| OpenFileData {
                            path: f.path.strip_prefix(&editor.get_root_dir())
                                .unwrap_or(&f.path)
                                .to_string_lossy()
                                .to_string()
                                .into(),
                            is_modified: f.is_modified,
                            is_active: editor.get_current_file().as_ref().map(|cf| cf == f.path.as_path()).unwrap_or(false),
                        }).collect::<Vec<OpenFileData>>()
                    };
                    tx.send(UIMessage::UpdateOpenFiles(open_files_data)).await.unwrap();
                },
            }
        }
    }

    fn load_directory_tree(root: &Path) -> std::io::Result<Vec<String>> {
        let mut result = Vec::new();
        if !root.exists() {
            fs::create_dir_all(root)?;
        }
        Self::load_directory_tree_recursive(root, &mut result, 0)?;
        Ok(result)
    }

    fn load_directory_tree_recursive(dir: &Path, result: &mut Vec<String>, depth: usize) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            
            if path.is_dir() {
                result.push(format!("{}{}/", "  ".repeat(depth), file_name));
                Self::load_directory_tree_recursive(&path, result, depth + 1)?;
            } else if path.extension().map_or(false, |ext| ext == "md") {
                result.push(format!("{}{}", "  ".repeat(depth), file_name));
            }
        }
        Ok(())
    }
}