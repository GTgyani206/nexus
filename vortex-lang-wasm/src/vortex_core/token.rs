// || shree ganesh ||
// This file will be containing the token definitions for the Vortex programming language.
// why a separate file for tokens? well cuz we want to keep the code organized and maintainable. And avoid circular dependencies!!.
//

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    //Keywords
    Let,
    Mut,
    If,
    Then,
    Else,
    For,
    In,
    Range,
    Fn,
    Return,

    //VORTEX mode
    Branch,
    Fallback,
    Parallel,
    GPU,

    //Symbols
    Equals,   // =
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Lparen,   // (
    Rparen,   // )
    LBrace,   // {
    RBrace,   // }
    Lsquare,  // [
    Rsquare,  // ]
    Dot,      // .
    Comma,    // ,
    Colon,    // :
    Arrow,    // ->
    FatArrow, // =>

    //Comparision sign
    GT, // >
    LT, // <
    GE, // >=
    LE, // <=
    EQ, // ==
    NE, // !=

    // Literals
    Number(i64),
    Floating(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

    //Other
    EOF,
}
