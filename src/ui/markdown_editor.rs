use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use parking_lot::Mutex;
use pulldown_cmark::{Parser, Event, Tag};

pub struct MarkdownEditor {
    current_file: Mutex<Option<PathBuf>>,
    content: Mutex<String>,
    root_dir: PathBuf,
    open_files: Mutex<Vec<OpenFile>>,
}

#[derive(Clone)]
pub struct OpenFile {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_modified: bool,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        MarkdownEditor {
            current_file: Mutex::new(None),
            content: Mutex::new(String::new()),
            root_dir: PathBuf::from("nodian"),
            open_files: Mutex::new(Vec::new()),
        }
    }

    pub fn open_file(&mut self, path: &Path) -> std::io::Result<()> {
        // 检查文件是否已经打开
        if self.open_files.lock().iter().any(|f| f.path == path) {
            // 如果文件已经打开，只需将其设置为当前文件
            *self.current_file.lock() = Some(path.to_path_buf());
            *self.content.lock() = fs::read_to_string(path)?;
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        *self.current_file.lock() = Some(path.to_path_buf());
        *self.content.lock() = content;
        self.open_files.lock().push(OpenFile {
            path: path.to_path_buf(),
            is_dir: path.is_dir(),
            is_modified: false,
        });
        Ok(())
    }

    pub fn get_content(&self) -> String {
        self.content.lock().clone()
    }

    pub fn get_open_files(&self) -> Vec<OpenFile> {
        self.open_files.lock().clone()
    }

    pub fn get_current_file(&self) -> Option<PathBuf> {
        self.current_file.lock().clone()
    }

    pub fn get_root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn close_file(&mut self, path: &str) -> Result<(), String> {
        let full_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.root_dir.join(path)
        };

        let mut open_files = self.open_files.lock();
        let index = open_files.iter().position(|f| f.path == full_path)
            .ok_or_else(|| format!("File not found: {}", path))?;
        
        open_files.remove(index);
        
        if self.current_file.lock().as_ref() == Some(&full_path) {
            *self.current_file.lock() = None;
            self.content.lock().clear();
        }
        
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), io::Error> {
        if let Some(path) = &*self.current_file.lock() {
            println!("Attempting to save file: {:?}, {:?}", path, self.content.lock().clone());
            match fs::write(path, &*self.content.lock()) {
                Ok(_) => {
                    println!("File saved successfully");
                    if let Some(file) = self.open_files.lock().iter_mut().find(|f| f.path == *path) {
                        file.is_modified = false;
                    }
                    Ok(())
                },
                Err(e) => {
                    eprintln!("Error saving file: {:?}", e);
                    Err(e)
                }
            }
        } else {
            eprintln!("No current file to save");
            Err(io::Error::new(io::ErrorKind::Other, "No current file to save"))
        }
    }

    pub fn create_file(&mut self, name: &str) -> std::io::Result<()> {
        let path = self.root_dir.join(name);
        fs::write(&path, "")?;
        self.open_file(&path)?;
        Ok(())
    }

    pub fn update_content(&mut self, content: String) {
        *self.content.lock() = content;
        if let Some(current_file) = &*self.current_file.lock() {
            if let Some(file) = self.open_files.lock().iter_mut().find(|f| f.path == *current_file) {
                file.is_modified = true;
            }
        }
        println!("Content updated, length: {}", self.content.lock().len());
    }

    pub fn markdown_to_rich_text(markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut rich_text = String::new();
        let mut list_level = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Paragraph) => rich_text.push_str("<p>"),
                Event::End(Tag::Paragraph) => rich_text.push_str("</p>\n"),
                Event::Start(Tag::Heading(level, _, _)) => rich_text.push_str(&format!("<h{}>", level)),
                Event::End(Tag::Heading(level, _, _)) => rich_text.push_str(&format!("</h{}>\n", level)),
                Event::Start(Tag::List(..)) => {
                    list_level += 1;
                    rich_text.push_str("<ul>")
                },
                Event::End(Tag::List(..)) => {
                    list_level -= 1;
                    rich_text.push_str("</ul>")
                },
                Event::Start(Tag::Item) => rich_text.push_str(&format!("{}<li>", "  ".repeat(list_level))),
                Event::End(Tag::Item) => rich_text.push_str("</li>\n"),
                Event::Start(Tag::Emphasis) => rich_text.push_str("<i>"),
                Event::End(Tag::Emphasis) => rich_text.push_str("</i>"),
                Event::Start(Tag::Strong) => rich_text.push_str("<b>"),
                Event::End(Tag::Strong) => rich_text.push_str("</b>"),
                Event::Code(code) => rich_text.push_str(&format!("<code>{}</code>", code)),
                Event::Text(text) => rich_text.push_str(&text),
                Event::SoftBreak => rich_text.push('\n'),
                Event::HardBreak => rich_text.push_str("<br>"),
                _ => {}
            }
        }

        rich_text
    }

    pub fn update_preview(&self) -> String {
        Self::markdown_to_rich_text(&self.content.lock())
    }
}

// This is safe if all fields in MarkdownEditor are Send + Sync
unsafe impl Send for MarkdownEditor {}
unsafe impl Sync for MarkdownEditor {}