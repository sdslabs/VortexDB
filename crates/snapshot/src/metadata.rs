use crate::constants::SMALL_ID_LEN;
use chrono::DateTime;
use chrono::Local;
use defs::DbError;
use semver::Version;
use std::{fmt::Display, path::PathBuf, time::SystemTime};
use std::{fs, path::Path};
use uuid::Uuid;

pub type SmallID = String;

// Metadata is the data that can be parsed from the snapshot filename
#[derive(Debug, Clone)]
pub struct Metadata {
    pub small_id: SmallID,
    pub date: SystemTime,
    pub path: PathBuf,
    pub sem_ver: Version,
}

const FILENAME_METADATA_SEPARATOR: &str = "-x";

impl Metadata {
    pub fn new(id: Uuid, date: SystemTime, path: PathBuf, sem_ver: Version) -> Self {
        Metadata {
            small_id: id.to_string()[..SMALL_ID_LEN].to_string(),
            date,
            path,
            sem_ver,
        }
    }

    pub fn parse(path: &Path) -> Result<Metadata, DbError> {
        if !path.is_file() {
            return Err(DbError::SnapshotError("File not found".to_string()));
        }
        let filename = path
            .file_name()
            .ok_or(DbError::SnapshotError("No filename".to_string()))?
            .to_str()
            .ok_or(DbError::SnapshotError(
                "Invalid UTF-8 in filename".to_string(),
            ))?
            .strip_suffix(".tar.gz")
            .ok_or(DbError::SnapshotError(
                "Snapshot filename doesnt end with .tar.gz".to_string(),
            ))?;

        let parts = filename
            .split(FILENAME_METADATA_SEPARATOR)
            .collect::<Vec<&str>>();

        if parts.len() != 3 {
            return Err(DbError::SnapshotError("Invalid filename".to_string()));
        }

        let id = parts[1];
        if id.len() != SMALL_ID_LEN {
            return Err(DbError::SnapshotError("Invalid UUID".to_string()));
        }

        let date = chrono::DateTime::parse_from_rfc3339(parts[0])
            .map_err(|_| DbError::SnapshotError("Invalid date".to_string()))?;
        let version = Version::parse(parts[2])
            .map_err(|_| DbError::SnapshotError("Invalid version".to_string()))?;

        Ok(Metadata {
            small_id: id.to_string(),
            date: date.into(),
            path: path.to_path_buf(),
            sem_ver: version,
        })
    }

    pub fn snapshot_dir_metadata(path: &Path) -> Result<Vec<Metadata>, DbError> {
        if !path.is_dir() {
            return Err(DbError::SnapshotError(
                "Path is not a directory".to_string(),
            ));
        }

        let mut metadata_vec = Vec::new();

        for item in fs::read_dir(path).map_err(|_| {
            DbError::SnapshotError(format!("Cannot read directory: {}", path.display()))
        })? {
            let entry = item.map_err(|_| {
                DbError::SnapshotError(format!("Invalid entry: {}", path.display()))
            })?;
            let path = entry.path();
            if path.is_file()
                && let Ok(metadata) = Self::parse(&path)
            {
                metadata_vec.push(metadata);
            }
        }
        Ok(metadata_vec)
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt_now_local: DateTime<Local> = self.date.into();
        write!(
            f,
            "{}{}{}{}{}",
            dt_now_local.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            FILENAME_METADATA_SEPARATOR,
            self.small_id,
            FILENAME_METADATA_SEPARATOR,
            self.sem_ver
        )
    }
}
