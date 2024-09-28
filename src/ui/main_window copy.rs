// use slint::{ComponentHandle, Model, VecModel, SharedString};
// use std::rc::Rc;
// use crate::features::markdown_editor::{MarkdownEditor, OpenFile};
// use pulldown_cmark::{Parser, html};

// slint::include_modules!();

// // 将 slint! 宏移到结构体定义之前
// slint::slint! {
//     import { Button, VerticalBox, HorizontalBox, GroupBox, LineEdit, TextEdit } from "std-widgets.slint";

//     struct OpenFileData {
//         path: string,
//         is_modified: bool,
//     }

//     component SidebarButton inherits Button {
//         width: 40px;
//         height: 40px;
//     }

//     component FileTab inherits Rectangle {
//         callback clicked();
//         in property <string> file-name;
//         in property <bool> is-modified;

//         width: 120px;
//         height: 30px;
//         background: lightgray;

//         HorizontalLayout {
//             Text {
//                 text: file-name + (is-modified ? "*" : "");
//             }
//         }

//         TouchArea {
//             clicked => {
//                 root.clicked();
//             }
//         }
//     }

//     export global Callbacks {
//         callback create_file(string);
//         callback open_file(string);
//         callback save_file();
//         callback update_content(string);
//         callback request_preview() -> string;
//     }

//     export component AppWindow inherits Window {
//         title: "Nodian";
//         width: 1024px;
//         height: 768px;

//         in property <[string]> file_tree: [];
//         in property <[OpenFileData]> open_files: [];

//         HorizontalBox {
//             VerticalBox {
//                 width: 50px;
//                 SidebarButton { text: "📝"; }
//                 SidebarButton { text: "📅"; }
//                 SidebarButton { text: "🔧"; }
//                 SidebarButton { text: "⏱️"; }
//                 SidebarButton { text: "🔒"; }
//                 SidebarButton { text: "📋"; }
//             }
//             VerticalBox {
//                 HorizontalBox {
//                     height: 30px;
//                     LineEdit {
//                         placeholder-text: "Enter file name";
//                         edited => {
//                             Callbacks.create_file(self.text);
//                             self.text = "";
//                         }
//                     }
//                     Button {
//                         text: "Save";
//                         clicked => { Callbacks.save_file(); }
//                     }
//                 }
//                 HorizontalBox {
//                     for file in open_files: FileTab {
//                         file-name: file.path;
//                         is-modified: file.is_modified;
//                         clicked => {
//                             Callbacks.open_file(file.path);
//                         }
//                     }
//                 }
//                 HorizontalBox {
//                     VerticalBox {
//                         width: 200px;
//                         for file in file_tree: Text {
//                             text: file;
//                             mouse-area := TouchArea {
//                                 clicked => { Callbacks.open_file(file); }
//                             }
//                         }
//                     }
//                     TextEdit {
//                         edited => { Callbacks.update_content(self.text); }
//                     }
//                     Rectangle {
//                         width: 50%;
//                         background: white;
//                         Text {
//                             text: Callbacks.request_preview();
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub struct MainWindow {
//     window: AppWindow,
//     markdown_editor: Rc<std::cell::RefCell<MarkdownEditor>>,
// }

// impl MainWindow {
//     pub fn new() -> Result<Self, slint::PlatformError> {
//         let window = AppWindow::new()?;
//         let markdown_editor = Rc::new(std::cell::RefCell::new(MarkdownEditor::new()));

//         let md_editor = markdown_editor.clone();
//         window.global::<Callbacks>().on_create_file(move |name: slint::SharedString| {
//             if let Err(e) = md_editor.borrow_mut().create_file(&name) {
//                 eprintln!("Failed to create file: {}", e);
//             }
//         });

//         let md_editor = markdown_editor.clone();
//         window.global::<Callbacks>().on_open_file(move |path| {
//             if let Err(e) = md_editor.borrow_mut().open_file(std::path::Path::new(&path)) {
//                 eprintln!("Failed to open file: {}", e);
//             }
//         });

//         let md_editor = markdown_editor.clone();
//         window.global::<Callbacks>().on_save_file(move || {
//             if let Err(e) = md_editor.borrow_mut().save_file() {
//                 eprintln!("Failed to save file: {}", e);
//             }
//         });

//         let md_editor = markdown_editor.clone();
//         window.global::<Callbacks>().on_update_content(move |content| {
//             md_editor.borrow_mut().update_content(content.to_string());
//         });

//         let md_editor = markdown_editor.clone();
//         window.global::<Callbacks>().on_request_preview(move || {
//             let content = md_editor.borrow().get_content();
//             let parser = Parser::new(content);
//             let mut html_output = String::new();
//             html::push_html(&mut html_output, parser);
//             html_output.into()
//         });

//         Ok(Self { window, markdown_editor })
//     }

//     pub fn run(&self) -> Result<(), slint::PlatformError> {
//         self.update_file_tree();
//         self.update_open_files();
//         self.window.run()
//     }

//     fn update_file_tree(&self) {
//         let files = self.markdown_editor.borrow().get_file_tree();
//         let file_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(
//             files.into_iter().map(|p| p.to_string_lossy().to_string().into()).collect::<Vec<SharedString>>()
//         ));
//         self.window.set_file_tree(file_model.into());
//     }

//     fn update_open_files(&self) {
//         let open_files = self.markdown_editor.borrow().get_open_files();
//         let open_files_model: Rc<VecModel<slint::ModelRc<OpenFileData>>> = Rc::new(VecModel::from(
//             open_files.iter().map(|f| slint::ModelRc::new(OpenFileData {
//                 path: f.path.to_string_lossy().to_string().into(),
//                 is_modified: f.is_modified,
//             })).collect::<Vec<slint::ModelRc<OpenFileData>>>()
//         ));
//         self.window.set_open_files(open_files_model.into());
//     }
// }