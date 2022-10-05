use std::sync::Arc;

use crate::BTreeConfig;

use super::Node;

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    pub fn delete_by_key(&mut self, key: &K, config: &BTreeConfig) -> Option<(K, V)> {
        if self.is_leaf() {
            match self.key_values.binary_search_by(|(k, _)| k.cmp(key)) {
                Ok(idx) => {
                    self.count -= 1;
                    Some(self.key_values.remove(idx))
                }
                Err(_) => None,
            }
        } else {
            match self.key_values.binary_search_by(|(k, _)| k.cmp(key)) {
                Ok(idx) => {
                    // find the left most large key, replace it here
                    let child = Arc::make_mut(&mut self.children[idx]);
                    let left_most_large_key = child.take_right_most(config);
                    let prev_key_value =
                        std::mem::replace(&mut self.key_values[idx], left_most_large_key);

                    self.rebalance(idx, config);

                    Some(prev_key_value)
                }
                Err(idx) => {
                    let child = Arc::make_mut(&mut self.children[idx]);
                    let deleted_k_v = child.delete_by_key(key, config)?;
                    self.count -= 1;
                    self.rebalance(idx, config);
                    Some(deleted_k_v)
                }
            }
        }
    }

    fn take_right_most(&mut self, config: &BTreeConfig) -> (K, V) {
        if self.is_leaf() {
            // shrink is processed at parent. At leaf, just delete and return
            self.count -= 1;
            return self.key_values.pop().unwrap();
        }

        let child_idx = self.children.len() - 1;
        let right_most_child = Arc::make_mut(self.children.last_mut().unwrap());
        let right_most = right_most_child.take_right_most(config);

        self.rebalance(child_idx, config);

        right_most
    }

    /// For non-leaf node, need to rebalance the tree after deletion
    /// the child_idx and child pointer, is the child which caused this
    /// rebalance
    fn rebalance(&mut self, child_idx: usize, config: &BTreeConfig) {
        let child = &self.children[child_idx];
        let child_is_leaf = child.is_leaf();
        let last_child_idx = self.children.len() - 1;

        let (left_child, right_child, key_value_idx) = if child_idx == last_child_idx {
            (&self.children[child_idx - 1], child, child_idx - 1)
        } else {
            (child, &self.children[child_idx + 1], child_idx)
        };

        if config.node_at_min_size(left_child.key_values.len())
            && config.node_at_min_size(right_child.key_values.len())
        {
            // merge two children
            let mut new_child_key_values = left_child.key_values.clone();
            new_child_key_values.push(self.key_values.remove(key_value_idx));
            new_child_key_values.extend(right_child.key_values.clone());

            let mut new_children = left_child.children.clone();
            new_children.extend(right_child.children.clone());

            // use new_child to replace prev two children
            let new_child = Self::new_with_key_values(new_child_key_values, new_children);
            self.children[key_value_idx] = Arc::new(new_child);
            self.children.remove(key_value_idx + 1);
        } else if config.node_under_size(left_child.key_values.len()) {
            // borrow from right, aka: rotate left
            let mut new_left_key_values = left_child.key_values.clone();
            new_left_key_values.push(self.key_values[key_value_idx].clone());
            let new_right_key_values = right_child.key_values[1..].to_vec();

            let (new_left_children, new_right_children) = if !child_is_leaf {
                let mut new_left_children = left_child.children.clone();
                let right_first_child = right_child.children[0].clone();
                new_left_children.push(right_first_child);

                let new_right_children = right_child.children[1..].to_vec();

                (new_left_children, new_right_children)
            } else {
                (vec![], vec![])
            };

            let new_left_child = Self::new_with_key_values(new_left_key_values, new_left_children);

            let new_right_child =
                Self::new_with_key_values(new_right_key_values, new_right_children);

            let key_value = right_child.key_values[0].clone();

            self.children[key_value_idx] = Arc::new(new_left_child);
            self.children[key_value_idx + 1] = Arc::new(new_right_child);
            self.key_values[key_value_idx] = key_value;
        } else if config.node_under_size(right_child.key_values.len()) {
            // borrow from left, aka: rotate right

            let mut new_left_key_values = left_child.key_values.clone();
            let new_root_key_value = new_left_key_values.pop().unwrap();
            let prev_key_value =
                std::mem::replace(&mut self.key_values[key_value_idx], new_root_key_value);

            let mut new_right_key_values = vec![prev_key_value];
            new_right_key_values.extend(right_child.key_values.clone());

            let (new_left_children, new_right_children) = if !child_is_leaf {
                let new_left_children =
                    left_child.children[0..left_child.children.len() - 1].to_vec();

                let mut new_right_children = vec![left_child.children.last().unwrap().clone()];
                new_right_children.extend(right_child.children.clone());
                (new_left_children, new_right_children)
            } else {
                (vec![], vec![])
            };

            let new_left = Self::new_with_key_values(new_left_key_values, new_left_children);
            let new_right = Self::new_with_key_values(new_right_key_values, new_right_children);
            self.children[key_value_idx] = Arc::new(new_left);
            self.children[key_value_idx + 1] = Arc::new(new_right);
        }
    }
}
