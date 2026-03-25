use crate::ast::*;
use std::collections::HashMap;

pub struct TypeChecker {
    functions: HashMap<String, (Vec<Type>, Type)>,
    variables: HashMap<String, Type>,
    current_return_type: Type,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            functions: HashMap::new(),
            variables: HashMap::new(),
            current_return_type: Type::Void,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        // First pass: collect function signatures
        for func in &program.functions {
            let param_types = func.params.iter().map(|p| p.param_type.clone()).collect();
            self.functions
                .insert(func.name.clone(), (param_types, func.return_type.clone()));
        }

        // Second pass: type check each function
        for func in &program.functions {
            self.check_function(func)?;
        }

        Ok(())
    }

    fn check_function(&mut self, func: &Function) -> Result<(), String> {
        self.variables.clear();
        self.current_return_type = func.return_type.clone();

        // Add parameters to scope
        for param in &func.params {
            self.variables.insert(param.name.clone(), param.param_type.clone());
        }

        // Check all statements
        for stmt in &func.body {
            self.check_statement(stmt)?;
        }

        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
            }
            Statement::Let { name, var_type, value } => {
                let value_type = self.check_expression(value)?;
                let final_type = match var_type {
                    Some(t) => {
                        if t != &value_type {
                            return Err(format!(
                                "Type mismatch in let binding '{}': expected {}, found {}",
                                name, t, value_type
                            ));
                        }
                        t.clone()
                    }
                    None => value_type,
                };
                self.variables.insert(name.clone(), final_type);
            }
            Statement::Return(expr_opt) => {
                match expr_opt {
                    Some(expr) => {
                        let expr_type = self.check_expression(expr)?;
                        if expr_type != self.current_return_type {
                            return Err(format!(
                                "Return type mismatch: expected {}, found {}",
                                self.current_return_type, expr_type
                            ));
                        }
                    }
                    None => {
                        if self.current_return_type != Type::Void {
                            return Err(format!(
                                "Expected return value of type {}",
                                self.current_return_type
                            ));
                        }
                    }
                }
            }
            Statement::If { condition, then_body, else_body } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!(
                        "If condition must be bool, found {}",
                        cond_type
                    ));
                }

                for stmt in then_body {
                    self.check_statement(stmt)?;
                }

                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Statement::Match { expr, arms } => {
                let expr_type = self.check_expression(expr)?;
                self.check_match(&expr_type, arms)?;
            }
        }
        Ok(())
    }

    fn check_match(&mut self, expr_type: &Type, arms: &[MatchArm]) -> Result<(), String> {
        match expr_type {
            Type::Result { ok, err } => {
                let mut has_ok = false;
                let mut has_err = false;

                for arm in arms {
                    match &arm.pattern {
                        Pattern::Ok(name) => {
                            has_ok = true;
                            self.variables.insert(name.clone(), (**ok).clone());
                        }
                        Pattern::Err(name) => {
                            has_err = true;
                            self.variables.insert(name.clone(), (**err).clone());
                        }
                        _ => return Err("Invalid pattern for Result type".to_string()),
                    }

                    for stmt in &arm.body {
                        self.check_statement(stmt)?;
                    }
                }

                if !has_ok || !has_err {
                    return Err("Match on Result must have Ok and Err patterns".to_string());
                }
            }
            Type::Option { inner } => {
                let mut has_some = false;
                let mut has_none = false;

                for arm in arms {
                    match &arm.pattern {
                        Pattern::Some(name) => {
                            has_some = true;
                            self.variables.insert(name.clone(), (**inner).clone());
                        }
                        Pattern::None => {
                            has_none = true;
                        }
                        _ => return Err("Invalid pattern for Option type".to_string()),
                    }

                    for stmt in &arm.body {
                        self.check_statement(stmt)?;
                    }
                }

                if !has_some || !has_none {
                    return Err("Match on Option must have Some and None patterns".to_string());
                }
            }
            _ => {
                return Err(format!(
                    "Cannot match on type {} (must be Result or Option)",
                    expr_type
                ))
            }
        }

        Ok(())
    }

    fn check_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Integer(_) => Ok(Type::I64),
            Expression::Float(_) => Ok(Type::F64),
            Expression::Bool(_) => Ok(Type::Bool),
            Expression::String(_) => Ok(Type::Str),
            Expression::Identifier(name) => {
                self.variables.get(name).cloned().ok_or_else(|| {
                    format!("Undefined variable: {}", name)
                })
            }
            Expression::Call { name, args } => {
                match self.functions.get(name).cloned() {
                    Some((param_types, return_type)) => {
                        if args.len() != param_types.len() {
                            return Err(format!(
                                "Function {} expects {} arguments, got {}",
                                name,
                                param_types.len(),
                                args.len()
                            ));
                        }

                        for (arg, expected_type) in args.iter().zip(param_types.iter()) {
                            let arg_type = self.check_expression(arg)?;
                            if arg_type != *expected_type {
                                return Err(format!(
                                    "Argument type mismatch: expected {}, found {}",
                                    expected_type, arg_type
                                ));
                            }
                        }

                        Ok(return_type.clone())
                    }
                    None => Err(format!("Undefined function: {}", name)),
                }
            }
            Expression::Binary { left, op, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        // Arithmetic operators require matching numeric types
                        match (&left_type, &right_type) {
                            (Type::I64, Type::I64) => {
                                Ok(Type::I64)
                            }
                            (Type::F64, Type::F64) => Ok(Type::F64),
                            _ => Err(format!(
                                "Type mismatch in arithmetic: {} {} {}",
                                left_type, op, right_type
                            )),
                        }
                    }
                    BinaryOp::Modulo => {
                        // Modulo only works with integers
                        match (&left_type, &right_type) {
                            (Type::I64, Type::I64) => Ok(Type::I64),
                            _ => Err(format!(
                                "Modulo operator requires integer types, got {} and {}",
                                left_type, right_type
                            )),
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if left_type != right_type {
                            return Err(format!(
                                "Cannot compare {} and {}",
                                left_type, right_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    BinaryOp::Less
                    | BinaryOp::Greater
                    | BinaryOp::LessEqual
                    | BinaryOp::GreaterEqual => {
                        // Comparison operators on numeric types
                        match (&left_type, &right_type) {
                            (Type::I64, Type::I64) => Ok(Type::Bool),
                            (Type::F64, Type::F64) => Ok(Type::Bool),
                            _ => Err(format!(
                                "Cannot compare {} and {}",
                                left_type, right_type
                            )),
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(format!(
                                "Logical operators require bool types, got {} and {}",
                                left_type, right_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Unary { op, operand } => {
                let operand_type = self.check_expression(operand)?;
                match op {
                    UnaryOp::Not => {
                        if operand_type != Type::Bool {
                            return Err(format!(
                                "! operator requires bool, found {}",
                                operand_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    UnaryOp::Negate => {
                        match operand_type {
                            Type::I64 | Type::F64 => Ok(operand_type),
                            _ => Err(format!(
                                "Negation requires numeric type, found {}",
                                operand_type
                            )),
                        }
                    }
                }
            }
            Expression::Ok(inner) => {
                let inner_type = self.check_expression(inner)?;
                Ok(Type::Result {
                    ok: Box::new(inner_type),
                    err: Box::new(Type::Str), // Default error type is string
                })
            }
            Expression::Err(inner) => {
                let inner_type = self.check_expression(inner)?;
                Ok(Type::Result {
                    ok: Box::new(Type::Str), // Default success type is string
                    err: Box::new(inner_type),
                })
            }
            Expression::Some(inner) => {
                let inner_type = self.check_expression(inner)?;
                Ok(Type::Option {
                    inner: Box::new(inner_type),
                })
            }
            Expression::None => {
                Ok(Type::Option {
                    inner: Box::new(Type::Str), // Default inner type is string
                })
            }
            Expression::Block(statements) => {
                let mut last_type = Type::Void;
                for stmt in statements {
                    match stmt {
                        Statement::Expression(expr) => {
                            last_type = self.check_expression(expr)?;
                        }
                        _ => {
                            self.check_statement(stmt)?;
                        }
                    }
                }
                Ok(last_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_check_simple() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            }],
        };
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }
}
