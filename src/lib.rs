use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct BTreeConfig {
    max_degree: usize,
}

impl BTreeConfig {
    pub fn node_max_children(&self) -> usize {
        self.max_degree
    }

    pub fn node_min_children(&self) -> usize {
        self.max_degree / 2 + self.max_degree % 2
    }

    pub fn node_max_key_value(&self) -> usize {
        self.node_max_children() - 1
    }

    pub fn node_min_key_value(&self) -> usize {
        self.node_min_children() - 1
    }

    pub fn node_under_size(&self, key_value_count: usize) -> bool {
        key_value_count < self.node_min_key_value()
    }

    /// node is already at min size, which means it can't lend to other
    pub fn node_at_min_size(&self, key_value_count: usize) -> bool {
        key_value_count <= self.node_min_key_value()
    }

    pub fn node_should_split(&self, key_value_count: usize) -> bool {
        key_value_count > self.node_max_key_value()
    }
}

pub struct BTree<K, V> {
    root: Option<Arc<Node<K, V>>>,
    config: BTreeConfig,
}

impl<K: Debug, V: Debug> Debug for BTree<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BTree").field("root", &self.root).finish()
    }
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    pub fn new(config: BTreeConfig) -> Self {
        Self { root: None, config }
    }

    /// insert key value into map
    pub fn insert(&mut self, key: K, value: V) {
        let new_root = match self.root.as_mut() {
            Some(root) => {
                let root = Arc::make_mut(root);
                match root.insert(key, value, &self.config) {
                    InsertResult::Splited {
                        new_k_v,
                        new_l,
                        new_r,
                    } => {
                        // root node splitted, make a new node
                        Node::new_with_key_values(vec![new_k_v], vec![new_l, new_r])
                    }
                    InsertResult::NotSplited => {
                        return;
                    }
                }
            }
            None => Node::new_with_key_values(vec![(key, value)], vec![]),
        };
        self.root = Some(Arc::new(new_root));
    }

    /// delete by key
    pub fn delete_by_key(&mut self, key: &K) -> Option<(K, V)> {
        let root = Arc::make_mut(self.root.as_mut()?);
        let delete_result = root.delete_by_key(key, &self.config);

        if root.count == 0 {
            self.root = None
        } else if root.key_values.len() == 0 {
            // if root node key_value is empty, promote its child as new root
            self.root = Some(root.children.remove(0))
        }

        delete_result
    }

    /// get value by key
    pub fn get_by_key(&self, key: &K) -> Option<&V> {
        self.root.as_ref()?.get_by_key(key)
    }

    /// get key, value by offset
    ///
    /// # Examples
    /// ```
    /// use imord2::BTree;
    ///
    /// let mut tree = BTree::<&'static str, i32>::new(4);
    /// tree.insert("a", 1);
    /// tree.insert("b", 2);
    /// assert_eq!(tree.get_by_offset(0).unwrap().0, "a");
    /// assert_eq!(tree.get_by_offset(1).unwrap().0, "b");
    /// ```
    pub fn get_by_offset(&self, offset: usize) -> Option<&(K, V)> {
        self.root.as_ref()?.get_by_offset(offset)
    }
}

pub enum VisitResult {
    Continue,
    Break,
}

/// Node is the tree node, root, branch and leaf node are same
pub struct Node<K, V> {
    key_values: Vec<(K, V)>,
    children: Vec<Arc<Node<K, V>>>,
    count: usize,
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

pub enum InsertResult<K, V> {
    Splited {
        new_k_v: (K, V),
        new_l: Arc<Node<K, V>>,
        new_r: Arc<Node<K, V>>,
    },
    NotSplited,
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    pub fn new() -> Self {
        Self {
            key_values: vec![],
            children: vec![],
            count: 0,
        }
    }

    fn new_with_key_values(key_values: Vec<(K, V)>, children: Vec<Arc<Self>>) -> Self {
        let count = key_values.len() + children.iter().fold(0, |a, c| a + c.count);
        Self {
            key_values,
            children,
            count,
        }
    }

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

mod find;
pub use find::*;
mod delete;
pub use delete::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tree_insert() {
        let mut tree = BTree::<i32, i32>::new(BTreeConfig { max_degree: 4 });
        let keys = (1..13i32).rev().collect::<Vec<_>>();
        for i in keys.iter() {
            tree.insert(*i, i * 100);
            assert_eq!(*tree.get_by_key(i).unwrap(), i * 100);
        }
    }

    #[test]
    fn test_tree_delete() {
        let mut tree = BTree::<i32, u32>::new(BTreeConfig { max_degree: 4 });
        let keys = (1..13i32).collect::<Vec<_>>();
        for i in keys.iter() {
            tree.insert(*i, (i * 100) as u32);
        }

        dbg!(&tree);

        let mut key_values = vec![];
        for i in keys.iter() {
            key_values.push(tree.delete_by_key(i).unwrap());
            dbg!(&tree);
        }

        assert_eq!(key_values.len(), keys.len());
    }

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
