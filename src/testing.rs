//Need to add test cases for finding the k-nearest neigbors in all distances

#[cfg(test)]
mod tests {
    use kd_tree::KDTree;
    use types::{Data, DataType, VectorData};
    use rand::prelude::*;
    use crate::{kd_tree, types};

    #[test]
    fn test_text() {
        let vector = VectorData {
            vector: vec![1.0, 2.0, 3.0],
            embedding_type: String::from("text"),
        };
        let data = Data {
            vector: vector,
            payload: String::from("Hello"),
            data_type: DataType::Text,
        };
        assert_eq!(data.payload, "Hello");
    }
    #[test]
    fn test_image() {
        let vector = VectorData {
            vector: vec![1.0, 2.0, 3.0],
            embedding_type: String::from("image"),
        };
        let data = Data {
            vector: vector,
            payload: String::from("Hello"),
            data_type: DataType::Image,
        };
        assert_eq!(data.payload, "Hello");
    }
    #[test]
    fn test_blob() {
        let vector = VectorData {
            vector: vec![1.0, 2.0, 3.0],
            embedding_type: String::from("binary"),
        };
        let data = Data {
            vector: vector,
            payload: String::from("Hello"),
            data_type: DataType::Blob,
        };
        assert_eq!(data.payload, "Hello");
    }
    #[test]
    fn test_audio() {
        let vector = VectorData {
            vector: vec![1.0, 2.0, 3.0],
            embedding_type: String::from("audio"),
        };
        let data = Data {
            vector: vector,
            payload: String::from("Hello"),
            data_type: DataType::Audio,
        };
        assert_eq!(data.payload, "Hello");
    }

    fn random_data(dim: usize) -> Data {
        let mut rng = rand::thread_rng();
        let mut randoms: Vec<f32> = Vec::new();
        for _ in 0..dim {
            randoms.push(rng.gen());
        }
        let vector = VectorData {
            vector: randoms,
            embedding_type: String::from("text"),
        };
        let data = Data {
            vector: vector,
            payload: String::from("Hello"),
            data_type: DataType::Text,
        };
        return data;
    }

    #[test]
    #[should_panic]
    fn test_input_incorrect_vector() {
        let mut tree = KDTree::new();
        let data = random_data(2);
        tree.add_node((String::from("test"), data.vector.vector), 0);
        let data = random_data(3);
        tree.add_node((String::from("test"), data.vector.vector), 0);
    }

    #[test]
    fn rebuild_check() {
        let mut tree = KDTree::new();
        tree.dim = 3;
        for _ in 0..12{
            let data = random_data(3);
            tree.add_node((String::from("test"), data.vector.vector), 0);
        }
        tree.print_tree_for_debug();
        assert_eq!(tree._internals.rebuild_counter, 2);
    }
}
