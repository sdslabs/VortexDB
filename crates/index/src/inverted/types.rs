use defs::{Element, PointId};

#[derive(Default, Debug, Clone)]
pub struct PostingList {
    entries: Vec<(PointId, Element)>,
}

impl PostingList {
    pub fn insert(&mut self, point_id: PointId, weight: Element) {
        self.entries.push((point_id, weight));
    }

    pub fn remove(&mut self, point_id: PointId) -> bool {
        match self.entries.iter().position(|&(id, _)| id == point_id) {
            Some(pos) => {
                self.entries.swap_remove(pos);
                true
            }
            None => false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (PointId, Element)> + '_ {
        self.entries.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
