use anyhow::{Ok, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::PathBuf,
};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub uuid: String,
    pub content: String,
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ClipboardStorage {
    entries: Vec<ClipboardEntry>,
}

impl ClipboardStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Result<Self> {
        let path = Self::storage_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let file = File::open(path)?;
        let storage: Self = serde_json::from_reader(file)?;
        Ok(storage)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path()?;
        let parent = path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir(parent)?;
        }

        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn add_entry(&mut self, content: String, nickname: Option<String>) -> Result<()> {
        let new_entry = ClipboardEntry {
            uuid: Uuid::new_v4().to_string(),
            content,
            nickname,
        };

        self.entries.push(new_entry);
        self.save()?;
        Ok(())
    }

    pub fn remove_entry(&mut self, index: usize) -> Result<()> {
        self.entries.remove(index);
        self.save()?;
        Ok(())
    }

    pub fn get_entries(&self) -> &[ClipboardEntry] {
        &self.entries
    }

    pub fn storage_path() -> Result<PathBuf> {
        let dir = ProjectDirs::from("com", "cogStudios", "cliphoard")
            .ok_or_else(|| anyhow::anyhow!("Could not locate project directory."))?;
        Ok(dir.data_dir().join("snippets.json"))
    }
}
