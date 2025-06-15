use crate::db::Database;
use crate::kd_tree::KDTreeNode;
use core::f32;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy)]
pub enum KNNType {
    Euclidean,
    Manhattan,
    Hamming,
    Cosine,
}

pub struct DataHeap {
    key: String,
    distance: f32,
}

// These traits must be implemented for a custom BinaryHeap
impl Eq for DataHeap {}
impl Ord for DataHeap {
    fn cmp(&self, other: &DataHeap) -> Ordering {
        if self.distance < other.distance {
            Ordering::Less
        } else if self.distance > other.distance {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
impl PartialEq for DataHeap {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl PartialOrd for DataHeap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn distance(a: Vec<f32>, b: Vec<f32>, dist_type: KNNType) -> f32 {
    assert_eq!(a.len(), b.len());
    match dist_type {
        KNNType::Euclidean => {
            let score: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| (x - y) * (x - y))
                .collect();
            return score.iter().sum::<f32>().sqrt();
        }
        KNNType::Manhattan => {
            let score: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| (x - y).abs())
                .collect();
            return score.iter().sum::<f32>();
        }
        KNNType::Hamming => {
            let score: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| (if x != y { 1f32 } else { 0f32 }))
                .collect();
            return score.iter().sum::<f32>();
        }
        KNNType::Cosine => {
            let p_score: Vec<f32> = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).collect();
            let p = p_score.iter().sum::<f32>();
            let q_score: Vec<f32> = a.iter().map(|&n| n * n).collect();
            let q = q_score.iter().sum::<f32>().sqrt();
            let r_score: Vec<f32> = b.iter().map(|&n| n * n).collect();
            let r = r_score.iter().sum::<f32>().sqrt();
            return p / (q * r);
        }
    };
}

pub fn get_knn(
    database: &mut Database,
    input: Vec<f32>,
    kvalue: usize,
    knn_type: KNNType,
) -> Vec<String> {
    //Finding the first k elements to insert into the BinaryHeap
    let k_nodes = database.tree.traversal(kvalue);
    let mut insert_heap: BinaryHeap<DataHeap> = BinaryHeap::new();
    for node in &k_nodes {
        insert_heap.push(DataHeap {
            key: node.0.clone(),
            distance: distance(input.clone(), node.1.clone(), knn_type),
        });
    }
    let binding = database.tree._root.as_ref().unwrap();
    let (heap, n_visited) = binding.find_nearest_neighbors(input, knn_type, &mut insert_heap);
    let mut ret_vec: Vec<String> = Vec::new();
    ret_vec.push(format!("Visited {} nodes", n_visited));
    for point in heap.iter() {
        ret_vec.push(point.key.clone());
    }
    return ret_vec;
}

impl KDTreeNode {
    pub fn find_nearest_neighbors<'a>(
        &'a self,
        point: Vec<f32>,
        knn_type: KNNType,
        heap: &'a mut BinaryHeap<DataHeap>,
    ) -> (&'a mut BinaryHeap<DataHeap>, usize) {
        self.find_nearest_neighbor_helper(point, 1, knn_type, heap)
    }

    fn find_nearest_neighbor_helper<'a>(
        &'a self,
        point: Vec<f32>,
        n_visited: usize,
        knn_type: KNNType,
        distances: &'a mut BinaryHeap<DataHeap>,
    ) -> (&'a mut BinaryHeap<DataHeap>, usize) {
        if distances.is_empty() {
            panic!("Empty heap entered!");
        }

        let mut my_n_visited = n_visited;
        let mut my_distances = distances;

        if self.vector[self.dim] < point[self.dim] && self.left.is_some() {
            let (a, b) = self.left.as_ref().unwrap().find_nearest_neighbor_helper(
                point.clone(),
                my_n_visited,
                knn_type,
                my_distances,
            );
            my_distances = a;
            my_n_visited = b;
        }

        // distance along this node's axis
        let axis_dist = distance(point.clone(), self.vector.clone(), knn_type);
        if axis_dist <= my_distances.peek().unwrap().distance {
            // self can only be nearer than worst if axis_dist is less than worst_dist because axis_dist is a lower bound for self_dist
            let self_dist = distance(point.clone(), self.vector.clone(), knn_type.clone());
            if self_dist < my_distances.peek().unwrap().distance {
                my_distances.pop();
                my_distances.push(DataHeap {
                    key: self.key.clone(),
                    distance: self_dist,
                });
            }

            // bookkeeping
            my_n_visited += 1;

            // same reasoning applies for the far side of the split
            if self.vector[self.dim] < point[self.dim] && self.left.is_some() {
                let (a, b) = self.left.as_ref().unwrap().find_nearest_neighbor_helper(
                    point,
                    my_n_visited,
                    knn_type,
                    my_distances,
                );
                my_distances = a;
                my_n_visited = b;
            } else if self.right.is_some() {
                let (a, b) = self.right.as_ref().unwrap().find_nearest_neighbor_helper(
                    point,
                    my_n_visited,
                    knn_type,
                    my_distances,
                );
                my_distances = a;
                my_n_visited = b;
            }
        }

        (my_distances, my_n_visited)
    }
}
