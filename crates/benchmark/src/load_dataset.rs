use crate::types::{Dataset, DatasetType, GroundTruth};
use defs::IndexedVector;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const MAX_DATA_POINTS: u128 = 50000;

pub fn load_ground_truth(set: &mut Dataset, dataset: String, dataset_type: DatasetType) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let base_path = PathBuf::from(manifest_dir).join("Datasets");

    let path: PathBuf = match (&dataset_type, dataset.as_str()) {
        (DatasetType::DataSet, "1M") => base_path.join("sift/sift_base.fvecs"),
        (DatasetType::DataSet, _) => base_path.join("siftsmall/siftsmall_base.fvecs"),
        (DatasetType::TestQueries, "1M") => base_path.join("sift/sift_query.fvecs"),
        (DatasetType::TestQueries, _) => base_path.join("siftsmall/siftsmall_query.fvecs"),
        (DatasetType::GroundTruth, "1M") => base_path.join("sift/sift_groundtruth.ivecs"),
        (DatasetType::GroundTruth, _) => base_path.join("siftsmall/siftsmall_groundtruth.ivecs"),
    };
    let data = fs::read(path).unwrap();

    let mut data_iter = data.iter();

    let mut point_id: u128 = 0;
    loop {
        if point_id > MAX_DATA_POINTS {
            break;
        }

        let Some(first) = data_iter.next() else {
            break;
        };

        let dim: u32 = (*first as u32)
            | ((*data_iter.next().unwrap() as u32) << 8)
            | ((*data_iter.next().unwrap() as u32) << 16)
            | ((*data_iter.next().unwrap() as u32) << 24);

        let mut one_vector: GroundTruth = GroundTruth {
            id: Uuid::from_bytes(point_id.to_be_bytes()),
            vector: Vec::with_capacity(set.dimension),
        };
        for _i in 0..dim {
            let int_val: u128 = (*data_iter.next().unwrap() as u128)
                | ((*data_iter.next().unwrap() as u128) << 8)
                | ((*data_iter.next().unwrap() as u128) << 16)
                | ((*data_iter.next().unwrap() as u128) << 24);
            one_vector
                .vector
                .push(Uuid::from_bytes(int_val.to_be_bytes()));
        }
        if let DatasetType::GroundTruth = dataset_type {
            set.ground_truth.push(one_vector);
        }
        point_id += 1;
    }
}

pub fn load_dataset_and_test_query(set: &mut Dataset, dataset: String, dataset_type: DatasetType) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let base_path = PathBuf::from(manifest_dir).join("Datasets");

    let path: PathBuf = match (&dataset_type, dataset.as_str()) {
        (DatasetType::DataSet, "1M") => base_path.join("sift/sift_base.fvecs"),
        (DatasetType::DataSet, _) => base_path.join("siftsmall/siftsmall_base.fvecs"),
        (DatasetType::TestQueries, "1M") => base_path.join("sift/sift_query.fvecs"),
        (DatasetType::TestQueries, _) => base_path.join("siftsmall/siftsmall_query.fvecs"),
        (DatasetType::GroundTruth, "1M") => base_path.join("sift/sift_groundtruth.ivecs"),
        (DatasetType::GroundTruth, _) => base_path.join("siftsmall/siftsmall_groundtruth.ivecs"),
    };

    let data = fs::read(path).unwrap();

    let mut data_iter = data.iter();

    let mut point_id: u128 = 0;
    loop {
        if point_id > MAX_DATA_POINTS {
            break;
        }
        let Some(first) = data_iter.next() else {
            break;
        };

        let dim: u32 = (*first as u32)
            | ((*data_iter.next().unwrap() as u32) << 8)
            | ((*data_iter.next().unwrap() as u32) << 16)
            | ((*data_iter.next().unwrap() as u32) << 24);

        let mut one_vector: IndexedVector = IndexedVector {
            id: Uuid::from_bytes(point_id.to_be_bytes()),
            vector: Vec::with_capacity(set.dimension),
        };
        for _i in 0..dim {
            let int_val: u32 = (*data_iter.next().unwrap() as u32)
                | ((*data_iter.next().unwrap() as u32) << 8)
                | ((*data_iter.next().unwrap() as u32) << 16)
                | ((*data_iter.next().unwrap() as u32) << 24);
            one_vector.vector.push(f32::from_bits(int_val));
        }
        match dataset_type {
            DatasetType::DataSet => {
                set.data.push(one_vector);
            }
            DatasetType::TestQueries => {
                set.test_queries.push(one_vector);
            }
            _ => (),
        }
        point_id += 1;
    }
}
