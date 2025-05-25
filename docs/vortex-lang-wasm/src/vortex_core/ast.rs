//|| shree ganesh ||
// this code will be containing the AST structures

//AST  is Abstract Syntax Tree which represents the structure of a Vortex program.
//GOAL: To know what the character means that are recieved from the lexer
//Defining the AST structures

use super::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Floating(f64),
    Boolean(bool),
    String(String),
    Ident(String),
    Unary {
        op: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
    },
    Grouping(Box<Expr>),
    FunctionCall {
        callee: Box<Expr>,
        arguments: Vec<Expr>, // not boxed unless needed
    },
    Return(Box<Expr>),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        type_name: Option<String>,
        value: Expr,
        mutable: bool,
    },
    ExprStmt(Expr),
    Block(Vec<Stmt>),

    IfStmt {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    FunctionDef {
        name: String,
        params: Vec<(String, Option<String>)>, // name + optional type
        return_type: Option<String>,
        body: Box<Stmt>,
        gpu: bool,
    },

    For {
        var: String,
        range: Expr,
        body: Box<Stmt>,
    },
    Parallel {
        var: String,
        range: Expr,
        body: Box<Stmt>,
    },

    Branch {
        condition: Expr,
        body: Box<Stmt>,
    },
    Fallback(Box<Stmt>),
    Return(Expr),
}
