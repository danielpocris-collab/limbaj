use std::fmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    I64,
    F64,
    Bool,
    Str,
    Void,
    Result { ok: Box<Type>, err: Box<Type> },
    Option { inner: Box<Type> },
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Void => write!(f, "void"),
            Type::Result { ok, err } => write!(f, "Result[{}, {}]", ok, err),
            Type::Option { inner } => write!(f, "Option[{}]", inner),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Let {
        name: String,
        var_type: Option<Type>,
        value: Expression,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    Match {
        expr: Expression,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Ok(String),
    Err(String),
    Some(String),
    None,
    Identifier(String),
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Identifier(String),
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    Ok(Box<Expression>),
    Err(Box<Expression>),
    Some(Box<Expression>),
    None,
    Block(Vec<Statement>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::I32), "i32");
        assert_eq!(format!("{}", Type::I64), "i64");
        assert_eq!(format!("{}", Type::Str), "str");
        assert_eq!(
            format!(
                "{}",
                Type::Result {
                    ok: Box::new(Type::I32),
                    err: Box::new(Type::Str)
                }
            ),
            "Result[i32, str]"
        );
    }

    #[test]
    fn test_type_equality() {
        assert_eq!(Type::I32, Type::I32);
        assert_ne!(Type::I32, Type::I64);
    }
}
