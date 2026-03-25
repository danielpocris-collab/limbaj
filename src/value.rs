use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Unit,
    Ok(Box<Value>),
    Err(Box<Value>),
    Some(Box<Value>),
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::I64(n) => write!(f, "{}", n),
            Value::F64(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Unit => write!(f, "()"),
            Value::Ok(v) => write!(f, "Ok({})", v),
            Value::Err(v) => write!(f, "Err({})", v),
            Value::Some(v) => write!(f, "Some({})", v),
            Value::None => write!(f, "None"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::F64(a), Value::F64(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Ok(a), Value::Ok(b)) => a == b,
            (Value::Err(a), Value::Err(b)) => a == b,
            (Value::Some(a), Value::Some(b)) => a == b,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::I64(a), Value::I64(b)) => a.partial_cmp(b),
            (Value::F64(a), Value::F64(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::I64(42), Value::I64(42));
        assert_ne!(Value::I64(42), Value::I64(43));
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::I64(42)), "42");
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
    }
}
