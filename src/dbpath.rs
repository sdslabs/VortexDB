use dotenv::dotenv;
use rocksdb::{Options, DB};
use std::fs::OpenOptions;
use std::{collections::HashMap, env, io, io::Write, path::Path};

pub fn check_path(file_path: &String) -> bool {
    dotenv().ok();
    let path = Path::new(file_path);
    return path.exists();
}

pub fn check_database(file_path: &String) -> bool {
    dotenv().ok();
    let options = Options::default();

    match DB::open_for_read_only(&options, file_path, false) {
        Ok(_) => {
            return true;
        }
        Err(_) => {
            return false;
        }
    }
}

pub fn find_databases() -> HashMap<String, String> {
    let mut collections = HashMap::new();
    dotenv().ok();
    let count: u32 = env::var("NUMBER_OF_DATABASE").unwrap().parse().unwrap();
    for cnt in 1..count + 1 {
        let temp: String = cnt.to_string();
        let mut db_path_var = "DATABASE_PATH".to_string();
        let mut db_name_var = "DATABASE_NAME".to_string();
        db_path_var.push_str(&temp);
        db_name_var.push_str(&temp);

        collections.insert(
            env::var(db_name_var).unwrap(),
            env::var(db_path_var).unwrap(),
        );
    }
    return collections;
}

pub fn write_env(databases: &HashMap<String, String>) {
    let file_path = ".env";
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .unwrap();

    let mut buffered_file = io::BufWriter::new(file);

    let env_string = format!("NUMBER_OF_DATABASE={}\n", databases.len());
    buffered_file.write_all(env_string.as_bytes()).unwrap();

    let mut count = 0;

    for (key, value) in databases.iter() {
        count += 1;
        let temp: String = count.to_string();
        let mut db_path_var = "DATABASE_PATH".to_string();
        let mut db_name_var = "DATABASE_NAME".to_string();
        db_path_var.push_str(&temp);
        db_name_var.push_str(&temp);
        let env_string_path = format!("{}={}\n", db_path_var, value.clone());
        let env_string_name = format!("{}={}\n", db_name_var, key.clone());
        buffered_file.write_all(env_string_path.as_bytes()).unwrap();
        buffered_file.write_all(env_string_name.as_bytes()).unwrap();
    }
}
