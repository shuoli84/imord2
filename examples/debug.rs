use imord2::{BTree, BTreeConfig};

fn main() {
    let mut btree = BTree::<i32, ()>::new_with_config(BTreeConfig { max_degree: 10 });
    let keys = (0..100).rev().collect::<Vec<_>>();
    for i in keys {
        btree.insert(i, ());
    }

    btree.visit(&mut |node| {
        let prefix = "  ".repeat(node.depth);
        println!(
            "{prefix}{:?}",
            node.key_values.iter().map(|(k, _)| k).collect::<Vec<_>>()
        );
    });
}
