use crate::db;
use crate::dbpath;
use crate::indexing;
use crate::keygen;
use crate::types;
use crate::vectoriser;

use std::collections::HashMap;
use std::{env, io};
// use std::ops::Deref;
use db::Database;
use dbpath::{check_database, check_path, find_databases, write_env};
use indexing::{get_knn, KNNType};
use keygen::deserialize;
use types::{Data, DataType, VectorData};

use rocksdb::IteratorMode;

pub fn run_cli() {
    println!("Welcome to Vector DB");
    main_menu();
}

fn main_menu() {
    let mut databases = find_databases();

    loop {
        println!("(1) Show databases");
        println!("(2) Use database");
        println!("(3) Add new database");
        println!("(4) Delete database");
        println!("(0) Exit");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        match choice.trim() {
            "1" => {
                show_databases(&mut databases);
            }
            "2" => {
                use_databases(&mut databases);
            }
            "3" => {
                add_databases(&mut databases);
            }
            "4" => {
                delete_databases(&mut databases);
            }
            "0" => {
                break;
            }
            _ => {
                println!("Invalid choice");
            }
        };
    }
}

fn show_databases(databases: &mut HashMap<String, String>) {
    println!("Avaiable databases are...");
    for (key, _) in databases.iter() {
        println!("{}", key);
    }
}

fn use_databases(mut databases: &mut HashMap<String, String>) {
    println!("Enter name of database");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Faile to read line");

    if !databases.contains_key(input.trim()) {
        println!("Database does not exitst");
        return;
    }

    let mut database: Option<Database> = valid_database(&mut databases, input.trim());

    if database.is_none() {
        let exit = change_path(&mut databases, input.trim());
        if exit {
            databases.remove(input.trim());
            return;
        } else {
            database = valid_database(&mut databases, input.trim());
        }
    }

    let database = &mut database.unwrap();
    loop {
        println!("{}", input.trim());
        println!("(1) Insert in Database");
        println!("(2) View Database");
        println!("(3) Get from Database");
        println!("(4) Delete from Database");
        println!("(5) Find K Nearest Neighbours");
        println!("(6) View current path");
        println!("(7) Switch database");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        match choice.trim() {
            "1" => {
                insert_in_database(database);
            }
            "2" => {
                view_database(database);
            }
            "3" => {
                get_from_database(database);
            }
            "4" => {
                delete_from_database(database);
            }
            "5" => {
                find_knn(database);
            }
            "6" => {
                view_current_path(database);
            }
            "7" => {
                break;
            }
            _ => println!("Invalid choice"),
        }
    }
}

fn add_databases(mut databases: &mut HashMap<String, String>) {
    println!("Enter the databse name");
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");
    if databases.contains_key(name.trim()) {
        println!("This name already exists");
        return;
    }
    if !change_path(&mut databases, name.trim()) {
        println!("Created database successfully");
        return;
    }
}

fn delete_databases(mut databases: &mut HashMap<String, String>) {
    println!("Enter the database name");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    if databases.contains_key(input.trim()) {
        let exit = change_path(&mut databases, &input);
        if exit {
            return;
        }

        let database = valid_database(&mut databases, input.trim()).unwrap();
        delete_database(&database);
    } else {
        println!("Database does not exist");
    }
}

fn valid_database(databases: &mut HashMap<String, String>, input: &str) -> Option<Database> {
    let file_path = databases.get(input).unwrap();
    let mut database: Option<Database> = None;
    if check_path(file_path) {
        println!("Path is valid, validating database...");
        if check_database(file_path) {
            println!("Database exists on current path");
            let new_path = (*file_path).clone();
            database = Some(create_switch_database(new_path));
        } else {
            println!("Database does not exist on current path.");
            println!("Creating a new database...");
            let new_path = (*file_path).clone();
            database = Some(create_switch_database(new_path));
        }
    } else {
        println!("Path in .env file is invalid");
    }

    return database;
}

fn change_path(databases: &mut HashMap<String, String>, input: &str) -> bool {
    let mut br = false;

    loop {
        println!("Enter a new path for database");
        println!("(1) Use current working directory");
        println!("(2) Enter custom path");
        println!("(3) Remove database");
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        match choice.trim() {
            "1" => {
                let current_dir = env::current_dir().unwrap();
                let absolute_path = current_dir.canonicalize().unwrap();
                println!("Setting path to: {}", absolute_path.display());
                let absolute_path = absolute_path.as_os_str().to_str().unwrap().to_string();
                databases.insert(input.to_string(), absolute_path.trim().to_string());
                write_env(databases);
                break;
            }
            "2" => {
                println!("Enter path");
                let mut new_path = String::new();
                io::stdin()
                    .read_line(&mut new_path)
                    .expect("Failed to read line");
                if check_path(&new_path.trim().to_string()) {
                    databases.insert(input.to_string(), new_path.trim().to_string());
                    write_env(databases);
                    break;
                } else {
                    println!("Please enter a valid path");
                }
            }
            "3" => {
                br = true;
                break;
            }
            _ => {
                println!("Invalid input");
            }
        };
    }
    return br;
}

fn create_switch_database(addr: String) -> Database {
    match Database::create_switch_database(addr) {
        Ok(database) => {
            return database;
        }
        Err(err) => panic!("Failed to create database as {:?}", err),
    };
}

fn delete_database(database: &Database) {
    println!("Deleting database...");
    match database.delete_database() {
        Ok(()) => println!("Database deleted successfully"),
        Err(e) => println!("{}", e),
    };
}

fn insert_in_database(database: &mut Database) {
    let data: Data;
    match read_data() {
        Some(v) => {
            data = v;
        }
        None => {
            return;
        }
    }
    println!("Inserting into a database...");
    match database.insert_in_database(data) {
        Ok(key) => {
            println!("Inserted with key {}", key);
        }
        Err(e) => println!("{}", e),
    };
}

fn view_database(database: &Database) {
    let iter = database.db.iterator(IteratorMode::Start); //iterates from the start
    println!("Iterating over database...");
    for item in iter {
        let (key, value) = item.unwrap();
        let hex_strings: Vec<String> = key.iter().map(|b| format!("{:02x}", b)).collect();
        let result = hex_strings.join("");
        let vec = deserialize(&value);
        println!("Key: {:?}\nValue: {:?}", result, vec.payload);
    }
}

fn get_from_database(database: &Database) {
    println!("Enter key of data");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read key");
    match database.get_data_from_key(input.trim()) {
        Ok(v) => match v {
            Ok(Some(v)) => println!("{:?}", v.payload),
            Ok(None) => {
                println!("Key not found");
                return;
            }
            Err(e) => {
                println!("{}", e);
                return;
            }
        },
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
}

fn delete_from_database(database: &mut Database) {
    println!("(1) Delete by entering data");
    println!("(2) Delete by entering key");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");
    match choice.trim() {
        "1" => {
            let data: Data;
            match read_data() {
                Some(v) => {
                    data = v;
                }
                None => {
                    return;
                }
            }
            match database.delete_from_database_with_value(data) {
                Ok(v) => match v {
                    Some(_) => {
                        println!("Data deleted successfully");
                        //change this function to delete from data
                        // database.tree.delete_node(input.trim().to_string());
                    }
                    None => println!("Key not found"),
                },
                Err(e) => println!("{}", e),
            };
        }
        "2" => {
            println!("Enter key");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            match database.delete_from_database_with_key(input.trim()) {
                Ok(v) => match v {
                    Ok(Some(_)) => {
                        println!("Data deleted successfully");
                        database.tree.delete_node(input.trim().to_string());
                    }
                    Ok(None) => println!("Key not found"),
                    Err(e) => println!("{}", e),
                },
                Err(e) => println!("{}", e),
            };
        }
        _ => {
            println!("Invalid choice");
            return;
        }
    }
}

fn read_data() -> Option<Data> {
    println!("Enter data type (Text, Image, Audio, Blob): ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    // match statement to match datatype from Datatype
    let datatype = match input.trim() {
        "Text" => DataType::Text,
        "Image" => DataType::Image,
        "Audio" => DataType::Audio,
        "Blob" => DataType::Blob,
        _ => {
            println!("Invalid data type");
            return None;
        }
    };
    println!("Enter payload: ");
    let mut payload = String::new();
    io::stdin()
        .read_line(&mut payload)
        .expect("Failed to read line");

    let mut embedding_type = String::new();
    println!("Enter embedding type: ");
    io::stdin()
        .read_line(&mut embedding_type)
        .expect("Failed to read line");

    let vec = vectoriser::vectorise(&payload, "");

    let data: Data = Data {
        vector: VectorData {
            vector: vec.vector,
            embedding_type: embedding_type,
        },
        payload: payload,
        data_type: datatype,
    };

    println!("Your Data is");
    println!("{:?}", data.payload);
    println!("(1) Confirm");
    println!("(0) Cancel");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    match choice.trim() {
        "1" => {
            return Some(data);
        }
        "0" => {
            return None;
        }
        _ => {
            return None;
        }
    };
}

fn find_knn(database: &mut Database) {
    println!("Please Select input for KNN");
    println!("(1) Enter Data");
    println!("(2) Enter Key");
    let givenvec;
    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");
    match choice.trim() {
        "1" => {
            let mut payload = String::new();
            io::stdin()
                .read_line(&mut payload)
                .expect("Failed to read line");
            let vec = vectoriser::vectorise(&payload, "");
            givenvec = vec.vector;
            println!("{:?}", givenvec)
        }
        "2" => {
            println!("Enter key");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            match database.get_data_from_key(input.trim()) {
                Ok(v) => match v {
                    Ok(Some(v)) => {
                        givenvec = v.vector.vector;
                    }
                    Ok(None) => {
                        println!("Key not found");
                        return;
                    }
                    Err(e) => {
                        println!("{}", e);
                        return;
                    }
                },
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }
        _ => {
            println!("Invalid choice");
            return;
        }
    }

    println!("Please Enter K Value");

    let mut kvalue = String::new();
    io::stdin()
        .read_line(&mut kvalue)
        .expect("Failed to read line");
    let kvalue: usize = kvalue.trim().parse().unwrap();

    loop {
        println!("Please Select method for KNN");
        println!("(1) Euclidean Distance");
        println!("(2) Manhattan Distance");
        println!("(3) Hamming Distance");
        println!("(4) Cosine Similarity");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        match choice.trim() {
            "1" => {
                let result = get_knn(database, givenvec, kvalue, KNNType::Euclidean);
                for r in &result {
                    println!("{}", r);
                }
                break;
            }
            "2" => {
                let result = get_knn(database, givenvec, kvalue, KNNType::Manhattan);
                for r in &result {
                    println!("{}", r);
                }
                break;
            }
            "3" => {
                let result = get_knn(database, givenvec, kvalue, KNNType::Hamming);
                for r in &result {
                    println!("{}", r);
                }
                break;
            }
            "4" => {
                let result = get_knn(database, givenvec, kvalue, KNNType::Cosine);
                for r in &result {
                    println!("{}", r);
                }
                break;
            }
            _ => {
                println!("Invalid choice");
            }
        }
    }
}

fn view_current_path(database: &Database) {
    println!("{}", database.get_current_path());
}
