use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenFile {
    pub path: PathBuf,
    pub is_modified: bool,
}

pub struct MarkdownEditor {
    root_dir: PathBuf,
    current_file: Option<PathBuf>,
    content: String,
    open_files: Vec<OpenFile>,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        let root_dir = PathBuf::from("nodian");
        if !root_dir.exists() {
            fs::create_dir(&root_dir).expect("Failed to create nodian directory");
        }
        let open_files = Self::load_open_files();
        Self {
            root_dir,
            current_file: None,
            content: String::new(),
            open_files,
        }
    }

    pub fn open_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        self.content.clear();
        file.read_to_string(&mut self.content)?;
        self.current_file = Some(path.to_path_buf());
        self.add_to_open_files(path);
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = self.current_file.clone() {
            let content = self.content.clone();
            let mut file = File::create(&path)?;
            file.write_all(content.as_bytes())?;
            self.mark_file_as_saved(&path);
        }
        self.save_open_files();
        Ok(())
    }

    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        if let Some(path) = self.current_file.clone() {
            self.mark_file_as_modified(&path);
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn create_file(&mut self, name: &str) -> Result<(), std::io::Error> {
        let path = self.root_dir.join(name);
        File::create(&path)?;
        self.open_file(&path)?;
        Ok(())
    }

    pub fn get_file_tree(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.walk_dir(&self.root_dir, &mut files);
        files
    }

    pub fn get_open_files(&self) -> &[OpenFile] {
        &self.open_files
    }

    fn walk_dir(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.walk_dir(&path, files);
                } else if path.extension().map_or(false, |ext| ext == "md") {
                    files.push(path);
                }
            }
        }
    }

    fn add_to_open_files(&mut self, path: &Path) {
        if !self.open_files.iter().any(|f| f.path == path) {
            self.open_files.push(OpenFile {
                path: path.to_path_buf(),
                is_modified: false,
            });
            self.save_open_files();
        }
    }

    fn mark_file_as_modified(&mut self, path: &Path) {
        if let Some(file) = self.open_files.iter_mut().find(|f| f.path == path) {
            file.is_modified = true;
        }
    }

    fn mark_file_as_saved(&mut self, path: &Path) {
        if let Some(file) = self.open_files.iter_mut().find(|f| f.path == path) {
            file.is_modified = false;
        }
    }

    fn save_open_files(&self) {
        let json = serde_json::to_string(&self.open_files).unwrap();
        fs::write("open_files.json", json).expect("Unable to save open files");
    }

    fn load_open_files() -> Vec<OpenFile> {
        match fs::read_to_string("open_files.json") {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }
}