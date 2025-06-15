//For rocks-db
use super::types::Data;
use crate::kd_tree::KDTree;
use crate::keygen::*;
use hex::{decode, FromHexError as hexerr};
use rocksdb::{
    DBWithThreadMode,
    Error as err,
    IteratorMode,
    Options,
    // WriteBatch,
    // DBPinnableSlice,
    SingleThreaded,
    DB,
};
use sha2::{Digest, Sha256};

pub struct Database {
    pub db: DBWithThreadMode<SingleThreaded>,
    pub path: String,
    pub tree: KDTree,
}

impl Database {
    pub fn create_switch_database(addr: String) -> Result<Database, err> {
        let mut options = Options::default();

        //Optimize RocksDB
        options.increase_parallelism(12);
        options.optimize_level_style_compaction(512 * 1024 * 1024);

        //Create the database if not already present
        options.create_if_missing(true);

        //Open the database
        let mut database = Database {
            db: DB::open(&options, &addr).unwrap(),
            path: addr,
            tree: KDTree::new(),
        };

        // Build the KD-Tree
        let iter = database.db.iterator(IteratorMode::Start); //iterates from the start
        println!("Iterating over database...");
        for item in iter {
            let (key, value) = item.unwrap();
            let hex_strings: Vec<String> = key.iter().map(|b| format!("{:02x}", b)).collect();
            let result = hex_strings.join("");
            let vec = deserialize(&value);
            database.tree.add_node((result, vec.vector.vector), 0);
        }

        database.tree.print_tree_for_debug();

        return Ok(database);
    }

    pub fn get_current_path(&self) -> String {
        return self.path.clone();
    }

    pub fn insert_in_database(&mut self, data: Data) -> Result<String, err> {
        let value = serialize(data.clone());
        let mut hasher = Sha256::new();
        hasher.update(&value);
        let key = hasher.finalize();
        let key_string = format!("{:x}", key);
        match self.db.put(&key, value.as_ref() as &[u8]) {
            Ok(_) => {
                self.tree.add_node((key_string.clone(),data.vector.vector), 0);
                return Ok(key_string);
            }
            Err(e) => {
                return Err(e);
            }
        };
    }

    pub fn delete_database(&self) -> Result<(), err> {
        let options = Options::default();
        match DB::destroy(&options, &self.path) {
            Ok(()) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn delete_from_database_with_value(&mut self, data: Data) -> Result<Option<()>, err> {
        let value = serialize(data.clone());
        let mut hasher = Sha256::new();
        hasher.update(&value);
        let key = hasher.finalize();
        let key_string = format!("{:x}", key);

        match self.db.get(&key) {
            Ok(Some(_)) => {
                self.tree.delete_node(key_string);
                match self.db.delete(key) {
                    Ok(_) => Ok(Some(())),
                    Err(e) => Err(e),
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }

        // Find the corresponding point and remove it from the database
    }

    pub fn delete_from_database_with_key(
        &mut self,
        input: &str,
    ) -> Result<Result<Option<()>, err>, hexerr> {
        match decode(input) {
            Ok(bytes) => {
                let key = bytes.into_boxed_slice();
                match self.db.get(&key) {
                    Ok(Some(_)) => {
                        self.tree.delete_node(input.to_string());
                        match self.db.delete(key) {
                            Ok(_) => Ok(Ok(Some(()))),
                            Err(e) => Ok(Err(e)),
                        }
                    }
                    Ok(None) => Ok(Ok(None)),
                    Err(e) => Ok(Err(e)),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_data_from_key(&self, input: &str) -> Result<Result<Option<Data>, err>, hexerr> {
        match decode(input) {
            Ok(bytes) => {
                let key = bytes.into_boxed_slice();
                match self.db.get(&key) {
                    Ok(Some(value)) => {
                        let vec = deserialize(&value);
                        return Ok(Ok(Some(vec)));
                    }
                    Ok(None) => {
                        return Ok(Ok(None));
                    }
                    Err(e) => {
                        return Ok(Err(e));
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
