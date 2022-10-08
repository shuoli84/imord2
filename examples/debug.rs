use imord2::{BTree, BTreeConfig};

fn main() {
    let mut btree = BTree::<i32, ()>::new_with_config(BTreeConfig { max_degree: 10 });
    let keys = (0..100).rev().collect::<Vec<_>>();
    for i in keys {
        btree.insert(i, ());
    }

    btree.visit(&mut |node_stack| {
        let node = &node_stack.node;
        let depth = node_stack.depth;
        let is_leaf = node_stack.is_leaf;
        let prefix = "  ".repeat(depth);

        if !is_leaf {
            // let (parent, parent_child_index) = node_stack.stacks.last().unwrap();

            // println!(
            //     "{prefix}{:?}",
            //     node.key_values.iter().map(|(k, _)| k).collect::<Vec<_>>()
            // );
            // if *parent_child_index != parent.key_values.len() {
            //     println!("{prefix}{:?}", parent.key_values[*parent_child_index]);
            // }
        } else {
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

                prev_child_is_last = *child_index == parent.key_values.len();
            }
        }
    });
}
