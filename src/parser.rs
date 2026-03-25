use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    lexer: Lexer,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self, String> {
        let tokens = lexer.tokenize()?;
        Ok(Parser {
            tokens,
            pos: 0,
            lexer,
        })
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: TokenType) -> Result<(), String> {
        match self.current() {
            Some(token) if std::mem::discriminant(&token.token_type) == std::mem::discriminant(&expected) => {
                self.advance();
                Ok(())
            }
            Some(token) => {
                Err(format!(
                    "{}:{} Expected {:?}, found {:?}",
                    token.line, token.col, expected, token.token_type
                ))
            }
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn get_identifier(&self, idx: usize) -> String {
        self.lexer.get_identifier(idx).unwrap_or("?").to_string()
    }

    fn get_string(&self, idx: usize) -> String {
        self.lexer.get_string(idx).unwrap_or("?").to_string()
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();

        while let Some(token) = self.current() {
            if token.token_type == TokenType::Eof {
                break;
            }
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(TokenType::Fn)?;

        let name = match self.current() {
            Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                let name = self.get_identifier(*idx);
                self.advance();
                name
            }
            _ => return Err("Expected function name".to_string()),
        };

        self.expect(TokenType::LeftParen)?;

        let params = self.parse_params()?;

        self.expect(TokenType::RightParen)?;
        self.expect(TokenType::Arrow)?;

        let return_type = self.parse_type()?;

        self.expect(TokenType::LeftBrace)?;

        let body = self.parse_statements()?;

        self.expect(TokenType::RightBrace)?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();

        loop {
            match self.current() {
                Some(Token { token_type: TokenType::RightParen, .. }) => break,
                Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                    let name = self.get_identifier(*idx);
                    self.advance();

                    self.expect(TokenType::Colon)?;
                    let param_type = self.parse_type()?;

                    params.push(Parameter { name, param_type });

                    match self.current() {
                        Some(Token { token_type: TokenType::Comma, .. }) => {
                            self.advance();
                        }
                        Some(Token { token_type: TokenType::RightParen, .. }) => break,
                        _ => return Err("Expected comma or right paren in params".to_string()),
                    }
                }
                _ => return Err("Expected parameter name".to_string()),
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current() {
            Some(Token { token_type: TokenType::I32, .. }) => {
                self.advance();
                Ok(Type::I32)
            }
            Some(Token { token_type: TokenType::I64, .. }) => {
                self.advance();
                Ok(Type::I64)
            }
            Some(Token { token_type: TokenType::F64, .. }) => {
                self.advance();
                Ok(Type::F64)
            }
            Some(Token { token_type: TokenType::Bool, .. }) => {
                self.advance();
                Ok(Type::Bool)
            }
            Some(Token { token_type: TokenType::Str, .. }) => {
                self.advance();
                Ok(Type::Str)
            }
            Some(Token { token_type: TokenType::Void, .. }) => {
                self.advance();
                Ok(Type::Void)
            }
            Some(Token { token_type: TokenType::Result, .. }) => {
                self.advance();
                self.expect(TokenType::LeftBracket)?;
                let ok = Box::new(self.parse_type()?);
                self.expect(TokenType::Comma)?;
                let err = Box::new(self.parse_type()?);
                self.expect(TokenType::RightBracket)?;
                Ok(Type::Result { ok, err })
            }
            Some(Token { token_type: TokenType::Option, .. }) => {
                self.advance();
                self.expect(TokenType::LeftBracket)?;
                let inner = Box::new(self.parse_type()?);
                self.expect(TokenType::RightBracket)?;
                Ok(Type::Option { inner })
            }
            Some(token) => {
                Err(format!(
                    "{}:{} Expected type, found {:?}",
                    token.line, token.col, token.token_type
                ))
            }
            None => Err("Unexpected end of input while parsing type".to_string()),
        }
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        loop {
            match self.current() {
                Some(Token { token_type: TokenType::RightBrace, .. }) => break,
                Some(Token { token_type: TokenType::Eof, .. }) => {
                    return Err("Unexpected end of input while parsing statements".to_string())
                }
                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token { token_type: TokenType::Let, .. }) => self.parse_let(),
            Some(Token { token_type: TokenType::Return, .. }) => self.parse_return(),
            Some(Token { token_type: TokenType::If, .. }) => self.parse_if(),
            Some(Token { token_type: TokenType::Match, .. }) => self.parse_match(),
            _ => {
                let expr = self.parse_expression()?;
                self.expect(TokenType::Semicolon)?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Let)?;

        let name = match self.current() {
            Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                let name = self.get_identifier(*idx);
                self.advance();
                name
            }
            _ => return Err("Expected variable name".to_string()),
        };

        let var_type = match self.current() {
            Some(Token { token_type: TokenType::Colon, .. }) => {
                self.advance();
                Some(self.parse_type()?)
            }
            _ => None,
        };

        self.expect(TokenType::Equals)?;
        let value = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;

        Ok(Statement::Let { name, var_type, value })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Return)?;

        match self.current() {
            Some(Token { token_type: TokenType::Semicolon, .. }) => {
                self.advance();
                Ok(Statement::Return(None))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(TokenType::Semicolon)?;
                Ok(Statement::Return(Some(expr)))
            }
        }
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::If)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::LeftBrace)?;
        let then_body = self.parse_statements()?;
        self.expect(TokenType::RightBrace)?;

        let else_body = match self.current() {
            Some(Token { token_type: TokenType::Else, .. }) => {
                self.advance();
                self.expect(TokenType::LeftBrace)?;
                let body = self.parse_statements()?;
                self.expect(TokenType::RightBrace)?;
                Some(body)
            }
            _ => None,
        };

        Ok(Statement::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_match(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Match)?;
        let expr = self.parse_expression()?;
        self.expect(TokenType::LeftBrace)?;

        let mut arms = Vec::new();
        loop {
            match self.current() {
                Some(Token { token_type: TokenType::RightBrace, .. }) => break,
                _ => {
                    let pattern = self.parse_pattern()?;
                    self.expect(TokenType::FatArrow)?;
                    let body = self.parse_statements()?;
                    arms.push(MatchArm { pattern, body });
                }
            }
        }

        self.expect(TokenType::RightBrace)?;

        if arms.is_empty() {
            return Err("Match expression must have at least one arm".to_string());
        }

        Ok(Statement::Match { expr, arms })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match self.current() {
            Some(Token { token_type: TokenType::Ok, .. }) => {
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let name = match self.current() {
                    Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                        let name = self.get_identifier(*idx);
                        self.advance();
                        name
                    }
                    _ => return Err("Expected identifier in pattern".to_string()),
                };
                self.expect(TokenType::RightParen)?;
                Ok(Pattern::Ok(name))
            }
            Some(Token { token_type: TokenType::Err, .. }) => {
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let name = match self.current() {
                    Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                        let name = self.get_identifier(*idx);
                        self.advance();
                        name
                    }
                    _ => return Err("Expected identifier in pattern".to_string()),
                };
                self.expect(TokenType::RightParen)?;
                Ok(Pattern::Err(name))
            }
            Some(Token { token_type: TokenType::Some, .. }) => {
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let name = match self.current() {
                    Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                        let name = self.get_identifier(*idx);
                        self.advance();
                        name
                    }
                    _ => return Err("Expected identifier in pattern".to_string()),
                };
                self.expect(TokenType::RightParen)?;
                Ok(Pattern::Some(name))
            }
            Some(Token { token_type: TokenType::None, .. }) => {
                self.advance();
                Ok(Pattern::None)
            }
            Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                let name = self.get_identifier(*idx);
                self.advance();
                Ok(Pattern::Identifier(name))
            }
            Some(_) => {
                // Wildcard pattern
                Ok(Pattern::Wildcard)
            }
            None => Err("Unexpected end of input while parsing pattern".to_string()),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and()?;

        while let Some(Token { token_type: TokenType::Or, .. }) = self.current() {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;

        while let Some(Token { token_type: TokenType::And, .. }) = self.current() {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.current() {
                Some(Token { token_type: TokenType::DoubleEquals, .. }) => BinaryOp::Equal,
                Some(Token { token_type: TokenType::NotEquals, .. }) => BinaryOp::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_addition()?;

        loop {
            let op = match self.current() {
                Some(Token { token_type: TokenType::Less, .. }) => BinaryOp::Less,
                Some(Token { token_type: TokenType::Greater, .. }) => BinaryOp::Greater,
                Some(Token { token_type: TokenType::LessEquals, .. }) => BinaryOp::LessEqual,
                Some(Token { token_type: TokenType::GreaterEquals, .. }) => BinaryOp::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;

        loop {
            let op = match self.current() {
                Some(Token { token_type: TokenType::Plus, .. }) => BinaryOp::Add,
                Some(Token { token_type: TokenType::Minus, .. }) => BinaryOp::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current() {
                Some(Token { token_type: TokenType::Star, .. }) => BinaryOp::Multiply,
                Some(Token { token_type: TokenType::Slash, .. }) => BinaryOp::Divide,
                Some(Token { token_type: TokenType::Percent, .. }) => BinaryOp::Modulo,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.current() {
            Some(Token { token_type: TokenType::Not, .. }) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            Some(Token { token_type: TokenType::Minus, .. }) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.current() {
            Some(Token { token_type: TokenType::Integer(n), .. }) => {
                let n = *n;
                self.advance();
                Ok(Expression::Integer(n))
            }
            Some(Token { token_type: TokenType::Float(f), .. }) => {
                let f = *f;
                self.advance();
                Ok(Expression::Float(f))
            }
            Some(Token { token_type: TokenType::True, .. }) => {
                self.advance();
                Ok(Expression::Bool(true))
            }
            Some(Token { token_type: TokenType::False, .. }) => {
                self.advance();
                Ok(Expression::Bool(false))
            }
            Some(Token { token_type: TokenType::String(idx), .. }) => {
                let s = self.get_string(*idx);
                self.advance();
                Ok(Expression::String(s))
            }
            Some(Token { token_type: TokenType::Identifier(idx), .. }) => {
                let name = self.get_identifier(*idx);
                self.advance();

                // Check if this is a function call
                if let Some(Token { token_type: TokenType::LeftParen, .. }) = self.current() {
                    self.advance();
                    let mut args = Vec::new();

                    loop {
                        match self.current() {
                            Some(Token { token_type: TokenType::RightParen, .. }) => break,
                            _ => {
                                args.push(self.parse_expression()?);
                                match self.current() {
                                    Some(Token { token_type: TokenType::Comma, .. }) => {
                                        self.advance();
                                    }
                                    Some(Token { token_type: TokenType::RightParen, .. }) => break,
                                    _ => return Err("Expected comma or right paren in args".to_string()),
                                }
                            }
                        }
                    }

                    self.expect(TokenType::RightParen)?;
                    Ok(Expression::Call { name, args })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Some(Token { token_type: TokenType::Ok, .. }) => {
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(Expression::Ok(Box::new(expr)))
            }
            Some(Token { token_type: TokenType::Err, .. }) => {
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(Expression::Err(Box::new(expr)))
            }
            Some(Token { token_type: TokenType::Some, .. }) => {
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(Expression::Some(Box::new(expr)))
            }
            Some(Token { token_type: TokenType::None, .. }) => {
                self.advance();
                Ok(Expression::None)
            }
            Some(Token { token_type: TokenType::LeftParen, .. }) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(expr)
            }
            Some(Token { token_type: TokenType::LeftBrace, .. }) => {
                self.advance();
                let body = self.parse_statements()?;
                self.expect(TokenType::RightBrace)?;
                Ok(Expression::Block(body))
            }
            Some(token) => {
                Err(format!(
                    "{}:{} Unexpected token {:?}",
                    token.line, token.col, token.token_type
                ))
            }
            None => Err("Unexpected end of input".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let lexer = Lexer::new("fn main() -> void { }");
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse().unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }

    #[test]
    fn test_parse_function_with_params() {
        let lexer = Lexer::new("fn add(a: i32, b: i32) -> i32 { }");
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse().unwrap();
        assert_eq!(program.functions[0].params.len(), 2);
        assert_eq!(program.functions[0].params[0].name, "a");
    }
}
