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

pub enum VisitResult {
    Continue,
    Break,
}

/// Node is the tree node, root, branch and leaf node are same
pub struct Node<K, V> {
    key_values: Vec<(K, V)>,
    children: Vec<Arc<Node<K, V>>>,
    n: usize,
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
            n: self.n,
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
    pub fn new(n: usize) -> Self {
        Self {
            key_values: vec![],
            children: vec![],
            n,
            count: 0,
        }
    }

    fn new_with_key_values(key_values: Vec<(K, V)>, children: Vec<Arc<Self>>, n: usize) -> Self {
        let count = key_values.len() + children.iter().fold(0, |a, c| a + c.count);
        Self {
            key_values,
            children,
            n,
            count,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> InsertResult<K, V> {
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
                    match child.insert(key, value) {
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

pub enum KeyRangeResult<'a, K> {
    None,
    Some { start: &'a K, end: &'a K, n: usize },
}

impl<K: Ord> KeyRangeResult<'_, K> {
    #[must_use]
    pub fn merge_into(self, other: Self) -> Self {
        match (&self, &other) {
            (Self::None, _) => other,
            (_, Self::None) => self,
            (
                Self::Some {
                    start: self_start,
                    end: self_end,
                    n: self_n,
                },
                Self::Some { start, end, n },
            ) => Self::Some {
                start: std::cmp::min(self_start, start),
                end: std::cmp::max(self_end, end),
                n: self_n + n,
            },
        }
    }

    pub fn n(&self) -> usize {
        match self {
            KeyRangeResult::None => 0,
            KeyRangeResult::Some { n, .. } => *n,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PredicateResult {
    Left,
    Match,
    Right,
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    /// predicate result should be consistent for range
    /// if true for larger range, then it must be true for smaller range
    /// if false for larger range, then it must be false for smaller range
    /// this helps us to visit range with logn
    pub fn find_key_range<P: Fn(&K) -> PredicateResult>(&self, predicate: &P) -> KeyRangeResult<K> {
        if self.is_leaf() {
            let mut key_predicate_iter = self
                .key_values
                .iter()
                .filter(|k| predicate(&k.0) == PredicateResult::Match);
            match key_predicate_iter.next() {
                None => KeyRangeResult::None,
                Some(first_key) => {
                    let start_key = &first_key.0;
                    let mut end_key: &K = start_key;
                    let mut count = 1;

                    while let Some(key) = key_predicate_iter.next() {
                        count += 1;
                        end_key = &key.0;
                    }

                    KeyRangeResult::Some {
                        start: start_key,
                        end: end_key,
                        n: count,
                    }
                }
            }
        } else {
            // for branch,
            // 1. find the first idx with predicate as Equal/Greater, it is the start
            // 2. then find the first idx with predicate as Greater, also need to check last
            //     child

            let mut result = KeyRangeResult::None;
            let mut extra_child_to_check: Option<usize> = None;

            for (index, key) in self.key_values.iter().enumerate() {
                match predicate(&key.0) {
                    PredicateResult::Left => {
                        extra_child_to_check = Some(index + 1);
                        continue;
                    }
                    PredicateResult::Match => {
                        extra_child_to_check = Some(index + 1);
                        result = result.merge_into(self.children[index].find_key_range(predicate));
                        result = result.merge_into(KeyRangeResult::Some {
                            start: &key.0,
                            end: &key.0,
                            n: 1,
                        });
                    }
                    PredicateResult::Right => {
                        extra_child_to_check = None;
                        result = result.merge_into(self.children[index].find_key_range(predicate));
                        break;
                    }
                }
            }
            if let Some(child_idx) = extra_child_to_check {
                result = result.merge_into(self.children[child_idx].find_key_range(predicate));
            }

            result
        }
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
    fn test_node_find_key_range() {
        {
            let node = Node::<i32, i32>::new_with_key_values(
                vec![(1, 1), (2, 2), (3, 3), (4, 4)],
                vec![],
                4,
            );
            {
                let pred = |k: &i32| match *k {
                    i32::MIN..=1 => PredicateResult::Left,
                    2 => PredicateResult::Match,
                    3.. => PredicateResult::Right,
                };
                let find_result = node.find_key_range(&pred);
                assert_eq!(find_result.n(), 1);
            }
            {
                let pred = |k: &i32| match *k {
                    i32::MIN..=1 => PredicateResult::Left,
                    2..=3 => PredicateResult::Match,
                    4.. => PredicateResult::Right,
                };

                let find_result = node.find_key_range(&pred);

                assert_eq!(find_result.n(), 2);
            }
        }

        {
            let left_child = Arc::new(Node::<i32, i32>::new_with_key_values(
                vec![(1, 1), (2, 2), (3, 3), (4, 4)],
                vec![],
                4,
            ));

            let right_child = Arc::new(Node::<i32, i32>::new_with_key_values(
                vec![(10, 1), (20, 2), (30, 3), (40, 4)],
                vec![],
                4,
            ));

            let node = Node::<i32, i32>::new_with_key_values(
                vec![(9, 9)],
                vec![left_child, right_child],
                4,
            );

            let pred = |k: &i32| match *k {
                i32::MIN..=1 => PredicateResult::Left,
                2..=20 => PredicateResult::Match,
                21.. => PredicateResult::Right,
            };

            let find_result = node.find_key_range(&pred);
            assert_eq!(find_result.n(), 6);
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
