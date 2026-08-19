pub const POLL_DURATION: std::time::Duration = std::time::Duration::from_millis(50);

// Set how many vectors to fetch per function call in list_vectors
pub const VECTOR_LIST_LIMIT: usize = 50;

pub const VECTOR_OPERATIONS: &[&str] = &[
    "List All Vectors",
    "Delete Vector",
    "Search Similar Vectors",
    "Insert Text Embedding",
    "Insert Image Embedding",
];

pub const DB_OPERATIONS: &[&str] = &["Create New Database", "Select Database", "Delete Database"];

pub const ENV_TEXT_EMBEDDING_URL: &str = "TEXT_EMBEDDING_URL";
pub const ENV_IMAGE_EMBEDDING_URL: &str = "IMAGE_EMBEDDING_URL";
