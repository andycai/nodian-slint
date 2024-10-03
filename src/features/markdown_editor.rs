use std::fs;
use std::path::{Path, PathBuf};

pub struct MarkdownEditor {
    current_file: Option<PathBuf>,
    content: String,
    root_dir: PathBuf,
    open_files: Vec<OpenFile>,
}

pub struct OpenFile {
    pub path: PathBuf,
    pub is_modified: bool,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        MarkdownEditor {
            current_file: None,
            content: String::new(),
            root_dir: PathBuf::from("nodian"),
            open_files: Vec::new(),
        }
    }

    pub fn open_file(&mut self, path: &Path) -> std::io::Result<()> {
        // 检查文件是否已经打开
        if self.open_files.iter().any(|f| f.path == path) {
            // 如果文件已经打开，只需将其设置为当前文件
            self.current_file = Some(path.to_path_buf());
            self.content = fs::read_to_string(path)?;
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        self.current_file = Some(path.to_path_buf());
        self.content = content;
        self.open_files.push(OpenFile {
            path: path.to_path_buf(),
            is_modified: false,
        });
        Ok(())
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_open_files(&self) -> &[OpenFile] {
        &self.open_files
    }

    pub fn get_current_file(&self) -> Option<&Path> {
        self.current_file.as_deref()
    }

    pub fn get_root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn close_file(&mut self, path: &str) -> Result<(), String> {
        let index = self.open_files.iter().position(|f| f.path.to_str().unwrap() == path)
            .ok_or_else(|| format!("File not found: {}", path))?;
        self.open_files.remove(index);
        if Some(PathBuf::from(path)) == self.current_file {
            self.current_file = None;
            self.content.clear();
        }
        Ok(())
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.current_file {
            fs::write(path, &self.content)?;
            if let Some(file) = self.open_files.iter_mut().find(|f| f.path == *path) {
                file.is_modified = false;
            }
        }
        Ok(())
    }

    pub fn create_file(&mut self, name: &str) -> std::io::Result<()> {
        let path = self.root_dir.join(name);
        fs::write(&path, "")?;
        self.open_file(&path)?;
        Ok(())
    }
}