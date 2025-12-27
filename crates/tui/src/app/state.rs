use defs::{DenseVector, Payload};
use uuid::Uuid;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum AppState {
    #[default]
    Dashboard,
    Database,
    VectorOperations,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalType {
    CreateDatabase,
    DeleteDatabase,
    ConfirmDeleteDatabase,
    DatabaseList,
    ListVectors,
    VectorDetails,
    Error,
    Success,
    Failure,

    DeleteVector,
    SearchSimilarVectors,
    TextEmbedding,
    ImageEmbedding,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VectorListItem {
    pub id: Uuid,
    pub vector: DenseVector,
    pub payload: Option<Payload>,
}
