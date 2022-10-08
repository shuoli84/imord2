use std::fmt::Debug;
use std::sync::Arc;

pub use node::find::*;
use node::insert::InsertResult;
use node::node::Node;
pub use node::visit;

#[derive(Debug, Clone, Copy)]
pub struct BTreeConfig {
    pub max_degree: usize,
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

impl<K: Ord + Clone, V: Clone> Default for BTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    /// create a new tree with default max_degree
    pub fn new() -> Self {
        Self::new_with_config(BTreeConfig {
            max_degree: std::cmp::max(20, 4096 / std::mem::size_of::<(K, V)>()),
        })
    }

    pub fn new_with_config(config: BTreeConfig) -> Self {
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
    /// let mut tree = BTree::<&'static str, i32>::new();
    /// tree.insert("a", 1);
    /// tree.insert("b", 2);
    /// assert_eq!(tree.get_by_offset(0).unwrap().0, "a");
    /// assert_eq!(tree.get_by_offset(1).unwrap().0, "b");
    /// ```
    pub fn get_by_offset(&self, offset: usize) -> Option<&(K, V)> {
        self.root.as_ref()?.get_by_offset(offset)
    }

    /// visit inner node in Pre order
    pub fn visit(&self, visit_fn: &mut impl FnMut(&visit::VisitStack<K, V>)) -> Option<()> {
        let root = self.root.as_ref()?;
        visit::visit_node(root, visit_fn);
        Some(())
    }
}

mod node;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tree_insert() {
        let mut tree = BTree::<i32, i32>::new_with_config(BTreeConfig { max_degree: 4 });
        let keys = (1..13i32).rev().collect::<Vec<_>>();
        for i in keys.iter() {
            tree.insert(*i, i * 100);
            assert_eq!(*tree.get_by_key(i).unwrap(), i * 100);
        }
    }

    #[test]
    fn test_tree_delete() {
        let mut tree = BTree::<i32, u32>::new_with_config(BTreeConfig { max_degree: 4 });
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
}
