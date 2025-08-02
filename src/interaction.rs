// || shree ganesh ||
use crate::net::{Net, Node, NodeId, Redex, Port};

//main function of this is to set rules for the redexes
// as without the interaction the redex will be just be a static data structure
//
//
// Different pairs of nodes “interact” in different ways:
// Two matching Constructor nodes “annihilate”
// A Constructor and a Duplicator “copy” a structure
// Eraser nodes “delete” their partners
// Reference nodes “instantiate” a function

pub enum InteractionResult{
    Success,
    Error(String),
}

impl std::fmt::Display for InteractionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractionResult::Success => write!(f, "Success"),
            InteractionResult::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

//The workflow of this code:
// Given a redex (pair of node IDs)
// apply the correct interaction rule
// this will mutate the net in place
pub fn apply_interaction(net: &mut Net, redex: Redex) -> InteractionResult {
    let a = redex.a;
    let b = redex.b;


    //cloning node values so we can match on their types without borrowing errors
    let node_a = net.get_node(a).clone();
    let node_b = net.get_node(b).clone();

    //patern matching
    match(node_a, node_b){

        // 1. Annihilation: Two identical Constructor nodes
        (Node::Con {tag: tag_a, ports: ports_a}, Node::Con {tag: tag_b, ports: ports_b}) if tag_a == tag_b => {
            net.connect(ports_a[0]).node, ports_a[0].slot, ports_b[0].node, ports_b[0].slot;
            net.connect(ports_a[1].node, ports_a[1].slot, ports_b[1].node, ports_b[1].slot);

            InteractionResult::Success
        }
        // 2. Commutation: Constructor meets Duplicator
        (Node::Con { tag, ports: [a0, a1] }, Node::Dup { ports: [b0, b1] }) => {
            let dup_l = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let dup_r = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let con_l = net.create_node(Node::Con { tag: *tag, ports: [Port::null(), Port::null()] });
            let con_r = net.create_node(Node::Con { tag: *tag, ports: [Port::null(), Port::null()] });

            net.connect(*a0, 0, dup_l, 0);
            net.connect(*a1, 0, dup_r, 0);

            net.connect(dup_l, 1, con_l, 0);
            net.connect(dup_r, 1, con_r, 1);

            net.connect(con_l, 1, con_r, 0);

            net.connect(*b0, 0, con_l, 1);
            net.connect(*b1, 0, con_r, 1);

            InteractionResult::Success
        }

        (Node::Dup { ports: [a0, a1] }, Node::Con {tag, ports: [b0, b1] }) => {
            let dup_l = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let dup_r = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let con_l = net.create_node(Node::Con { tag: *tag, ports: [Port::null(), Port::null()] });
            let con_r = net.create_node(Node::Con { tag: *tag, ports: [Port::null(), Port::null()] });

            // Connect left and right
            net.connect(*b0, 0, dup_l, 0);
            net.connect(*b1, 0, dup_r, 0);

            net.connect(dup_l, 1, con_l, 0);
            net.connect(dup_r, 1, con_r, 1);

            net.connect(con_l, 1, con_r, 0);

            net.connect(*a0, 0, con_l, 0);
            net.connect(*a1, 0, con_r, 1);


            InteractionResult::Success
        }

        //3. Duplication: Duplicator meets Duplicator
        (Node::Dup { ports: [a0, a1] }, Node::Dup { ports: [b0, b1] }) => {
            // Create four new duplicators
            let d0 = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let d1 = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let d2 = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });
            let d3 = net.create_node(Node::Dup { ports: [Port::null(), Port::null()] });

            //Create new connections
            net.connect(*a0, 0, d0, 0);
            net.connect(*a1, 0, d1, 0);
            net.connect(*b0, 0, d2, 0);
            net.connect(*b1, 0, d3, 0);

            net.connect(d0, 1, d2, 1);
            net.connect(d1, 1, d3, 1);

            InteractionResult::Success
        }

        // 5. Erasure: Constructor meets Eraser
        (Node::Con { ports: [a0, a1], .. }, Node::Era { .. }) => {
            // Connect both children to new erasers
            let era_l = net.create_node(Node::Era { port: Port::null() });
            let era_r = net.create_node(Node::Era { port: Port::null() });
            net.connect(*a0, 0, era_l, 0);
            net.connect(*a1, 0, era_r, 0);
            InteractionResult::Success
        }

        // Erasure: Duplicator meets Eraser
        (Node::Dup { ports: [a0, a1] }, Node::Era { .. }) => {
            // Connect both children to new erasers
            let era_l = net.create_node(Node::Era { port: Port::null() });
            let era_r = net.create_node(Node::Era { port: Port::null() });
            net.connect(*a0, 0, era_l, 0);
            net.connect(*a1, 0, era_r, 0);
            InteractionResult::Success
        }

        // Erasure: Eraser meets Constructor
        (Node::Era { .. }, Node::Con { ports: [b0, b1], .. }) => {
            let era_l = net.create_node(Node::Era { port: Port::null() });
            let era_r = net.create_node(Node::Era { port: Port::null() });
            net.connect(*b0, 0, era_l, 0);
            net.connect(*b1, 0, era_r, 0);
            InteractionResult::Success
        }

        // Erasure: Eraser meets Duplicator
        (Node::Era { .. }, Node::Dup { ports: [b0, b1] }) => {
            let era_l = net.create_node(Node::Era { port: Port::null() });
            let era_r = net.create_node(Node::Era { port: Port::null() });
            net.connect(*b0, 0, era_l, 0);
            net.connect(*b1, 0, era_r, 0);
            InteractionResult::Success
        }

        // Erasure: Eraser meets Eraser (nothing to do)
        (Node::Era { .. }, Node::Era { .. }) => {
            InteractionResult::Success
        }

        //6. Reference node: Instantiate function if possible
        (Node::Ref { name, .. }, other) => {
            if let Some(def_root) = net.get_definition(name) {
                // Clone the function body, connect its root to the other node
                let clone_root = net.clone_subgraph(def_root);
                // Connect the clone to the other node
                net.connect(clone_root, 0, b, 0);
                InteractionResult::Success
            } else {
                InteractionResult::Error(format!("Undefined function: {}", name))
            }
        }

        // Reference node: Instantiate function if possible (symmetric)
        (other, Node::Ref { name, .. }) => {
            if let Some(def_root) = net.get_definition(name) {
                let clone_root = net.clone_subgraph(def_root);
                net.connect(clone_root, 0, a, 0);
                InteractionResult::Success
            } else {
                InteractionResult::Error(format!("Undefined function: {}", name))
            }
        }

        // Fallback: No rule matches
        _ => InteractionResult::Error("Unsupported interaction".to_string()),
    }
}
