use crate::ast::*;
use crate::value::Value;
use std::collections::HashMap;

pub struct Interpreter {
    functions: HashMap<String, Function>,
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new(program: &Program) -> Self {
        let mut functions = HashMap::new();
        for func in &program.functions {
            functions.insert(func.name.clone(), func.clone());
        }

        Interpreter {
            functions,
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
        }
    }

    pub fn run(&mut self) -> Result<Value, String> {
        match self.functions.get("main") {
            Some(func) => {
                if !func.params.is_empty() {
                    return Err("main() must not take parameters".to_string());
                }
                self.call_function("main", &[])
            }
            None => Err("No main() function found".to_string()),
        }
    }

    fn call_function(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let func = self.functions.get(name).cloned().ok_or_else(|| {
            format!("Undefined function: {}", name)
        })?;

        // Create new scope
        let mut new_scope = HashMap::new();

        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            new_scope.insert(param.name.clone(), arg.clone());
        }

        self.locals.push(new_scope);

        // Execute function body
        let mut result = Value::Unit;
        for stmt in &func.body {
            result = self.execute_statement(stmt)?;

            // Check if we should return
            if matches!(result, Value::Unit) {
                continue;
            } else {
                break;
            }
        }

        self.locals.pop();

        Ok(result)
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<Value, String> {
        match stmt {
            Statement::Expression(expr) => self.evaluate_expression(expr),
            Statement::Let { name, var_type: _, value } => {
                let val = self.evaluate_expression(value)?;
                if let Some(scope) = self.locals.last_mut() {
                    scope.insert(name.clone(), val);
                }
                Ok(Value::Unit)
            }
            Statement::Return(expr_opt) => {
                match expr_opt {
                    Some(expr) => self.evaluate_expression(expr),
                    None => Ok(Value::Unit),
                }
            }
            Statement::If { condition, then_body, else_body } => {
                let cond_val = self.evaluate_expression(condition)?;

                match cond_val {
                    Value::Bool(true) => {
                        let mut result = Value::Unit;
                        for stmt in then_body {
                            result = self.execute_statement(stmt)?;
                            if !matches!(result, Value::Unit) {
                                return Ok(result);
                            }
                        }
                        Ok(result)
                    }
                    Value::Bool(false) => {
                        if let Some(else_stmts) = else_body {
                            let mut result = Value::Unit;
                            for stmt in else_stmts {
                                result = self.execute_statement(stmt)?;
                                if !matches!(result, Value::Unit) {
                                    return Ok(result);
                                }
                            }
                            Ok(result)
                        } else {
                            Ok(Value::Unit)
                        }
                    }
                    _ => Err("Condition must be bool".to_string()),
                }
            }
            Statement::Match { expr, arms } => {
                let val = self.evaluate_expression(expr)?;

                for arm in arms {
                    if let Some(binding) = self.match_pattern(&arm.pattern, &val) {
                        // Add binding to scope
                        if let Some(scope) = self.locals.last_mut() {
                            if let Some((name, bound_val)) = binding {
                                scope.insert(name, bound_val);
                            }
                        }

                        // Execute arm body
                        let mut result = Value::Unit;
                        for stmt in &arm.body {
                            result = self.execute_statement(stmt)?;
                            if !matches!(result, Value::Unit) {
                                return Ok(result);
                            }
                        }
                        return Ok(result);
                    }
                }

                Err("No matching pattern in match".to_string())
            }
        }
    }

    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Option<(String, Value)>> {
        match (pattern, value) {
            (Pattern::Ok(name), Value::Ok(v)) => Some(Some((name.clone(), (**v).clone()))),
            (Pattern::Err(name), Value::Err(v)) => Some(Some((name.clone(), (**v).clone()))),
            (Pattern::Some(name), Value::Some(v)) => Some(Some((name.clone(), (**v).clone()))),
            (Pattern::None, Value::None) => Some(None),
            (Pattern::Identifier(name), v) => Some(Some((name.clone(), v.clone()))),
            (Pattern::Wildcard, _) => Some(None),
            _ => None,
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Integer(n) => Ok(Value::I64(*n)),
            Expression::Float(f) => Ok(Value::F64(*f)),
            Expression::Bool(b) => Ok(Value::Bool(*b)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Identifier(name) => self.get_variable(name),
            Expression::Call { name, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }
                self.call_function(name, &arg_values)
            }
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(&left_val, *op, &right_val)
            }
            Expression::Unary { op, operand } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.evaluate_unary_op(*op, &operand_val)
            }
            Expression::Ok(inner) => {
                let val = self.evaluate_expression(inner)?;
                Ok(Value::Ok(Box::new(val)))
            }
            Expression::Err(inner) => {
                let val = self.evaluate_expression(inner)?;
                Ok(Value::Err(Box::new(val)))
            }
            Expression::Some(inner) => {
                let val = self.evaluate_expression(inner)?;
                Ok(Value::Some(Box::new(val)))
            }
            Expression::None => Ok(Value::None),
            Expression::Block(statements) => {
                let mut result = Value::Unit;
                for stmt in statements {
                    result = self.execute_statement(stmt)?;
                    if !matches!(result, Value::Unit) {
                        return Ok(result);
                    }
                }
                Ok(result)
            }
        }
    }

    fn evaluate_binary_op(&self, left: &Value, op: BinaryOp, right: &Value) -> Result<Value, String> {
        match (left, right, op) {
            (Value::I64(a), Value::I64(b), BinaryOp::Add) => Ok(Value::I64(a + b)),
            (Value::I64(a), Value::I64(b), BinaryOp::Subtract) => Ok(Value::I64(a - b)),
            (Value::I64(a), Value::I64(b), BinaryOp::Multiply) => Ok(Value::I64(a * b)),
            (Value::I64(a), Value::I64(b), BinaryOp::Divide) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::I64(a / b))
                }
            }
            (Value::I64(a), Value::I64(b), BinaryOp::Modulo) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::I64(a % b))
                }
            }

            (Value::F64(a), Value::F64(b), BinaryOp::Add) => Ok(Value::F64(a + b)),
            (Value::F64(a), Value::F64(b), BinaryOp::Subtract) => Ok(Value::F64(a - b)),
            (Value::F64(a), Value::F64(b), BinaryOp::Multiply) => Ok(Value::F64(a * b)),
            (Value::F64(a), Value::F64(b), BinaryOp::Divide) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::F64(a / b))
                }
            }

            // Comparison operators
            (a, b, BinaryOp::Equal) => Ok(Value::Bool(a == b)),
            (a, b, BinaryOp::NotEqual) => Ok(Value::Bool(a != b)),

            (Value::I64(a), Value::I64(b), BinaryOp::Less) => Ok(Value::Bool(a < b)),
            (Value::I64(a), Value::I64(b), BinaryOp::Greater) => Ok(Value::Bool(a > b)),
            (Value::I64(a), Value::I64(b), BinaryOp::LessEqual) => Ok(Value::Bool(a <= b)),
            (Value::I64(a), Value::I64(b), BinaryOp::GreaterEqual) => Ok(Value::Bool(a >= b)),

            (Value::F64(a), Value::F64(b), BinaryOp::Less) => Ok(Value::Bool(a < b)),
            (Value::F64(a), Value::F64(b), BinaryOp::Greater) => Ok(Value::Bool(a > b)),
            (Value::F64(a), Value::F64(b), BinaryOp::LessEqual) => Ok(Value::Bool(a <= b)),
            (Value::F64(a), Value::F64(b), BinaryOp::GreaterEqual) => Ok(Value::Bool(a >= b)),

            (Value::Bool(a), Value::Bool(b), BinaryOp::And) => Ok(Value::Bool(*a && *b)),
            (Value::Bool(a), Value::Bool(b), BinaryOp::Or) => Ok(Value::Bool(*a || *b)),

            _ => Err(format!(
                "Type mismatch in binary operation: {} {:?} {}",
                left, op, right
            )),
        }
    }

    fn evaluate_unary_op(&self, op: UnaryOp, operand: &Value) -> Result<Value, String> {
        match (op, operand) {
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnaryOp::Negate, Value::I64(n)) => Ok(Value::I64(-n)),
            (UnaryOp::Negate, Value::F64(f)) => Ok(Value::F64(-f)),
            _ => Err(format!(
                "Invalid unary operation: {:?} on {}",
                op, operand
            )),
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        // Search from innermost to outermost scope
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }

        if let Some(val) = self.globals.get(name) {
            return Ok(val.clone());
        }

        Err(format!("Undefined variable: {}", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::I64,
                body: vec![Statement::Return(Some(Expression::Integer(42)))],
            }],
        };

        let mut interpreter = Interpreter::new(&program);
        let result = interpreter.run().unwrap();
        assert_eq!(result, Value::I64(42));
    }

    #[test]
    fn test_arithmetic() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::I64,
                body: vec![Statement::Return(Some(Expression::Binary {
                    left: Box::new(Expression::Integer(2)),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Integer(3)),
                }))],
            }],
        };

        let mut interpreter = Interpreter::new(&program);
        let result = interpreter.run().unwrap();
        assert_eq!(result, Value::I64(5));
    }
}
