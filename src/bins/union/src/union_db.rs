use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Debug)]
pub struct UnionDB<V: Hash + Eq> {
    pub path: PathBuf,
    // im using a vec instad of a map bc iterating over the vec is faster than hashing V in most cases
    map: DashMap<V, Index<V>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Index<V> {
    pub mentions: u32,
    pub relations: Vec<Relation<V>>,
}

impl<V> Default for Index<V> {
    fn default() -> Self {
        Self {
            mentions: Default::default(),
            relations: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Relation<V> {
    pub val: V,
    pub weight: u32,
}

impl<V: Hash + Eq + Clone> UnionDB<V> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            map: DashMap::new(),
            path,
        }
    }

    pub fn increase_weight(&self, val_1: V, val_2: V, weight_increase: u8) {
        fn find_or_create_relation<V: Eq>(
            relations: &mut Vec<Relation<V>>,
            val: V,
        ) -> &mut Relation<V> {
            let index = relations.iter().position(|rel| rel.val == val);
            match index {
                Some(i) => &mut relations[i],
                None => {
                    relations.push(Relation { val, weight: 0 });
                    relations.last_mut().unwrap()
                }
            }
        }

        {
            let val_1_relations = &mut self.map.entry(val_1.to_owned()).or_default().relations;
            let relation_1_to_2 = find_or_create_relation(val_1_relations, val_2.to_owned());
            relation_1_to_2.weight += weight_increase as u32;
        }

        {
            let val_2_relations = &mut self.map.entry(val_2).or_default().relations;
            let relation_2_to_1 = find_or_create_relation(val_2_relations, val_1);
            relation_2_to_1.weight += weight_increase as u32;
        }
    }

    pub fn insert_buf(&self, buf: &[V], weight_increase: u8, window_size: usize) {
        for window in buf.windows(window_size) {
            for (i, val_1) in window.iter().enumerate() {
                for val_2 in window.iter().skip(i + 1) {
                    self.increase_weight(val_1.to_owned(), val_2.to_owned(), weight_increase);
                }
            }
        }

        for val in buf {
            if let Some(mut e) = self.map.get_mut(val) {
                let index = e.value_mut();
                index.mentions += 1;
            };
        }
    }

    pub fn extract_relations(&self, val: &V, relation_amount: usize) -> Index<V> {
        let index = self.map.get(val);
        let index = match index {
            Some(e) => e.value().to_owned(),
            None => Index::default(),
        };

        let rels = {
            let mut rels = index.relations;
            rels.sort_by(|a, b| b.weight.cmp(&a.weight));
            rels.truncate(relation_amount);
            rels
        };

        Index {
            mentions: index.mentions,
            relations: rels,
        }
    }

    pub fn extract_relations_recursive(&self, val: V, max_depth: u8) -> RRNode<V> {
        let mut current_path = Vec::new();
        self.extract_recursive_inner(val, 0, max_depth, &mut current_path)
    }

    fn extract_recursive_inner(
        &self,
        val: V,
        weight: u32,
        depth: u8,
        path: &mut Vec<V>,
    ) -> RRNode<V> {
        if depth == 0 || path.contains(&val) {
            return RRNode {
                val,
                weight,
                children: Box::new([]),
            };
        }
        path.push(val.clone());

        let rels = self.extract_relations(&val, 10);
        let children = rels
            .into_vec() // Convert Box<[T]> to Vec<T> for mapping
            .into_iter()
            .map(|rel| self.extract_recursive_inner(rel.val, rel.weight, depth - 1, path))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        path.pop();

        RRNode {
            val,
            weight,
            children,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RRNode<V> {
    pub val: V,
    pub weight: u32,
    pub children: Box<[RRNode<V>]>,
}
