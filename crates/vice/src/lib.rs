use std::collections::HashMap;
use std::time::Instant;

use nexus_ir::{CellId, CellKind, Net as IrNet, OpKind, Port};

#[derive(Debug, Clone, Default)]
pub struct ReduceStats {
    pub reductions: usize,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Default)]
pub struct ReduceOutput {
    pub value: i64,
    pub stats: ReduceStats,
}

pub fn reduce_ir(ir: IrNet) -> i64 {
    reduce_ir_with_stats(ir).value
}

pub fn reduce_ir_with_stats(ir: IrNet) -> ReduceOutput {
    let started = Instant::now();
    let mut evaluator = Evaluator::new(&ir);

    // TODO v0.2: replace this sequential loop with rayon::par_iter()
    let value = evaluator.eval_root();

    let elapsed_ms = started.elapsed().as_millis();
    let reductions = ir.redexes.len().max(evaluator.op_steps);

    ReduceOutput {
        value,
        stats: ReduceStats {
            reductions,
            elapsed_ms,
        },
    }
}

struct Evaluator<'a> {
    net: &'a IrNet,
    memo: HashMap<CellId, i64>,
    op_steps: usize,
}

impl<'a> Evaluator<'a> {
    fn new(net: &'a IrNet) -> Self {
        Self {
            net,
            memo: HashMap::new(),
            op_steps: 0,
        }
    }

    fn eval_root(&mut self) -> i64 {
        if let Some(root) = self.net.iface.first() {
            self.eval_port(root)
        } else {
            0
        }
    }

    fn eval_port(&mut self, port: &Port) -> i64 {
        match port {
            Port::Principal(id) => self.eval_cell(*id),
            Port::Aux(id, idx) => self
                .net
                .cells
                .get(id)
                .and_then(|cell| cell.ports.get(1 + *idx as usize))
                .and_then(|port| port.as_ref())
                .map(|target| self.eval_port(target))
                .unwrap_or(0),
            Port::Free(id) => self
                .net
                .iface
                .get(*id as usize)
                .map(|target| self.eval_port(target))
                .unwrap_or(0),
        }
    }

    fn eval_cell(&mut self, id: CellId) -> i64 {
        if let Some(cached) = self.memo.get(&id) {
            return *cached;
        }

        let value = match self.net.cells.get(&id) {
            Some(cell) => match &cell.kind {
                CellKind::Num(value) => *value,
                CellKind::Op(kind) => {
                    self.op_steps += 1;
                    let lhs = self.read_aux(cell.ports[1].as_ref());
                    let rhs = self.read_aux(cell.ports[2].as_ref());
                    apply_op(kind, lhs, rhs)
                }
                CellKind::Con | CellKind::Dup | CellKind::Switch | CellKind::Era => cell
                    .ports
                    .get(2)
                    .and_then(|p| p.as_ref())
                    .or_else(|| cell.ports.get(1).and_then(|p| p.as_ref()))
                    .or_else(|| cell.ports.first().and_then(|p| p.as_ref()))
                    .map(|target| self.eval_port(target))
                    .unwrap_or(0),
            },
            None => 0,
        };

        self.memo.insert(id, value);
        value
    }

    fn read_aux(&mut self, port: Option<&Port>) -> i64 {
        port.map(|port| self.eval_port(port)).unwrap_or(0)
    }
}

fn apply_op(kind: &OpKind, lhs: i64, rhs: i64) -> i64 {
    match kind {
        OpKind::Add => lhs + rhs,
        OpKind::Sub => lhs - rhs,
        OpKind::Mul => lhs * rhs,
        OpKind::Div => {
            if rhs == 0 {
                0
            } else {
                lhs / rhs
            }
        }
        OpKind::Mod => {
            if rhs == 0 {
                0
            } else {
                lhs % rhs
            }
        }
        OpKind::Eq => (lhs == rhs) as i64,
        OpKind::Ne => (lhs != rhs) as i64,
        OpKind::Lt => (lhs < rhs) as i64,
        OpKind::Le => (lhs <= rhs) as i64,
        OpKind::Gt => (lhs > rhs) as i64,
        OpKind::Ge => (lhs >= rhs) as i64,
        OpKind::And => ((lhs != 0) && (rhs != 0)) as i64,
        OpKind::Or => ((lhs != 0) || (rhs != 0)) as i64,
        OpKind::Not => (lhs == 0) as i64,
    }
}

#[cfg(test)]
mod tests {
    use nexus_ir::{CellKind, Net, OpKind, Port};

    use crate::reduce_ir;

    #[test]
    fn reduces_basic_arithmetic_net() {
        let mut net = Net::new();
        let n2 = net.alloc(CellKind::Num(2));
        let n3 = net.alloc(CellKind::Num(3));
        let add = net.alloc(CellKind::Op(OpKind::Add));

        net.connect(Port::Aux(add, 0), Port::Principal(n2));
        net.connect(Port::Aux(add, 1), Port::Principal(n3));
        net.add_redex(add, n2);
        net.iface.push(Port::Principal(add));

        assert_eq!(reduce_ir(net), 5);
    }
}
