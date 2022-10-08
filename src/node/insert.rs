use super::node::Node;
use crate::BTreeConfig;
use std::sync::Arc;

pub enum InsertResult<K, V> {
    Splited {
        new_k_v: (K, V),
        new_l: Arc<Node<K, V>>,
        new_r: Arc<Node<K, V>>,
    },
    NotSplited,
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    pub fn insert(&mut self, key: K, value: V, config: &BTreeConfig) -> InsertResult<K, V> {
        if self.is_leaf() {
            self.key_values.push((key, value));
            self.key_values.sort_by(|l, r| l.0.cmp(&r.0));
            self.count += 1;
        } else {
            match self.key_values.binary_search_by(|(k, _v)| k.cmp(&key)) {
                Ok(idx) => {
                    // we are the node
                    self.key_values[idx] = (key, value);
                }
                Err(idx) => {
                    // we should insert at child at idx
                    let child = Arc::make_mut(&mut self.children[idx]);
                    match child.insert(key, value, config) {
                        InsertResult::NotSplited => {
                            self.count += 1;
                            return InsertResult::NotSplited;
                        }
                        InsertResult::Splited {
                            new_k_v,
                            new_l,
                            new_r,
                        } => {
                            self.count += 1;
                            self.key_values.insert(idx, new_k_v);
                            self.children[idx] = new_l;
                            self.children.insert(idx + 1, new_r);
                        }
                    }
                }
            }
        }

        if !config.node_should_split(self.key_values.len()) {
            return InsertResult::NotSplited;
        }

        let split_at = self.key_values.len() / 2;
        let split_off = split_at + 1;

        let mut left_key_values = std::mem::replace(&mut self.key_values, vec![]);

        let right_key_values = left_key_values.split_off(split_off);
        let root_key_value = left_key_values.pop().unwrap();

        let (left_children, right_children) = if !self.is_leaf() {
            let mut left_children = std::mem::replace(&mut self.children, vec![]);
            let right_children = left_children.split_off(split_off);
            (left_children, right_children)
        } else {
            (vec![], vec![])
        };

        let new_l = Arc::new(Self::new_with_key_values(left_key_values, left_children));
        let new_r = Arc::new(Self::new_with_key_values(right_key_values, right_children));

        InsertResult::Splited {
            new_k_v: root_key_value,
            new_l,
            new_r,
        }
    }
}