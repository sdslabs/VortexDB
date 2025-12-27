use api::{DbConfig, VectorDb, init_api};
use index::IndexType;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use storage::{StorageEngine, StorageType};

pub struct DatabaseManager {
    pub current_db_path: Option<PathBuf>,
    pub storage_engine: Option<Arc<dyn StorageEngine>>,
    pub api_db: Option<VectorDb>,
    pub available_databases: Vec<(String, PathBuf)>,
    pub selected_database: Option<(String, PathBuf)>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            current_db_path: None,
            storage_engine: None,
            api_db: None,
            available_databases: Vec::new(),
            selected_database: None,
        }
    }

    fn close_current_database(&mut self) {
        self.storage_engine = None;
        self.current_db_path = None;
        self.selected_database = None;
        self.api_db = None;
    }

    pub fn create_new_database(&mut self, name: String, path: PathBuf) -> io::Result<()> {
        // Drop any database currently open before creating a new one
        self.close_current_database();
        // Check if database already exists to avoid name collisions
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Database '{name}' already exists!"),
            ));
        }

        let cfg = DbConfig {
            storage_type: StorageType::RocksDb,
            index_type: IndexType::Flat,
            data_path: path.clone(),
            dimension: 512,
        };

        match init_api(cfg) {
            Ok(db) => {
                self.api_db = Some(db);
                self.current_db_path = Some(path.clone());
                self.available_databases.push((name.clone(), path.clone()));
                self.selected_database = Some((name, path));
                Ok(())
            }
            Err(err) => Err(io::Error::other(format!(
                "Failed to create database: {:?}",
                err
            ))),
        }
    }

    pub fn select_database(&mut self, name: String, path: PathBuf) -> io::Result<()> {
        self.close_current_database();

        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Database path does not exist",
            ));
        }

        let cfg = DbConfig {
            storage_type: StorageType::RocksDb,
            index_type: IndexType::Flat,
            data_path: path.clone(),
            dimension: 512,
        };

        match init_api(cfg) {
            Ok(db) => {
                self.api_db = Some(db);
                self.current_db_path = Some(path.clone());
                self.selected_database = Some((name, path));
                Ok(())
            }
            Err(err) => Err(io::Error::other(format!(
                "Failed to open database: {:?}",
                err
            ))),
        }
    }

    pub fn delete_database(&mut self, path: &PathBuf) -> io::Result<()> {
        // Close current connection if it's the same database
        if let Some(current_path) = &self.current_db_path
            && current_path == path
        {
            self.storage_engine = None;
            self.current_db_path = None;
            self.api_db = None;
        }

        // Remove from available databases list
        self.available_databases
            .retain(|(_, db_path)| db_path != path);

        // Delete the database directory
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }

        Ok(())
    }

    pub fn load_available_databases(&mut self) -> io::Result<()> {
        let db_dir = PathBuf::from("./databases");
        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir)?;
        }

        self.available_databases.clear();

        for entry in std::fs::read_dir(&db_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name()
            {
                self.available_databases
                    .push((name.to_string_lossy().to_string(), path));
            }
        }

        Ok(())
    }

    pub fn get_selected_database_name(&self) -> Option<&str> {
        self.selected_database
            .as_ref()
            .map(|(name, _)| name.as_str())
    }

    pub fn is_database_selected(&self) -> bool {
        self.selected_database.is_some()
    }
}
