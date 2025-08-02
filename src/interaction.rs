use crate::net::{Net, Node, NodeId, Redex, Port};

pub enum InteractionResult{
    Success,
    Error(String),
}

pub fn apply_interaction(net: &mut Net, redex: Redex) -> InteractionResult {

    let node_a = net.get_node(redex.a).clone();
    let node_b = net.get_node(redex.b).clone();

    match(node_a, node_b){
        (Node::Con {tag: tag_a, ports: ports_a}, Node::Con {tag: tag_b, ports: ports_b}) if tag_a == tag_b => {
            net.connect(ports_a[0]).node, ports_a[0].slot, ports_b[0].node, ports_b[0].slot;
            net.connect(ports_a[1].node, ports_a[1].slot, ports_b[1].node, ports_b[1].slot);

            InteractionResult::Success
        }

        (Node::Con { tag, ports: [a0, a1] }, Node::Dup { ports: [b0, b1] }) => {
            InteractionResult::Success
        }

        (Node::Dup { ports: [a0, a1] }, Node::Dup { ports: [b0, b1] }) => {
            InteractionResult::Success
        }

        (Node::Con { ports: [a0, a1], .. }, Node::Era { .. }) => {
            InteractionResult::Success
        }

        _ => InteractionResult::Error("Unsupported interaction".to_string()),
    }
}
