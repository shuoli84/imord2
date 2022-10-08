use super::node::Node;

pub enum KeyRangeResult<'a, K> {
    None,
    Some { start: &'a K, end: &'a K, n: usize },
}

impl<K: std::fmt::Debug> std::fmt::Debug for KeyRangeResult<'_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Some { start, end, n } => f
                .debug_struct("Some")
                .field("start", start)
                .field("end", end)
                .field("n", n)
                .finish(),
        }
    }
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

    pub fn start_key(&self) -> Option<&K> {
        match self {
            KeyRangeResult::None => None,
            KeyRangeResult::Some { ref start, .. } => Some(start),
        }
    }

    pub fn end_key(&self) -> Option<&K> {
        match self {
            KeyRangeResult::None => None,
            KeyRangeResult::Some { ref end, .. } => Some(end),
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
    /// if true for smaller range, then it must be true for larger range
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

            let mut matched_indexes = vec![];

            for (index, key) in self.key_values.iter().enumerate() {
                match predicate(&key.0) {
                    PredicateResult::Left => {
                        extra_child_to_check = Some(index + 1);
                        continue;
                    }
                    PredicateResult::Match => {
                        matched_indexes.push((index, key));
                        extra_child_to_check = Some(index + 1);
                    }
                    PredicateResult::Right => {
                        extra_child_to_check = None;
                        result = result.merge_into(self.children[index].find_key_range(predicate));
                        break;
                    }
                }
            }

            if !matched_indexes.is_empty() {
                let (first_idx, first_key) = matched_indexes[0];

                // for first match index, visit child to update result
                result = result.merge_into(self.children[first_idx].find_key_range(predicate));
                result = result.merge_into(KeyRangeResult::Some {
                    start: &first_key.0,
                    end: &first_key.0,
                    n: 1,
                });

                if matched_indexes.len() >= 2 {
                    // we do not need to visit children between two matched key
                    let (last_idx, last_key) = matched_indexes[matched_indexes.len() - 1];

                    let mut count = 0;
                    for idx in first_idx + 1..=last_idx {
                        // add children count
                        count += self.children[idx].count;
                        // also add the item itself
                        count += 1;
                    }

                    result = result.merge_into(KeyRangeResult::Some {
                        start: &first_key.0,
                        end: &last_key.0,
                        n: count,
                    });
                } else {
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
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_node_find_key_range() {
        {
            let node =
                Node::<i32, i32>::new_with_key_values(vec![(1, 1), (2, 2), (3, 3), (4, 4)], vec![]);
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
            ));

            let right_child = Arc::new(Node::<i32, i32>::new_with_key_values(
                vec![(10, 1), (20, 2), (30, 3), (40, 4)],
                vec![],
            ));

            let node =
                Node::<i32, i32>::new_with_key_values(vec![(9, 9)], vec![left_child, right_child]);

            let pred = |k: &i32| match *k {
                i32::MIN..=1 => PredicateResult::Left,
                2..=20 => PredicateResult::Match,
                21.. => PredicateResult::Right,
            };

            let find_result = node.find_key_range(&pred);
            assert_eq!(find_result.n(), 6);

            let find_result = node.find_key_range(&|_k| PredicateResult::Match);
            assert_eq!(find_result.n(), 9);
            assert_eq!(*find_result.start_key().unwrap(), 1);
            assert_eq!(*find_result.end_key().unwrap(), 40);
        }
    }
}
