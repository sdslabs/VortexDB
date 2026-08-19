use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyRole {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub name: String,
    pub role: ApiKeyRole,
    pub key: String,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyStore {
    keys: Vec<ApiKeyEntry>,
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

impl ApiKeyStore {
    pub fn new(keys: Vec<ApiKeyEntry>) -> Self {
        Self { keys }
    }

    pub fn find(&self, presented: &str) -> Option<&ApiKeyEntry> {
        let mut matched = None;
        for entry in &self.keys {
            if constant_time_eq(entry.key.as_bytes(), presented.as_bytes()) {
                matched = Some(entry);
            }
        }
        matched
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn duplicate_key(&self) -> Option<&str> {
        for (i, entry) in self.keys.iter().enumerate() {
            for other in &self.keys[i + 1..] {
                if constant_time_eq(entry.key.as_bytes(), other.key.as_bytes()) {
                    return Some(&entry.key);
                }
            }
        }
        None
    }
}
