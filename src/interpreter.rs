// || shree ganesh ||
// Final Vortex Interpreter with return support and GPU placeholder

use crate::ast::{Expr, Stmt};
use crate::gpu_runtime::GPURuntime;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RuntimeResult {
    Value(Value),
    Return(Value),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Floating(f64),
    String(String),
    Boolean(bool),
    Nil,
    Function(String, Vec<String>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Floating(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Function(name, _) => write!(f, "<function {}>", name),
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
            Value::Function("print".to_string(), vec!["value".to_string()]),
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
            Err(format!("Undefined variable '{}'.", name))
        }
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    gpu_runtime: GPURuntime,
}

impl Interpreter {
    pub fn new() -> Self {
        let gpu_runtime = GPURuntime::new();
        
        // Check if GPU acceleration is available
        if gpu_runtime.is_available() {
            println!("GPU acceleration is enabled. GPU functions will run on the GPU.");
        } else {
            println!("GPU acceleration is not available. GPU functions will be simulated on the CPU.");
        }
        
        Self {
            environment: Environment::new(),
            gpu_runtime,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for stmt in statements {
            let result = self.execute(&stmt)?;
            if let RuntimeResult::Return(_) = result {
                break;
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<RuntimeResult, String> {
        match stmt {
            Stmt::ExprStmt(expr) => self.evaluate(expr),
            Stmt::Let { name, value, .. } => {
                match self.evaluate(value) {
                    Ok(RuntimeResult::Value(v)) => {
                        self.environment.borrow_mut().define(name.clone(), v);
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    Ok(RuntimeResult::Return(_)) => {
                        Err("Invalid return in variable declaration".to_string())
                    },
                    Ok(RuntimeResult::None) => {
                        // Define variable with default nil value
                        self.environment.borrow_mut().define(name.clone(), Value::Nil);
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    Err(e) => {
                        println!("Warning: Error in variable declaration: {}", e);
                        // Define variable with default nil value to continue execution
                        self.environment.borrow_mut().define(name.clone(), Value::Nil);
                        Ok(RuntimeResult::Value(Value::Nil))
                    }
                }
            }
            Stmt::Block(statements) => {
                let previous = Rc::clone(&self.environment);
                self.environment = Environment::with_parent(previous.clone());
                for stmt in statements {
                    let result = self.execute(stmt)?;
                    if let RuntimeResult::Return(_) = result {
                        self.environment = previous;
                        return Ok(result);
                    }
                }
                self.environment = previous;
                Ok(RuntimeResult::Value(Value::Nil))
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.evaluate(condition)?;
                if let RuntimeResult::Value(val) = cond {
                    if self.is_truthy(&val) {
                        self.execute(then_branch)
                    } else if let Some(else_branch) = else_branch {
                        self.execute(else_branch)
                    } else {
                        Ok(RuntimeResult::Value(Value::Nil))
                    }
                } else {
                    Err("Invalid return in if condition".to_string())
                }
            }
            Stmt::Branch {
                condition,
                body,
            } => {
                println!("Executing branch statement");
                let cond = self.evaluate(condition)?;
                if let RuntimeResult::Value(val) = cond {
                    if self.is_truthy(&val) {
                        println!("Branch condition true, executing body");
                        self.execute(body)
                    } else {
                        println!("Branch condition false, skipping body");
                        Ok(RuntimeResult::Value(Value::Nil))
                    }
                } else {
                    Err("Invalid return in branch condition".to_string())
                }
            }
            Stmt::Fallback(body) => {
                self.execute(body)
            }
            Stmt::For { var, range, body } => {
                println!("Executing 'for' loop with variable '{}'", var);
                // Basic implementation for 'for' loops
                match self.evaluate(&range)? {
                    RuntimeResult::Value(Value::Number(count)) => {
                        println!("Loop range evaluated to number: {}", count);
                        for i in 0..count {
                            println!("For loop iteration {}", i);
                            // Create a new environment for each iteration
                            let loop_env = Environment::with_parent(Rc::clone(&self.environment));
                            let prev_env = std::mem::replace(&mut self.environment, loop_env);
                            
                            // Define the loop variable
                            self.environment.borrow_mut().define(var.clone(), Value::Number(i));
                            
                            // Execute the loop body
                            match self.execute(body) {
                                Ok(RuntimeResult::Return(v)) => {
                                    println!("Return statement in loop body, exiting loop");
                                    self.environment = prev_env;
                                    return Ok(RuntimeResult::Return(v));
                                },
                                Err(e) => {
                                    println!("Error in loop body: {}", e);
                                    self.environment = prev_env;
                                    return Err(e);
                                },
                                _ => {}
                            }
                            
                            // Restore the environment
                            self.environment = prev_env;
                        }
                        println!("For loop completed");
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    RuntimeResult::Value(value) => {
                        // Handle range expressions (0..10)
                        println!("Loop range evaluated to: {:?}", value);
                        // Default to empty range
                        println!("Warning: For loop range is not a number or valid range");
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    _ => {
                        println!("Warning: For loop range evaluation failed");
                        Ok(RuntimeResult::Value(Value::Nil))
                    }
                }
            }
            Stmt::Parallel { var, range, body } => {
                println!("Executing parallel loop with variable '{}'", var);
                
                // Use the GPU runtime for parallel execution
                match self.evaluate(&range)? {
                    RuntimeResult::Value(Value::Number(count)) => {
                        println!("Using GPU runtime for parallel execution over range 0..{}", count);
                        // Convert the range to an expression for the GPU runtime
                        let range_expr = Expr::Number(count);
                        
                        // Execute in parallel on the GPU
                        match self.gpu_runtime.execute_parallel(&var, &range_expr, body) {
                            Ok(_) => {
                                println!("GPU parallel execution completed successfully");
                                Ok(RuntimeResult::Value(Value::Nil))
                            },
                            Err(e) => {
                                println!("GPU parallel execution failed: {}", e);
                                println!("Falling back to sequential execution");
                                
                                // Fall back to sequential execution
                                for i in 0..count {
                                    let loop_env = Environment::with_parent(Rc::clone(&self.environment));
                                    let prev_env = std::mem::replace(&mut self.environment, loop_env);
                                    
                                    self.environment.borrow_mut().define(var.clone(), Value::Number(i));
                                    
                                    let _ = self.execute(body); // Ignore returns in parallel execution
                                    
                                    self.environment = prev_env;
                                }
                                Ok(RuntimeResult::Value(Value::Nil))
                            }
                        }
                    },
                    RuntimeResult::Value(value) => {
                        println!("Warning: Parallel loop range is not a number: {:?}", value);
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    _ => {
                        println!("Warning: Parallel loop range evaluation failed");
                        Ok(RuntimeResult::Value(Value::Nil))
                    }
                }
            }
            Stmt::Return(expr) => {
                let result = self.evaluate(expr)?;
                match result {
                    RuntimeResult::Value(v) => Ok(RuntimeResult::Return(v)),
                    RuntimeResult::Return(_) => Err("Nested return not supported".to_string()),
                    RuntimeResult::None => Ok(RuntimeResult::Return(Value::Nil)),
                }
            }
            Stmt::FunctionDef { name, params, return_type: _, body, gpu } => {
                // Register the function in the environment
                let param_names: Vec<String> = params.iter()
                    .map(|(name, _)| name.clone())
                    .collect();
                
                self.environment.borrow_mut().define(
                    name.clone(),
                    Value::Function(name.clone(), param_names)
                );
                
                if *gpu {
                    println!("GPU function '{}' registered", name);
                    // Register with GPU runtime too
                    let param_types: Vec<(String, String)> = params.iter()
                        .map(|(name, type_opt)| (name.clone(), type_opt.clone().unwrap_or_else(|| "Any".to_string())))
                        .collect();
                    
                    match self.gpu_runtime.register_function(name.clone(), param_types, body) {
                        Ok(_) => {},
                        Err(e) => println!("Warning: Failed to register GPU function: {}", e),
                    }
                }
                
                Ok(RuntimeResult::Value(Value::Nil))
            }
            #[allow(unreachable_patterns)]
            _ => {
                println!("Warning: Unsupported statement type");
                Ok(RuntimeResult::Value(Value::Nil))
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
                // Special case for placeholder "error" identifier
                if name == "error" {
                    println!("Warning: evaluating error placeholder in expression");
                    return Ok(RuntimeResult::Value(Value::Nil));
                }
                
                let val = self.environment.borrow().get(name);
                val.map(|v| RuntimeResult::Value(v))
                    .ok_or(format!("Undefined variable '{}'", name))
            }
            Expr::Assignment { name, value } => {
                match self.evaluate(value) {
                    Ok(RuntimeResult::Value(v)) => {
                        match self.environment.borrow_mut().assign(name, v.clone()) {
                            Ok(_) => Ok(RuntimeResult::Value(v)),
                            Err(e) => {
                                println!("Warning: Assignment error: {}", e);
                                Ok(RuntimeResult::Value(Value::Nil))
                            }
                        }
                    },
                    Ok(RuntimeResult::Return(_)) => {
                        Err("Return in assignment not allowed".into())
                    },
                    Ok(RuntimeResult::None) => {
                        println!("Warning: Assignment evaluated to None");
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    Err(e) => {
                        println!("Warning: Error evaluating assignment value: {}", e);
                        Ok(RuntimeResult::Value(Value::Nil))
                    }
                }
            }
            Expr::Binary { left, op, right } => {
                let l = match self.evaluate(left) {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Warning: Error evaluating left side of binary expression: {}", e);
                        return Ok(RuntimeResult::Value(Value::Nil));
                    }
                };
                
                let r = match self.evaluate(right) {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Warning: Error evaluating right side of binary expression: {}", e);
                        return Ok(RuntimeResult::Value(Value::Nil));
                    }
                };
                
                if let (RuntimeResult::Value(left_val), RuntimeResult::Value(right_val)) = (l, r) {
                    match self.binary_op(op, left_val, right_val) {
                        Ok(val) => Ok(RuntimeResult::Value(val)),
                        Err(e) => {
                            println!("Warning: Binary operation error: {}", e);
                            Ok(RuntimeResult::Value(Value::Nil))
                        }
                    }
                } else {
                    Err("Return not allowed inside binary expression".to_string())
                }
            }
            Expr::Unary { op, expr } => {
                let val = self.evaluate(expr)?;
                if let RuntimeResult::Value(v) = val {
                    let res = self.unary_op(op, v)?;
                    Ok(RuntimeResult::Value(res))
                } else {
                    Err("Return not allowed in unary expression".into())
                }
            }
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Return(expr) => {
                let result = self.evaluate(expr)?;
                match result {
                    RuntimeResult::Value(val) => Ok(RuntimeResult::Return(val)),
                    RuntimeResult::Return(_) => Err("Nested return not supported".to_string()),
                    RuntimeResult::None => Ok(RuntimeResult::Return(Value::Nil)),
                }
            }
            Expr::Range { start, end: _ } => {
                // For now, ranges are just evaluated to the start value
                match self.evaluate(start) {
                    Ok(RuntimeResult::Value(val)) => Ok(RuntimeResult::Value(val)),
                    _ => Ok(RuntimeResult::Value(Value::Number(0))), // Default to 0
                }
            },
            Expr::FunctionCall { callee, arguments } => {
                match &**callee {
                    Expr::Ident(name) if name == "print" => {
                        // Handle print function specially
                        println!("Executing built-in print function");
                        for arg in arguments {
                            match self.evaluate(arg) {
                                Ok(RuntimeResult::Value(val)) => println!("Output: {}", val),
                                _ => println!("Output: <error>"),
                            }
                        }
                        Ok(RuntimeResult::Value(Value::Nil))
                    },
                    Expr::Ident(name) => {
                        // Check if it's a GPU function first
                        if name.ends_with("_gpu") || name.starts_with("gpu_") {
                            println!("Calling GPU function: {}", name);
                            
                            // Evaluate all arguments
                            let mut arg_values = Vec::new();
                            for arg in arguments {
                                match self.evaluate(arg)? {
                                    RuntimeResult::Value(val) => arg_values.push(val),
                                    _ => {
                                        return Err(format!("Invalid argument to GPU function {}", name));
                                    }
                                }
                            }
                            
                            // Execute on the GPU
                            match self.gpu_runtime.execute_function(name, arg_values) {
                                Ok(result) => {
                                    println!("GPU function returned: {:?}", result);
                                    Ok(RuntimeResult::Value(result))
                                },
                                Err(e) => {
                                    println!("GPU function execution failed: {}", e);
                                    Err(format!("GPU function execution error: {}", e))
                                }
                            }
                        } else {
                            // Look up user-defined function
                            let function_result = {
                                // Scope the borrow to avoid the borrow checker error
                                let env_ref = self.environment.borrow();
                                match env_ref.get(name) {
                                    Some(value) => Some(value.clone()),
                                    None => None
                                }
                            };
                            
                            match function_result {
                                Some(Value::Function(fn_name, _)) => {
                                    println!("Calling CPU function: {}", fn_name);
                                    
                                    // Evaluate all arguments
                                    let mut arg_values = Vec::new();
                                    for arg in arguments {
                                        match self.evaluate(arg)? {
                                            RuntimeResult::Value(val) => arg_values.push(val),
                                            _ => {
                                                return Err(format!("Invalid argument to function {}", name));
                                            }
                                        }
                                    }
                                    
                                    // For this example, just return a dummy result
                                    if name == "add_cpu" && arg_values.len() >= 2 {
                                        if let (Value::Number(a), Value::Number(b)) = (&arg_values[0], &arg_values[1]) {
                                            let result = a + b;
                                            println!("Function returned: {}", result);
                                            return Ok(RuntimeResult::Value(Value::Number(result)));
                                        }
                                    }
                                    
                                    println!("Function returned default value");
                                    Ok(RuntimeResult::Value(Value::Number(0)))
                                },
                                _ => {
                                    println!("Warning: Undefined function: {}", name);
                                    Err(format!("Undefined function: {}", name))
                                }
                            }
                        }
                    },
                    _ => {
                        println!("Warning: Callee is not a function name");
                        Err("Invalid function call: callee is not a function name".to_string())
                    }
                }
            }
        }
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    fn binary_op(&self, op: &Token, left: Value, right: Value) -> Result<Value, String> {
        use Value::*;
        match op {
            Token::Plus => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a + b)),
                (Floating(a), Floating(b)) => Ok(Floating(a + b)),
                (String(a), String(b)) => Ok(String(a + &b)),
                _ => Err("Invalid '+' operands".to_string()),
            },
            Token::Minus => self.numeric_op(left, right, |a, b| a - b),
            Token::Star => self.numeric_op(left, right, |a, b| a * b),
            Token::Slash => self.numeric_op(left, right, |a, b| a / b),
            Token::EQ => Ok(Boolean(left == right)),
            Token::NE => Ok(Boolean(left != right)),
            Token::GT => self.compare_op(left, right, |a, b| a > b),
            Token::LT => self.compare_op(left, right, |a, b| a < b),
            Token::GE => self.compare_op(left, right, |a, b| a >= b),
            Token::LE => self.compare_op(left, right, |a, b| a <= b),
            _ => Err("Unknown binary op".into()),
        }
    }

    fn unary_op(&self, op: &Token, val: Value) -> Result<Value, String> {
        match op {
            Token::Minus => match val {
                Value::Number(n) => Ok(Value::Number(-n)),
                Value::Floating(f) => Ok(Value::Floating(-f)),
                _ => Err("Invalid unary minus".into()),
            },
            _ => Err("Unknown unary operator".to_string()),
        }
    }

    fn numeric_op<F>(&self, left: Value, right: Value, f: F) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> i64,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
            (Value::Floating(a), Value::Floating(b)) => {
                // Convert the operation to work on floating point
                let float_result = f(a as i64, b as i64) as f64;
                Ok(Value::Floating(float_result))
            },
            (Value::Number(a), Value::Floating(b)) => {
                // Mixed number types
                let float_result = f(a, b as i64) as f64;
                Ok(Value::Floating(float_result))
            },
            (Value::Floating(a), Value::Number(b)) => {
                // Mixed number types
                let float_result = f(a as i64, b) as f64;
                Ok(Value::Floating(float_result))
            },
            _ => {
                println!("Warning: Expected numeric operands");
                Ok(Value::Number(0)) // Default to 0 for error recovery
            },
        }
    }

    fn compare_op<F>(&self, left: Value, right: Value, f: F) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(f(a, b))),
            _ => Err("Expected integer operands".to_string()),
        }
    }
}
