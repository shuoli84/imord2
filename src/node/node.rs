use std::fmt::Debug;
use std::sync::Arc;

/// Node is the tree node, root, branch and leaf node are same
pub struct Node<K, V> {
    pub(crate) key_values: Vec<(K, V)>,
    pub(crate) children: Vec<Arc<Node<K, V>>>,
    pub(crate) count: usize,
}

impl<K: Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("count", &self.count)
            .field("key_values", &self.key_values)
            .field("children", &self.children)
            .finish()
    }
}

impl<K: Clone, V: Clone> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Self {
            key_values: self.key_values.clone(),
            children: self.children.clone(),
            count: self.count,
        }
    }
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            key_values: vec![],
            children: vec![],
            count: 0,
        }
    }

    pub(crate) fn new_with_key_values(key_values: Vec<(K, V)>, children: Vec<Arc<Self>>) -> Self {
        let count = key_values.len() + children.iter().fold(0, |a, c| a + c.count);
        Self {
            key_values,
            children,
            count,
        }
    }

    pub fn get_by_key(&self, key: &K) -> Option<&V> {
        match self.key_values.binary_search_by(|(k, _)| k.cmp(key)) {
            Ok(idx) => Some(&self.key_values[idx].1),
            Err(idx) => {
                if self.is_leaf() {
                    None
                } else {
                    let child = &self.children[idx];
                    child.get_by_key(key)
                }
            }
        }
    }

    /// get k,v at offset
    pub fn get_by_offset(&self, offset: usize) -> Option<&(K, V)> {
        if self.count <= offset {
            return None;
        }

        if self.is_leaf() {
            self.key_values.get(offset)
        } else {
            let mut relative_offset = offset;

            for idx in 0..self.key_values.len() {
                let left_child = &self.children[idx];
                if left_child.count > relative_offset {
                    return left_child.get_by_offset(relative_offset);
                }

                relative_offset -= left_child.count;

                if relative_offset == 0 {
                    return Some(&self.key_values[idx]);
                }

                relative_offset -= 1;
            }

            // check the last child
            let last_child = &self.children.last().unwrap();
            last_child.get_by_offset(relative_offset)
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{node::insert::InsertResult, BTreeConfig};

    #[test]
    fn test_node() {
        let config = BTreeConfig { max_degree: 4 };
        let mut node = Node::<i32, i32>::new();
        let keys = (1..100i32).rev().collect::<Vec<_>>();
        for i in keys.clone() {
            match node.insert(i, i * 100, &config) {
                InsertResult::Splited {
                    new_k_v,
                    new_l,
                    new_r,
                } => {
                    node = Node::new_with_key_values(vec![new_k_v], vec![new_l, new_r]);
                }
                InsertResult::NotSplited => {
                    // do nothing
                }
            }
        }

        for i in keys.iter() {
            assert_eq!(*node.get_by_key(i).unwrap(), i * 100);
        }

        for i in 0..keys.len() {
            node.get_by_offset(i).unwrap();
        }
    }
}
