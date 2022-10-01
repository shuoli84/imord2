use std::fmt::Debug;
use std::sync::Arc;

pub struct BTree<K, V> {
    root: Option<Arc<Node<K, V>>>,
    n: usize,
}

impl<K: Debug, V: Debug> Debug for BTree<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BTree").field("root", &self.root).finish()
    }
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    pub fn new(n: usize) -> Self {
        Self { root: None, n }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let new_root = match self.root.as_mut() {
            Some(root) => {
                let root = Arc::make_mut(root);
                match root.insert(key, value) {
                    InsertResult::Splited {
                        new_k_v,
                        new_l,
                        new_r,
                    } => {
                        // root node splitted, make a new node
                        Node::new_with_key_values(vec![new_k_v], vec![new_l, new_r], self.n)
                    }
                    InsertResult::NotSplited => {
                        return;
                    }
                }
            }
            None => Node::new_with_key_values(vec![(key, value)], vec![], self.n),
        };
        self.root = Some(Arc::new(new_root));
    }
}

/// Node is the tree node, root, branch and leaf node are same
pub struct Node<K, V> {
    key_values: Vec<(K, V)>,
    children: Vec<Arc<Node<K, V>>>,
    n: usize,
}

impl<K: Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("key_values", &self.key_values)
            .field("children", &self.children)
            .field("n", &self.n)
            .finish()
    }
}

impl<K: Clone, V: Clone> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Self {
            key_values: self.key_values.clone(),
            children: self.children.clone(),
            n: self.n,
        }
    }
}

pub enum InsertResult<K, V> {
    Splited {
        new_k_v: (K, V),
        new_l: Arc<Node<K, V>>,
        new_r: Arc<Node<K, V>>,
    },
    NotSplited,
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    pub fn new(n: usize) -> Self {
        Self {
            key_values: vec![],
            children: vec![],
            n,
        }
    }

    fn new_with_key_values(key_values: Vec<(K, V)>, children: Vec<Arc<Self>>, n: usize) -> Self {
        Self {
            key_values,
            children,
            n,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> InsertResult<K, V> {
        if self.is_leaf() {
            self.key_values.push((key, value));
            self.key_values.sort_by(|l, r| l.0.cmp(&r.0));
        } else {
            match self.key_values.binary_search_by(|(k, _v)| k.cmp(&key)) {
                Ok(idx) => {
                    // we are the node
                    self.key_values[idx] = (key, value);
                }
                Err(idx) => {
                    // we should insert at child at idx
                    let child = Arc::make_mut(&mut self.children[idx]);
                    match child.insert(key, value) {
                        InsertResult::NotSplited => {
                            return InsertResult::NotSplited;
                        }
                        InsertResult::Splited {
                            new_k_v,
                            new_l,
                            new_r,
                        } => {
                            self.key_values.insert(idx, new_k_v);
                            self.children[idx] = new_l;
                            self.children.insert(idx + 1, new_r);
                        }
                    }
                }
            }
        }

        if !self.should_split() {
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

        let new_l = Arc::new(Self::new_with_key_values(
            left_key_values,
            left_children,
            self.n,
        ));
        let new_r = Arc::new(Self::new_with_key_values(
            right_key_values,
            right_children,
            self.n,
        ));

        InsertResult::Splited {
            new_k_v: root_key_value,
            new_l,
            new_r,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        for (k, value) in self.key_values.iter() {
            if k.eq(key) {
                return Some(value);
            }
        }

        None
    }

    /// if self.children size larger than n, then split
    fn should_split(&self) -> bool {
        self.key_values.len() >= self.n
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tree() {
        let mut tree = BTree::<i32, i32>::new(4);
        let keys = (1..13i32).rev().collect::<Vec<_>>();
        for i in keys {
            tree.insert(i, i * 100);
        }
    }

    #[test]
    fn test_node() {
        let mut node = Node::<i32, i32>::new(4);
        let keys = (1..100i32).rev().collect::<Vec<_>>();
        for i in keys {
            match node.insert(i, i * 100) {
                InsertResult::Splited {
                    new_k_v,
                    new_l,
                    new_r,
                } => {
                    node = Node::new_with_key_values(vec![new_k_v], vec![new_l, new_r], node.n);
                }
                InsertResult::NotSplited => {
                    // do nothing
                }
            }

            dbg!(&node);
        }
    }
}