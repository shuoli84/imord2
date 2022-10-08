use imord2::{BTree, BTreeConfig, PredicateResult};
use std::sync::atomic::AtomicUsize;

static CMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
struct Key {
    value: i32,
}

impl Key {
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
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

    // reset counter
    CMP_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);

    let result_key_range = btree.find_key_range(|k| {
        if *k >= Key::new(300) {
            PredicateResult::Match
        } else {
            PredicateResult::Left
        }
    });

    println!("{:?}", result_key_range);

    println!(
        "get {} result with {} cmp called",
        result_key_range.n(),
        CMP_COUNTER.load(std::sync::atomic::Ordering::Relaxed)
    );
}
