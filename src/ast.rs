use utils::PositiveFiniteF64;

pub mod utils;

pub enum Expr {
    Int(u64),
    Float(PositiveFiniteF64),
    Str(String),
    Bool(bool),
    EnumMember {
        enum_path: Vec<String>,
        label: String,
    },
    List(Vec<Expr>),

    Name(String),
    Attribute {
        value: Box<Expr>,
        attr_name: String,
    },
    MethodCall {
        value: Box<Expr>,
        method_name: String,
        args: Vec<Expr>,
    },

    UnaryOp {
        op: UnaryOp,
        v: Box<Expr>,
    },
    BinaryOp {
        l: Box<Expr>,
        op: BinaryOp,
        r: Box<Expr>,
    },
    CondOp {
        cond: Box<Expr>,
        if_true: Box<Expr>,
        if_false: Box<Expr>,
    },
    Subscript {
        value: Box<Expr>,
        idx: Box<Expr>,
    },
}

/// https://github.com/Mingun/ksc-rs/blob/7e6a82f0b6b09f9d7a6a9ae38e361b92f3a9c0e0/src/parser/expressions.rs#L274-L281
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UnaryOp {
    /// `-`: Negation
    Neg,
    /// `not`: Logical NOT
    Not,
    /// `~`: Bitwise NOT
    Inv,
}

/// https://github.com/Mingun/ksc-rs/blob/7e6a82f0b6b09f9d7a6a9ae38e361b92f3a9c0e0/src/parser/expressions.rs#L285-L326
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BinaryOp {
    /// `+`: Addition or concatenation
    Add,
    /// `-`: Subtraction
    Sub,
    /// `*`: Multiplication
    Mul,
    /// `/`: Division
    Div,
    /// `%`: Remainder of division
    Rem,

    /// `==`: Equal to
    Eq,
    /// `!=`: Not equal to
    Ne,
    /// `<`: Less than
    Lt,
    /// `<=`: Less than or equal to
    Le,
    /// `>`: Greater than
    Gt,
    /// `>=`: Greater than or equal to
    Ge,

    /// `and`: Logical AND
    And,
    /// `or`: Logical OR
    Or,

    /// `|`: Bitwise OR
    BitOr,
    /// `^`: Bitwise XOR
    BitXor,
    /// `&`: Bitwise AND
    BitAnd,

    /// `<<`: Bitwise left shift
    Shl,
    /// `>>`: Bitwise right shift
    Shr,
}
