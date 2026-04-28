use std::collections::HashMap;

pub type CellId = u32;
pub type IfaceId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Port {
    Principal(CellId),
    Aux(CellId, u8),
    Free(IfaceId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellKind {
    Con,
    Dup,
    Era,
    Num(i64),
    Op(OpKind),
    Switch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub id: CellId,
    pub kind: CellKind,
    pub ports: [Option<Port>; 3],
}

impl Cell {
    pub fn new(id: CellId, kind: CellKind) -> Self {
        Self {
            id,
            kind,
            ports: [None, None, None],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Redex {
    pub left: CellId,
    pub right: CellId,
}

#[derive(Debug, Default, Clone)]
pub struct Net {
    pub cells: HashMap<CellId, Cell>,
    pub redexes: Vec<Redex>,
    pub iface: Vec<Port>,
    pub debug: HashMap<CellId, String>,
    next_id: CellId,
}

impl Net {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc(&mut self, kind: CellKind) -> CellId {
        let id = self.next_id;
        self.next_id += 1;
        self.cells.insert(id, Cell::new(id, kind));
        id
    }

    pub fn connect(&mut self, a: Port, b: Port) {
        self.set_port(a.clone(), b.clone());
        self.set_port(b, a);
    }

    fn set_port(&mut self, target: Port, value: Port) {
        match target {
            Port::Principal(id) => {
                if let Some(cell) = self.cells.get_mut(&id) {
                    cell.ports[0] = Some(value);
                }
            }
            Port::Aux(id, idx) => {
                if let Some(cell) = self.cells.get_mut(&id) {
                    let slot = 1 + idx as usize;
                    if slot < 3 {
                        cell.ports[slot] = Some(value);
                    }
                }
            }
            Port::Free(_) => {}
        }
    }

    pub fn add_redex(&mut self, left: CellId, right: CellId) {
        self.redexes.push(Redex { left, right });
    }
}

#[cfg(test)]
mod tests {
    use super::{CellKind, Net, Port};

    #[test]
    fn alloc_connect_and_redex_smoke_test() {
        let mut net = Net::new();

        let left = net.alloc(CellKind::Con);
        let right = net.alloc(CellKind::Con);

        net.connect(Port::Principal(left), Port::Principal(right));
        net.add_redex(left, right);

        assert_eq!(net.cells.len(), 2);
        assert_eq!(net.redexes.len(), 1);
    }
}
