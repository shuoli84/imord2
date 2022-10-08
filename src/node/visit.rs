/// visit node recursively, useful when need to investigate tree inner structure
/// e.g: for debug output of tree
use super::node::Node;

pub struct NodeProxy<'a, K, V> {
    pub key_values: &'a [(K, V)],
}

pub struct VisitStack<'a, K, V> {
    pub node: NodeProxy<'a, K, V>,
    pub depth: usize,
    pub is_leaf: bool,
    pub stacks: Vec<(NodeProxy<'a, K, V>, usize)>,
}

pub(crate) fn visit_node<K, V>(
    node: &Node<K, V>,
    visit_fn: &mut impl FnMut(&VisitStack<'_, K, V>),
) {
    visit_node_inner(node, visit_fn, 0, vec![]);
}

fn visit_node_inner<'a, K, V>(
    node: &'a Node<K, V>,
    visit_fn: &mut impl FnMut(&VisitStack<'_, K, V>),
    depth: usize,
    stacks: Vec<(NodeProxy<'a, K, V>, usize)>,
) -> Vec<(NodeProxy<'a, K, V>, usize)> {
    let proxy = NodeProxy {
        key_values: &node.key_values,
    };
    let visit_stack = VisitStack {
        node: proxy,
        depth,
        is_leaf: node.is_leaf(),
        stacks,
    };
    visit_fn(&visit_stack);

    let VisitStack {
        node: mut node_proxy,
        mut stacks,
        ..
    } = visit_stack;
    {
        for (idx, child) in node.children.iter().enumerate() {
            stacks.push((node_proxy, idx));
            stacks = visit_node_inner(child, visit_fn, depth + 1, stacks);
            node_proxy = stacks.pop().unwrap().0;
        }
    }
    stacks
}
