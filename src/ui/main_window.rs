use slint::{ComponentHandle, Model, VecModel, SharedString, Weak};
use std::rc::Rc;
use crate::features::markdown_editor::{MarkdownEditor, OpenFile};
use pulldown_cmark::{Parser, html};

slint::include_modules!();

slint::slint! {
    import { Button, VerticalBox, HorizontalBox, GroupBox, LineEdit, TextEdit } from "std-widgets.slint";

    struct OpenFileData {
        path: string,
        is_modified: bool,
    }

    component SidebarButton inherits Button {
        width: 40px;
        height: 40px;
    }

    component FileTab inherits Rectangle {
        callback clicked();
        in property <string> file-name;
        in property <bool> is-modified;

        width: 120px;
        height: 30px;
        background: lightgray;

        HorizontalLayout {
            Text {
                text: file-name + (is-modified ? "*" : "");
            }
        }

        TouchArea {
            clicked => {
                root.clicked();
            }
        }
    }

    global Callbacks {
        pure callback create_file(string);
        pure callback open_file(string);
        pure callback save_file();
        pure callback update_content(string);
        pure callback request_preview() -> string;
    }

    export component AppWindow inherits Window {
        title: "Nodian";
        width: 1024px;
        height: 768px;

        in property <[string]> file_tree: [];
        in property <[OpenFileData]> open_files: [];

        HorizontalBox {
            VerticalBox {
                width: 50px;
                SidebarButton { text: "📝"; }
                SidebarButton { text: "📅"; }
                SidebarButton { text: "🔧"; }
                SidebarButton { text: "⏱️"; }
                SidebarButton { text: "🔒"; }
                SidebarButton { text: "📋"; }
            }
            VerticalBox {
                HorizontalBox {
                    height: 30px;
                    LineEdit {
                        placeholder-text: "Enter file name";
                        edited => {
                            Callbacks.create_file(self.text);
                            self.text = "";
                        }
                    }
                    Button {
                        text: "Save";
                        clicked => { Callbacks.save_file(); }
                    }
                }
                HorizontalBox {
                    for file in open_files: FileTab {
                        file-name: file.path;
                        is-modified: file.is_modified;
                        clicked => {
                            Callbacks.open_file(file.path);
                        }
                    }
                }
                HorizontalBox {
                    VerticalBox {
                        width: 200px;
                        for file in file_tree: Text {
                            text: file;
                            TouchArea {
                                clicked => { Callbacks.open_file(file); }
                            }
                        }
                    }
                    TextEdit {
                        // edited => {
                        //     let text = self.text;
                        //     if let Some((x, y)) = text.parse::<(f32, f32)>().ok() {
                        //         // 根据你的逻辑选择一个值，或者计算一个单一的值
                        //         let value = x; // 或者 let value = (x + y) / 2.0;
                        //         Callbacks.update_content(value.to_string());
                        //     }
                        // }
                        edited => { Callbacks.update_content(self.text); }
                    }
                    Rectangle {
                        width: 50%;
                        background: white;
                        Text {
                            text: Callbacks.request_preview();
                        }
                    }
                }
            }
        }
    }
}

pub struct MainWindow {
    window: Weak<AppWindow>,
    markdown_editor: Rc<std::cell::RefCell<MarkdownEditor>>,
}

impl MainWindow {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = AppWindow::new()?;
        let markdown_editor = Rc::new(std::cell::RefCell::new(MarkdownEditor::new()));

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_create_file(move |name| {
            if let Err(e) = md_editor.borrow_mut().create_file(&name) {
                eprintln!("Failed to create file: {}", e);
            }
        });

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_open_file(move |path| {
            if let Err(e) = md_editor.borrow_mut().open_file(std::path::Path::new(path.as_str())) {
                eprintln!("Failed to open file: {}", e);
            }
        });

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_save_file(move || {
            if let Err(e) = md_editor.borrow_mut().save_file() {
                eprintln!("Failed to save file: {}", e);
            }
        });

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_update_content(move |content| {
            md_editor.borrow_mut().update_content(content.to_string());
        });

        let md_editor = markdown_editor.clone();
        window.global::<Callbacks>().on_request_preview(move || {
            let content = md_editor.borrow().get_content();
            let parser = Parser::new(content);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            html_output.into()
        });

        Ok(Self { window: window.as_weak(), markdown_editor })
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.update_file_tree();
        self.update_open_files();
        slint::run_event_loop()
    }

    fn update_file_tree(&self) {
        let files = self.markdown_editor.borrow().get_file_tree();
        let file_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(
            files.into_iter().map(|p| p.to_string_lossy().to_string().into()).collect::<Vec<SharedString>>()
        ));
        if let Some(window) = self.window.upgrade() {
            window.set_file_tree(file_model.into());
        }
    }

    fn update_open_files(&self) {
        let open_files = self.markdown_editor.borrow().get_open_files();
        let open_files_model: Rc<VecModel<OpenFileData>> = Rc::new(VecModel::from(
            open_files.iter().map(|f| OpenFileData {
                path: f.path.to_string_lossy().to_string().into(),
                is_modified: f.is_modified,
            }).collect::<Vec<OpenFileData>>()
        ));
        if let Some(window) = self.window.upgrade() {
            window.set_open_files(open_files_model.into());
        }
    }
}