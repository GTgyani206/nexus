// ||shree ganesh||
//this code is for creating the graph
use std::collections::{HashMap, VecDeque};

//Using hashmap to store function definition by name
// Using VecDeque for storing active pairs

use std::fmt;
// This imports the formatting trait so we can print debugging information

pub type NodeID = usize; // this is done to uniquely identify nodes in the network and usize is for dynamically match the pointer size

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

//port will be used as a connector between the nodes...
pub struct Port {
    pub node: NodeId,
    pub slot: usize,
}

impl Port {
    // this function is for node creation
    pub fn new(node: NodeId, slot: usize) -> Self {
        Self { node, slot }
    }

    // null function is for creating a special not connected to anything port
    pub fn null() -> Self {
        // MAX means the largest possible number, which we use to mean "not connected" /// need to clarify this part
        Self {
            node: std::usize::MAX,
            slot: 0,
        }
    }

    // checking if the node is not connected to anything
    pub fn is_null(&self) -> bool {
        self.node == std::usize::MAX
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Con { tag: i32, ports: [Port; 2] },
    Dup { ports: [Port; 2] },
    Ref { name: String, port: Port },
    Era { port: Port },
}

#[derive(Debug, Clone, Copy)]
pub struct Redex {
    pub a: NodeId,
    pub b: NodeId,
}

pub struct Net {
    nodes: Vec<Node>,
    active_pairs: VecDeque<(Redex)>,
    definitions: HashMap<String, Node>,
}

impl Net {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            active_pairs: VecDeque::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn create_node(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn get_node(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id]
    }

    pub fn connect(&mut self, a: NodeId, a_slot: usize, b: NodeId, b_slot: usize) {
        match &mut self.node[a] {
            Node::Con { ports, .. } if a_slot < 2 => {
                ports[a_slot] = Port::new(b, b_slot);
            }
            Node::Dup { ports, .. } if a_slot < 2 => {
                ports[a_slot] = Port::new(b, b_slot);
            }

            Node::Ref { ports, .. } if a_slot == 0 => {
                *ports = Port::new(b, b_slot);
            }
            Node::Era { port, .. } if a_slot == 0 => {
                *port = Port::new(b, b_slot);
            }

            _ => panic!("Invalid port slot for node"),
        }

        if self.is_redex(a, b) {
            self.active_pairs.push_back(Redex { a, b });
        }
    }

    fn is_redex(&self, a: NodeId, b: NodeId) -> bool {
        match (&self.nodes[a], &self.nodes[b]) {
            (Node::Con { tag: tag_a, .. }, Node::Con { tag: tag_b, .. }) => tag_a == tag_b,
            (Node::Con { .. }, Node::Dup { .. }) | (Node::Dup { .. }, Node::Con { .. }) => true,
            (Node::Con { .. }, Node::Era { .. }) | (Node::Era { .. }, Node::Con { .. }) => true,

            (Node::Dup { .. }, Node::Dup { .. }) => true,

            (Node::Dup { .. }, Node::Era { .. }) | (Node::Era { .. }, Node::Dup { .. }) => true,
            (Node::Ref { .. }, _) | (_, Node::Ref { .. }) => true,
            _ => false,
        }
    }

    pub fn add_definition(&mut self, name: &str, root: NodeId) {
        self.definitions.insert(name.to_string(), root);
    }

    pub fn get_definition(&self, name: &str) -> Option<NodeId> {
        self.definitions.get(name).copied()
    }

    pub fn next_redex(&mut self) -> Option<Redex> {
        self.active_pairs.pop_front()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn redex_count(&self) -> usize {
        self.active_pairs.len()
    }

    pub fn clone_subgraph(&mut self, root: NodeId) -> NodeId {
        let mut cloned_nodes = HashMap::new();
        self.clone_node(root, &mut cloned_nodes);
    }

    fn clone_node(&mut self, id: NodeId, cloned_nodes: &mut HashMap<NodeId, NodeId>) -> NodeId {
        if let Some(&cloned_id) = cloned_nodes.get(&id) {
            return cloned_id;
        }

        let new_id = match &self.nodes[id] {
            Node::Con { tag, .. } => {
                let new_node = Node::Con {
                    tag: *tag,
                    ports: [Port::null(), Port::null()],
                };
                self.create_node(new_node)
            }
            Node::Dup { .. } => {
                let new_node = Node::Dup {
                    ports: [Port::null(), Port::null()],
                };
                self.create_node(new_node)
            }
            Node::Ref { name, .. } => {
                let new_node = Node::Ref {
                    name: name.clone(),
                    ports: [Port::null(), Port::null()],
                };
                self.create_node(new_node)
            }
            Node::Era { .. } => {
                let new_node = Node::Era {
                    ports: Port::null(),
                };
                self.create_node(new_node)
            }
        };
        cloned_nodes.insert(id, new_id);

        match &self.nodes[id] {
            Node::Con { ports, .. } => {
                for (i, port) in ports.iter().enumerate() {
                    if !port.is_null() {
                        let connected_id = port.node;
                        let connected_slot = port.slot;

                        let cloned_connected = self.clone_node(connected_id, cloned_nodes);

                        self.connect_nodes(new_id, i, cloned_connected, connected_slot);
                    }
                }
            }
            Node::Dup { ports, .. } => {
                for (i, port) in ports.iter().enumerate() {
                    if !port.is_null() {
                        let connected_id = port.node;
                        let connected_slot = port.slot;

                        // Clone the connected node
                        let cloned_connected = self.clone_node(connected_id, cloned_nodes);

                        // Connect the clones together
                        self.connect(new_id, i, cloned_connected, connected_slot);
                    }
                }
            }
            Node::Ref { port, .. } => {
                if !port.is_null() {
                    let connected_id = port.node;
                    let connected_slot = port.slot;

                    // Clone the connected node
                    let cloned_connected = self.clone_node(connected_id, cloned_nodes);

                    // Connect the clones together
                    self.connect(new_id, 0, cloned_connected, connected_slot);
                }
            }
            Node::Era { port, .. } => {
                if !port.is_null() {
                    let connected_id = port.node;
                    let connected_slot = port.slot;

                    // Clone the connected node
                    let cloned_connected = self.clone_node(connected_id, cloned_nodes);

                    // Connect the clones together
                    self.connect(new_id, 0, cloned_connected, connected_slot);
                }
            }
        }

        new_id
    }

    fn port_str(&self, port: &Port) -> String {
        if port.is_null() {
            "NULL".to_string()
        } else {
            format!("{}:{}", port.node, port.slot)
        }
    }

    pub fn node_str(&self, id: NodeId) -> String {
        match &self.nodes[id] {
            Node::Con { tag, ports } => {
                format!(
                    "CON({})[{}, {}]",
                    tag,
                    self.port_str(&ports[0]),
                    self.port_str(&ports[1])
                )
            }
            Node::Dup { ports } => {
                format!(
                    "DUP[{}, {}]",
                    self.port_str(&ports[0]),
                    self.port_str(&ports[1])
                )
            }
            Node::Ref { name, port } => {
                format!("REF({}):{}", name, self.port_str(port))
            }
            Node::Era { port, .. } => {
                format!("ERA({}):{}", self.port_str(port))
            }
        }
    }
}

impl fmt::Debug for Net {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the header
        writeln!(f, "Net with {} nodes:", self.nodes.len())?;

        // Write each node
        for (i, node) in self.nodes.iter().enumerate() {
            writeln!(f, "  {}: {:?}", i, node)?;
        }

        // Write function definitions
        writeln!(f, "Definitions:")?;
        for (name, &root) in &self.definitions {
            writeln!(f, "  {} -> {}", name, root)?;
        }

        // Write active redexes
        writeln!(f, "Active pairs: {:?}", self.active_pairs)?;

        // All done
        Ok(())
    }
}
