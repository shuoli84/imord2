/// visit node recursively, useful when need to investigate tree inner structure
/// e.g: for debug output of tree
use super::node::Node;

pub struct NodeProxy<'a, K, V> {
    pub key_values: &'a [(K, V)],
    pub depth: usize,
}

pub(crate) fn visit_node<K, V>(node: &Node<K, V>, visit_fn: &mut impl FnMut(NodeProxy<K, V>)) {
    visit_node_inner(node, visit_fn, 0)
}

fn visit_node_inner<K, V>(
    node: &Node<K, V>,
    mut visit_fn: &mut impl FnMut(NodeProxy<'_, K, V>),
    depth: usize,
) {
    let proxy = NodeProxy {
        key_values: &node.key_values,
        depth,
    };
    visit_fn(proxy);
    for child in node.children.iter() {
        visit_node_inner(child, visit_fn, depth + 1);
    }
}
