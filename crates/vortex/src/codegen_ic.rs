use std::collections::HashMap;

use nexus_ir::{CellKind, Net, OpKind, Port};

use crate::ast::{Expr, Stmt};
use crate::token::Token;

type Env = HashMap<String, Port>;

#[derive(Clone)]
struct FunctionDef {
    params: Vec<String>,
    body: Stmt,
}

pub struct Codegen {
    pub net: Net,
    functions: HashMap<String, FunctionDef>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            net: Net::new(),
            functions: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, program: &[Stmt]) -> Port {
        let mut env = HashMap::new();
        let result = self.compile_statements(program, &mut env);

        if self.net.iface.is_empty() {
            self.net.iface.push(result.clone());
        } else {
            self.net.iface[0] = result.clone();
        }

        result
    }

    fn compile_statements(&mut self, statements: &[Stmt], env: &mut Env) -> Port {
        let mut last: Option<Port> = None;

        for stmt in statements {
            if let Some(value) = self.compile_stmt(stmt, env) {
                last = Some(value);
            }
        }

        last.unwrap_or_else(|| self.compile_int(0))
    }

    fn compile_stmt(&mut self, stmt: &Stmt, env: &mut Env) -> Option<Port> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let value_port = self.compile(value, env);
                env.insert(name.clone(), value_port.clone());
                Some(value_port)
            }
            Stmt::ExprStmt(expr) => Some(self.compile(expr, env)),
            Stmt::Block(statements) => {
                let mut local_env = env.clone();
                Some(self.compile_statements(statements, &mut local_env))
            }
            Stmt::FunctionDef {
                name, params, body, ..
            } => {
                let params = params.iter().map(|(name, _)| name.clone()).collect();
                self.functions.insert(
                    name.clone(),
                    FunctionDef {
                        params,
                        body: body.as_ref().clone(),
                    },
                );

                let fn_cell = self.net.alloc(CellKind::Con);
                self.net
                    .debug
                    .insert(fn_cell, format!("fn-definition:{name}"));
                None
            }
            Stmt::Return(expr) => Some(self.compile(expr, env)),
            Stmt::IfStmt { then_branch, .. } => self.compile_stmt(then_branch, env),
            Stmt::For { body, .. } | Stmt::Parallel { body, .. } | Stmt::Branch { body, .. } => {
                self.compile_stmt(body, env)
            }
            Stmt::Fallback(body) => self.compile_stmt(body, env),
        }
    }

    pub fn compile(&mut self, expr: &Expr, env: &mut Env) -> Port {
        match expr {
            Expr::Number(n) => self.compile_int(*n),
            Expr::Floating(f) => self.compile_int(*f as i64),
            Expr::Boolean(b) => self.compile_int(if *b { 1 } else { 0 }),
            Expr::String(_) => self.compile_int(0),
            Expr::Ident(name) => self.compile_var(name, env),
            Expr::Unary { op, expr } => self.compile_unary(op, expr, env),
            Expr::Binary { left, op, right } => self.compile_binop(op, left, right, env),
            Expr::Assignment { name, value } => self.compile_assignment(name, value, env),
            Expr::Grouping(inner) => self.compile(inner, env),
            Expr::FunctionCall { callee, arguments } => self.compile_fncall(callee, arguments, env),
            Expr::Return(expr) => self.compile(expr, env),
            Expr::Range { .. } => self.compile_int(0),
        }
    }

    fn compile_int(&mut self, n: i64) -> Port {
        let id = self.net.alloc(CellKind::Num(n));
        Port::Principal(id)
    }

    fn compile_unary(&mut self, op: &Token, expr: &Expr, env: &mut Env) -> Port {
        match op {
            Token::Minus => {
                let zero = self.compile_int(0);
                let rhs = self.compile(expr, env);
                self.compile_binop_direct(OpKind::Sub, zero, rhs)
            }
            _ => self.compile(expr, env),
        }
    }

    fn compile_assignment(&mut self, name: &str, value: &Expr, env: &mut Env) -> Port {
        let value_port = self.compile(value, env);
        env.insert(name.to_string(), value_port.clone());
        value_port
    }

    fn compile_binop(&mut self, op: &Token, a: &Expr, b: &Expr, env: &mut Env) -> Port {
        let op_kind = map_op(op);
        let a_port = self.compile(a, env);
        let b_port = self.compile(b, env);
        self.compile_binop_direct(op_kind, a_port, b_port)
    }

    fn compile_binop_direct(&mut self, op_kind: OpKind, a_port: Port, b_port: Port) -> Port {
        let op_cell = self.net.alloc(CellKind::Op(op_kind));
        self.net.connect(Port::Aux(op_cell, 0), a_port.clone());
        self.net.connect(Port::Aux(op_cell, 1), b_port);

        let left = match a_port {
            Port::Principal(id) => id,
            Port::Aux(id, _) => id,
            Port::Free(_) => op_cell,
        };
        self.net.add_redex(op_cell, left);

        Port::Principal(op_cell)
    }

    fn compile_var(&self, name: &str, env: &Env) -> Port {
        env.get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Unbound variable: {name}"))
    }

    fn compile_fncall(&mut self, callee: &Expr, args: &[Expr], env: &mut Env) -> Port {
        if let Expr::Ident(name) = callee {
            if let Some(function) = self.functions.get(name).cloned() {
                return self.compile_named_fncall(name, function, args, env);
            }
        }

        // Fallback for non-named calls: compile first argument to keep behavior predictable.
        let _ = self.compile(callee, env);
        if let Some(arg) = args.first() {
            self.compile(arg, env)
        } else {
            self.compile_int(0)
        }
    }

    fn compile_named_fncall(
        &mut self,
        function_name: &str,
        function: FunctionDef,
        args: &[Expr],
        env: &mut Env,
    ) -> Port {
        let app = self.net.alloc(CellKind::Con);
        let function_cell = self.net.alloc(CellKind::Con);
        self.net
            .debug
            .insert(app, format!("fn-call:{function_name}"));
        self.net
            .debug
            .insert(function_cell, format!("fn:{function_name}"));

        self.net
            .connect(Port::Principal(app), Port::Principal(function_cell));
        self.net.add_redex(app, function_cell);

        let mut fn_env = env.clone();
        if let Some(first_arg) = args.first() {
            let arg_port = self.compile(first_arg, env);
            self.net.connect(Port::Aux(app, 0), arg_port.clone());
            if let Some(param) = function.params.first() {
                fn_env.insert(param.clone(), arg_port);
            }
        }

        for (param, arg) in function.params.iter().zip(args.iter()).skip(1) {
            let arg_port = self.compile(arg, env);
            fn_env.insert(param.clone(), arg_port);
        }

        let body_result = self
            .compile_stmt(&function.body, &mut fn_env)
            .unwrap_or_else(|| self.compile_int(0));

        self.net.connect(Port::Aux(app, 1), body_result);
        Port::Aux(app, 1)
    }
}

fn map_op(token: &Token) -> OpKind {
    match token {
        Token::Plus => OpKind::Add,
        Token::Minus => OpKind::Sub,
        Token::Star => OpKind::Mul,
        Token::Slash => OpKind::Div,
        Token::EQ => OpKind::Eq,
        Token::NE => OpKind::Ne,
        Token::LT => OpKind::Lt,
        Token::LE => OpKind::Le,
        Token::GT => OpKind::Gt,
        Token::GE => OpKind::Ge,
        _ => OpKind::Add,
    }
}

#[cfg(test)]
mod tests {
    use super::Codegen;
    use crate::ast::{Expr, Stmt};
    use crate::token::Token;
    use nexus_ir::CellKind;

    fn compile(statements: Vec<Stmt>) -> Codegen {
        let mut cg = Codegen::new();
        cg.compile_program(&statements);
        cg
    }

    #[test]
    fn compiles_simple_addition_net_shape() {
        let program = vec![Stmt::ExprStmt(Expr::Binary {
            left: Box::new(Expr::Number(2)),
            op: Token::Plus,
            right: Box::new(Expr::Number(3)),
        })];

        let cg = compile(program);
        assert_eq!(cg.net.cells.len(), 3);
        assert_eq!(cg.net.redexes.len(), 1);

        let op_count = cg
            .net
            .cells
            .values()
            .filter(|cell| matches!(cell.kind, CellKind::Op(_)))
            .count();
        assert_eq!(op_count, 1);
    }

    #[test]
    fn compiles_let_binding_and_reduces() {
        let program = vec![
            Stmt::Let {
                name: "x".to_string(),
                type_name: None,
                value: Expr::Number(5),
                mutable: false,
            },
            Stmt::ExprStmt(Expr::Binary {
                left: Box::new(Expr::Ident("x".to_string())),
                op: Token::Plus,
                right: Box::new(Expr::Number(1)),
            }),
        ];

        let cg = compile(program);
        let value = vice::reduce_ir(cg.net);
        assert_eq!(value, 6);
    }

    #[test]
    fn compiles_function_call_and_reduces() {
        let square_body = Stmt::Block(vec![Stmt::Return(Expr::Binary {
            left: Box::new(Expr::Ident("n".to_string())),
            op: Token::Star,
            right: Box::new(Expr::Ident("n".to_string())),
        })]);

        let program = vec![
            Stmt::FunctionDef {
                name: "square".to_string(),
                params: vec![("n".to_string(), None)],
                return_type: None,
                body: Box::new(square_body),
                gpu: false,
            },
            Stmt::ExprStmt(Expr::FunctionCall {
                callee: Box::new(Expr::Ident("square".to_string())),
                arguments: vec![Expr::Number(9)],
            }),
        ];

        let cg = compile(program);
        let value = vice::reduce_ir(cg.net);
        assert_eq!(value, 81);
    }
}
