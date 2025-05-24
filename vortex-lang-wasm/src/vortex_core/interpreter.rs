// || shree ganesh ||
// Final Vortex Interpreter with return support and GPU placeholder

use super::ast::{Expr, Stmt};
use super::gpu_runtime::GPURuntime;
use super::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RuntimeResult {
    Value(Value),
    Return(Value), // Represents a return statement's value
    None,          // Represents no value, e.g., after a statement that doesn't produce one
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Floating(f64),
    String(String),
    Boolean(bool),
    Nil,
    Function(String, Vec<String>, Box<Stmt>, bool), // name, params, body, is_gpu
    Range { start: Box<Value>, end: Box<Value> }, 
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Floating(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Function(name, params, ..) => write!(f, "<fn {}({})>", name, params.join(", ")),
            Value::Range { start, end } => write!(f, "{}..{}", start, end),
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut env = Environment {
            values: HashMap::new(),
            parent: None,
        };
        env.define(
            "print".to_string(),
            Value::Function(
                "print".to_string(),
                vec!["value".to_string()], 
                Box::new(Stmt::Block(vec![])), 
                false,                         
            ),
        );
        Rc::new(RefCell::new(env))
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.borrow().get(name))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'. Cannot assign.", name))
        }
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    pub gpu_runtime: GPURuntime,
}

impl Interpreter {
    pub fn new() -> Self {
        let gpu_runtime = GPURuntime::new();
        
        Self {
            environment: Environment::new(),
            gpu_runtime,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<Option<Value>, String> {
        let mut last_value: Option<Value> = None;
        for stmt in statements {
            match self.execute(&stmt)? {
                RuntimeResult::Value(v) => {
                    if !matches!(v, Value::Nil) { 
                        last_value = Some(v);
                    }
                }
                RuntimeResult::Return(v) => return Ok(Some(v)), 
                RuntimeResult::None => {
                    last_value = None; 
                }
            }
        }
        Ok(last_value) 
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<RuntimeResult, String> {
        match stmt {
            Stmt::ExprStmt(expr) => self.evaluate(expr), 
            Stmt::Let { name, value, mutable: _, type_name: _ } => {
                match self.evaluate(value)? {
                    RuntimeResult::Value(v) => {
                        self.environment.borrow_mut().define(name.clone(), v);
                        Ok(RuntimeResult::None) 
                    }
                    RuntimeResult::Return(_) => Err("Invalid return in variable declaration".to_string()),
                    RuntimeResult::None => { 
                        self.environment.borrow_mut().define(name.clone(), Value::Nil);
                         Ok(RuntimeResult::None)
                    }
                }
            }
            Stmt::Block(statements) => {
                let previous_env = Rc::clone(&self.environment);
                self.environment = Environment::with_parent(Rc::clone(&previous_env));
                let mut block_result = Ok(RuntimeResult::None); 

                for s in statements {
                    match self.execute(s)? {
                        RuntimeResult::Return(val) => { 
                            block_result = Ok(RuntimeResult::Return(val));
                            break;
                        }
                        RuntimeResult::Value(val) => { 
                            block_result = Ok(RuntimeResult::Value(val));
                        }
                        RuntimeResult::None => {}
                    }
                }
                self.environment = previous_env; 
                block_result
            }
            Stmt::IfStmt { condition, then_branch, else_branch } => {
                match self.evaluate(condition)? {
                    RuntimeResult::Value(cond_val) => {
                        if self.is_truthy(&cond_val) {
                            self.execute(then_branch)
                        } else if let Some(else_b) = else_branch {
                            self.execute(else_b)
                        } else {
                            Ok(RuntimeResult::None) 
                        }
                    }
                    _ => Err("If condition must evaluate to a value".to_string()),
                }
            }
            Stmt::Branch { condition, body } => { 
                match self.evaluate(condition)? {
                    RuntimeResult::Value(cond_val) => {
                        if self.is_truthy(&cond_val) {
                            self.execute(body)
                        } else {
                            Ok(RuntimeResult::None)
                        }
                    }
                     _ => Err("Branch condition must evaluate to a value".to_string()),
                }
            }
            Stmt::Fallback(body) => { 
                self.execute(body)
            }
            Stmt::For { var, range, body } => {
                let range_val = match self.evaluate(range)? {
                    RuntimeResult::Value(Value::Range { start, end }) => (*start, *end),
                    RuntimeResult::Value(Value::Number(n)) if n >= 0 => (Value::Number(0), Value::Number(n)),
                    _ => return Err("For loop range must be a valid range (e.g., 0..10) or a positive number.".to_string()),
                };

                let (start_num, end_num) = match (range_val.0, range_val.1) {
                    (Value::Number(s), Value::Number(e)) => (s, e),
                    _ => return Err("Range bounds must be numbers.".to_string()),
                };

                if start_num >= end_num { 
                     return Ok(RuntimeResult::None);
                }

                for i in start_num..end_num { 
                    let loop_env = Environment::with_parent(Rc::clone(&self.environment));
                    let prev_env = std::mem::replace(&mut self.environment, loop_env);
                    self.environment.borrow_mut().define(var.clone(), Value::Number(i));

                    match self.execute(body)? {
                        RuntimeResult::Return(v) => { 
                            self.environment = prev_env;
                            return Ok(RuntimeResult::Return(v));
                        }
                        _ => {} 
                    }
                    self.environment = prev_env; 
                }
                Ok(RuntimeResult::None) 
            }
            Stmt::Parallel { var, range, body } => {
                 let (start_val, end_val) = match self.evaluate(range)? {
                    RuntimeResult::Value(Value::Range { start, end }) => {
                        let s = if let Value::Number(n) = *start { n } else { return Err("Parallel range start must be a number".to_string()); };
                        let e = if let Value::Number(n) = *end { n } else { return Err("Parallel range end must be a number".to_string()); };
                        (s, e)
                    },
                    RuntimeResult::Value(Value::Number(n)) if n >= 0 => (0, n),
                    _ => return Err("Invalid range expression for 'parallel' loop".to_string()),
                };

                let range_expr_for_gpu = Expr::Range { 
                    start: Box::new(Expr::Number(start_val)),
                    end: Box::new(Expr::Number(end_val)),
                };

                match self.gpu_runtime.execute_parallel(var, &range_expr_for_gpu, body) {
                    Ok(_) => Ok(RuntimeResult::None),
                    Err(e) => {
                        // Fallback to sequential execution
                        println!("[Warning] GPU parallel execution failed: {}. Falling back to sequential.", e);
                        for i in start_val..end_val {
                            let loop_env = Environment::with_parent(Rc::clone(&self.environment));
                            let prev_env = std::mem::replace(&mut self.environment, loop_env);
                            self.environment.borrow_mut().define(var.clone(), Value::Number(i));
                            if let RuntimeResult::Return(v) = self.execute(body)? {
                                 self.environment = prev_env;
                                 return Ok(RuntimeResult::Return(v)); 
                            }
                            self.environment = prev_env;
                        }
                        Ok(RuntimeResult::None)
                    }
                }
            }
            Stmt::Return(expr) => {
                match self.evaluate(expr)? {
                    RuntimeResult::Value(v) => Ok(RuntimeResult::Return(v)),
                    RuntimeResult::Return(_) => Err("Nested return statement not allowed".to_string()),
                    RuntimeResult::None => Ok(RuntimeResult::Return(Value::Nil)), 
                }
            }
            Stmt::FunctionDef { name, params, body, gpu, .. } => {
                let param_names: Vec<String> = params.iter().map(|(p_name, _)| p_name.clone()).collect();
                self.environment.borrow_mut().define(
                    name.clone(),
                    Value::Function(name.clone(), param_names.clone(), body.clone(), *gpu)
                );
                if *gpu {
                    let param_types_for_gpu: Vec<(String, String)> = params.iter()
                        .map(|(p_name, type_opt)| (p_name.clone(), type_opt.clone().unwrap_or_else(|| "Any".to_string())))
                        .collect();
                    if let Err(e) = self.gpu_runtime.register_function(name.clone(), param_types_for_gpu, body) {
                         println!("[Warning] Failed to register GPU function '{}': {}", name, e);
                    }
                }
                Ok(RuntimeResult::None) 
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<RuntimeResult, String> {
        match expr {
            Expr::Number(n) => Ok(RuntimeResult::Value(Value::Number(*n))),
            Expr::Floating(f) => Ok(RuntimeResult::Value(Value::Floating(*f))),
            Expr::Boolean(b) => Ok(RuntimeResult::Value(Value::Boolean(*b))),
            Expr::String(s) => Ok(RuntimeResult::Value(Value::String(s.clone()))),
            Expr::Ident(name) => {
                if name == "error" { 
                    return Err("Encountered parser error placeholder in expression".to_string());
                }
                if name == "print" {
                    return Ok(RuntimeResult::Value(Value::Function(
                        "print".to_string(),
                        vec!["value".to_string()],
                        Box::new(Stmt::Block(vec![])), 
                        false,
                    )));
                }
                self.environment.borrow().get(name)
                    .map(RuntimeResult::Value)
                    .ok_or_else(|| format!("Undefined identifier '{}'", name))
            }
            Expr::Assignment { name, value } => {
                match self.evaluate(value)? {
                    RuntimeResult::Value(v) => {
                        self.environment.borrow_mut().assign(name, v.clone())?;
                        Ok(RuntimeResult::Value(v)) 
                    }
                    _ => Err("Cannot assign a non-value (e.g. result of a return) to a variable".to_string()),
                }
            }
            Expr::Binary { left, op, right } => {
                let l_eval = self.evaluate(left)?;
                let r_eval = self.evaluate(right)?;
                match (l_eval, r_eval) {
                    (RuntimeResult::Value(l_val), RuntimeResult::Value(r_val)) => {
                        self.binary_op(op, l_val, r_val).map(RuntimeResult::Value)
                    }
                    _ => Err("Operands of binary expression must be values".to_string()),
                }
            }
            Expr::Unary { op, expr } => {
                 match self.evaluate(expr)? {
                    RuntimeResult::Value(v) => self.unary_op(op, v).map(RuntimeResult::Value),
                    _ => Err("Operand of unary expression must be a value".to_string()),
                }
            }
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Return(ret_expr) => { 
                match self.evaluate(ret_expr)? {
                    RuntimeResult::Value(v) => Ok(RuntimeResult::Return(v)),
                    RuntimeResult::Return(_) => Err("Nested Expr::Return not supported".to_string()),
                    RuntimeResult::None => Ok(RuntimeResult::Return(Value::Nil)),
                }
            }
            Expr::Range { start, end } => {
                let s_eval = self.evaluate(start)?;
                let e_eval = self.evaluate(end)?;
                match (s_eval, e_eval) {
                    (RuntimeResult::Value(s_val), RuntimeResult::Value(e_val)) => {
                        Ok(RuntimeResult::Value(Value::Range { start: Box::new(s_val), end: Box::new(e_val) }))
                    }
                     _ => Err("Range start and end must evaluate to values.".to_string()),
                }
            }
            Expr::FunctionCall { callee, arguments } => {
                let callee_eval = self.evaluate(callee)?;
                let callee_val = match callee_eval {
                    RuntimeResult::Value(v) => v,
                    _ => return Err(format!("Callee must be a function, got {:?}", callee_eval)),
                };

                let mut arg_values = Vec::new();
                for arg_expr in arguments {
                    match self.evaluate(arg_expr)? {
                        RuntimeResult::Value(v) => arg_values.push(v),
                        _ => return Err("Function arguments must evaluate to values".to_string()),
                    }
                }

                match callee_val {
                    Value::Function(fn_name, param_names, body, is_gpu) => {
                        if fn_name == "print" { 
                                                    let output = arg_values.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join(" "); 
                                                    return Ok(RuntimeResult::Value(Value::String(output))); 
                                                }

                        if arg_values.len() != param_names.len() {
                            return Err(format!(
                                "Function '{}' expects {} arguments, but got {}",
                                fn_name, param_names.len(), arg_values.len()
                            ));
                        }
                        
                        if is_gpu {
                            match self.gpu_runtime.execute_function(&fn_name, arg_values) {
                                Ok(result_val) => Ok(RuntimeResult::Value(result_val)),
                                Err(e) => Err(format!("GPU function '{}' execution error: {}", fn_name, e)),
                            }
                        } else {
                            let prev_env = Rc::clone(&self.environment);
                            let func_env = Environment::with_parent(Rc::clone(&prev_env));
                            self.environment = func_env;

                            for (param, value) in param_names.iter().zip(arg_values.into_iter()) {
                                self.environment.borrow_mut().define(param.clone(), value);
                            }

                            let result = match self.execute(&body)? {
                                RuntimeResult::Return(val) => Ok(RuntimeResult::Value(val)), 
                                RuntimeResult::Value(val) => Ok(RuntimeResult::Value(val)),
                                RuntimeResult::None => Ok(RuntimeResult::Value(Value::Nil)), 
                            };
                            self.environment = prev_env; 
                            result
                        }
                    }
                    _ => Err(format!("'{}' is not callable.", callee_val)),
                }
            }
        }
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            Value::Number(n) => *n != 0,
            Value::Floating(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true, 
        }
    }
    
    fn binary_op(&self, op: &Token, left: Value, right: Value) -> Result<Value, String> {
        use Value::*;
        match op {
            Token::Plus => match (left.clone(), right.clone()) { 
                (Number(a), Number(b)) => Ok(Number(a + b)),
                (Floating(a), Floating(b)) => Ok(Floating(a + b)),
                (Number(a), Floating(b)) => Ok(Floating(a as f64 + b)),
                (Floating(a), Number(b)) => Ok(Floating(a + b as f64)),
                (String(a), String(b)) => Ok(String(a + &b)),
                (String(a), Number(b)) => Ok(String(format!("{}{}", a, b))),
                (String(a), Floating(b)) => Ok(String(format!("{}{}", a, b))),
                (String(a), Boolean(b)) => Ok(String(format!("{}{}", a, b))),
                (Number(a), String(b)) => Ok(String(format!("{}{}", a, b))),
                (Floating(a), String(b)) => Ok(String(format!("{}{}", a, b))),
                (Boolean(a), String(b)) => Ok(String(format!("{}{}", a, b))),
                _ => Err(format!("Cannot apply '+' to {} and {}", left, right)),
            },
            Token::Minus => self.numeric_op(left, right, |a, b| a - b, |a, b| a - b),
            Token::Star => self.numeric_op(left, right, |a, b| a * b, |a, b| a * b),
            Token::Slash => {
                match (&left, &right) {
                    (_, Number(0)) => Err("Division by zero (integer)".to_string()),
                    (_, Floating(f)) if *f == 0.0 => Err("Division by zero (float)".to_string()),
                    _ => self.numeric_op(left, right, |a, b| a / b, |a, b| a / b),
                }
            },
            Token::EQ => Ok(Boolean(left == right)),
            Token::NE => Ok(Boolean(left != right)),
            Token::GT => self.compare_op(left, right, |a, b| a > b, |a,b| a > b),
            Token::LT => self.compare_op(left, right, |a, b| a < b, |a,b| a < b),
            Token::GE => self.compare_op(left, right, |a, b| a >= b, |a,b| a >= b),
            Token::LE => self.compare_op(left, right, |a, b| a <= b, |a,b| a <= b),
            _ => Err(format!("Unsupported binary operator: {:?}", op)),
        }
    }

    fn unary_op(&self, op: &Token, val: Value) -> Result<Value, String> {
        match op {
            Token::Minus => match val {
                Value::Number(n) => Ok(Value::Number(-n)),
                Value::Floating(f) => Ok(Value::Floating(-f)),
                _ => Err(format!("Cannot apply unary '-' to {}", val)),
            },
            _ => Err(format!("Unsupported unary operator: {:?}", op)),
        }
    }

    fn numeric_op<FInt, FFloat>(&self, left: Value, right: Value, f_int: FInt, f_float: FFloat) -> Result<Value, String>
    where
        FInt: Fn(i64, i64) -> i64,
        FFloat: Fn(f64, f64) -> f64,
    {
        use Value::*;
        match (left.clone(), right.clone()) {
            (Number(a), Number(b)) => Ok(Number(f_int(a, b))),
            (Floating(a), Floating(b)) => Ok(Floating(f_float(a, b))),
            (Number(a), Floating(b)) => Ok(Floating(f_float(a as f64, b))),
            (Floating(a), Number(b)) => Ok(Floating(f_float(a, b as f64))),
            _ => Err(format!("Numeric operation requires numbers, got {} and {}", left, right)),
        }
    }

    fn compare_op<FInt, FFloat>(&self, left: Value, right: Value, f_int: FInt, f_float: FFloat) -> Result<Value, String>
    where
        FInt: Fn(i64, i64) -> bool,
        FFloat: Fn(f64, f64) -> bool,
    {
        use Value::*;
        match (left.clone(), right.clone()) {
            (Number(a), Number(b)) => Ok(Boolean(f_int(a, b))),
            (Floating(a), Floating(b)) => Ok(Boolean(f_float(a, b))),
            (Number(a), Floating(b)) => Ok(Boolean(f_float(a as f64, b))),
            (Floating(a), Number(b)) => Ok(Boolean(f_float(a, b as f64))),
            _ => Err(format!("Comparison requires numbers or compatible types, got {} and {}", left, right)),
        }
    }
}
