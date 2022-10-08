use imord2::{BTree, BTreeConfig};
use std::sync::atomic::AtomicUsize;

static CMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
struct Key {
    value: i32,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::cmp::Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        CMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.value.cmp(&other.value)
    }
}

fn main() {
    let mut btree = BTree::<Key, ()>::new_with_config(BTreeConfig { max_degree: 20 });
    let keys = (0..2000).rev().collect::<Vec<_>>();
    for i in keys {
        btree.insert(Key { value: i }, ());
    }

    btree.visit(&mut |node_stack| {
        let node = &node_stack.node;
        let depth = node_stack.depth;
        let is_leaf = node_stack.is_leaf;
        let prefix = "  ".repeat(depth);

        if is_leaf {
            println!(
                "{prefix}{:?}",
                node.key_values.iter().map(|(k, _)| k).collect::<Vec<_>>()
            );

            let mut prev_child_is_last = true;
            for (idx, (parent, child_index)) in node_stack.stacks.iter().rev().enumerate() {
                let prefix = "  ".repeat(depth - idx - 1);
                if *child_index != parent.key_values.len() && prev_child_is_last {
                    println!("{prefix}{:?}", parent.key_values[*child_index].0);
                }

                prev_child_is_last = *child_index == parent.key_values.len() && prev_child_is_last;
            }
        }
    });

    println!(
        "{} cmp called",
        CMP_COUNTER.load(std::sync::atomic::Ordering::Relaxed)
    );
}
