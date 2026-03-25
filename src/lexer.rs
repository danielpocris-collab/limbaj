#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Literals
    Integer(i64),
    Float(f64),
    String(usize), // Index into string pool
    True,
    False,

    // Keywords
    Fn,
    Let,
    Return,
    If,
    Else,
    Match,
    Ok,
    Err,
    Some,
    None,
    Result,
    Option,

    // Identifiers and types
    Identifier(usize), // Index into identifier pool
    I32,
    I64,
    F64,
    Bool,
    Str,
    Void,

    // Operators and punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,        // ->
    FatArrow,     // =>
    Equals,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleEquals,
    NotEquals,
    Less,
    Greater,
    LessEquals,
    GreaterEquals,
    And,          // &&
    Or,           // ||
    Not,          // !

    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    string_pool: Vec<String>,
    identifier_pool: Vec<String>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            string_pool: Vec::new(),
            identifier_pool: Vec::new(),
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        if self.pos + offset < self.input.len() {
            Some(self.input[self.pos + offset])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.current() == Some('/') && self.peek(1) == Some('/') {
            self.advance();
            self.advance();
            while let Some(ch) = self.current() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        self.advance(); // consume opening quote
        let mut result = String::new();

        loop {
            match self.current() {
                Some('"') => {
                    self.advance();
                    return Ok(result);
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('\\') => result.push('\\'),
                        Some('"') => result.push('"'),
                        _ => return Err("Invalid escape sequence".to_string()),
                    }
                    self.advance();
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
                None => return Err("Unterminated string".to_string()),
            }
        }
    }

    fn read_number(&mut self) -> Result<TokenType, String> {
        let mut number = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            number.parse::<f64>()
                .map(TokenType::Float)
                .map_err(|_| "Invalid float".to_string())
        } else {
            number.parse::<i64>()
                .map(TokenType::Integer)
                .map_err(|_| "Invalid integer".to_string())
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();

        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        ident
    }

    fn add_string(&mut self, s: String) -> usize {
        self.string_pool.push(s);
        self.string_pool.len() - 1
    }

    fn add_identifier(&mut self, ident: String) -> usize {
        self.identifier_pool.push(ident);
        self.identifier_pool.len() - 1
    }

    fn keyword_or_identifier(&mut self, ident: &str) -> TokenType {
        match ident {
            "fn" => TokenType::Fn,
            "let" => TokenType::Let,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "match" => TokenType::Match,
            "Ok" => TokenType::Ok,
            "Err" => TokenType::Err,
            "Some" => TokenType::Some,
            "None" => TokenType::None,
            "Result" => TokenType::Result,
            "Option" => TokenType::Option,
            "i32" => TokenType::I32,
            "i64" => TokenType::I64,
            "f64" => TokenType::F64,
            "bool" => TokenType::Bool,
            "str" => TokenType::Str,
            "void" => TokenType::Void,
            "true" => TokenType::True,
            "false" => TokenType::False,
            _ => TokenType::Identifier(self.add_identifier(ident.to_string())),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        loop {
            self.skip_whitespace();

            if self.current() == Some('/') && self.peek(1) == Some('/') {
                self.skip_comment();
                continue;
            }
            break;
        }

        let line = self.line;
        let col = self.col;

        let token_type = match self.current() {
            None => TokenType::Eof,
            Some('(') => {
                self.advance();
                TokenType::LeftParen
            }
            Some(')') => {
                self.advance();
                TokenType::RightParen
            }
            Some('{') => {
                self.advance();
                TokenType::LeftBrace
            }
            Some('}') => {
                self.advance();
                TokenType::RightBrace
            }
            Some('[') => {
                self.advance();
                TokenType::LeftBracket
            }
            Some(']') => {
                self.advance();
                TokenType::RightBracket
            }
            Some(',') => {
                self.advance();
                TokenType::Comma
            }
            Some(';') => {
                self.advance();
                TokenType::Semicolon
            }
            Some(':') => {
                self.advance();
                TokenType::Colon
            }
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenType::DoubleEquals
                } else if self.current() == Some('>') {
                    self.advance();
                    TokenType::FatArrow
                } else {
                    TokenType::Equals
                }
            }
            Some('-') => {
                self.advance();
                if self.current() == Some('>') {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            Some('+') => {
                self.advance();
                TokenType::Plus
            }
            Some('*') => {
                self.advance();
                TokenType::Star
            }
            Some('/') => {
                self.advance();
                TokenType::Slash
            }
            Some('%') => {
                self.advance();
                TokenType::Percent
            }
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenType::NotEquals
                } else {
                    TokenType::Not
                }
            }
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenType::LessEquals
                } else {
                    TokenType::Less
                }
            }
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenType::GreaterEquals
                } else {
                    TokenType::Greater
                }
            }
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    TokenType::And
                } else {
                    return Err("Unexpected character: &".to_string());
                }
            }
            Some('|') => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    TokenType::Or
                } else {
                    return Err("Unexpected character: |".to_string());
                }
            }
            Some('"') => {
                let s = self.read_string()?;
                TokenType::String(self.add_string(s))
            }
            Some(ch) if ch.is_ascii_digit() => self.read_number()?,
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                self.keyword_or_identifier(&ident)
            }
            Some(ch) => return Err(format!("Unexpected character: {}", ch)),
        };

        Ok(Token { token_type, line, col })
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn get_string(&self, idx: usize) -> Option<&str> {
        self.string_pool.get(idx).map(|s| s.as_str())
    }

    pub fn get_identifier(&self, idx: usize) -> Option<&str> {
        self.identifier_pool.get(idx).map(|s| s.as_str())
    }

    pub fn string_pool(&self) -> &[String] {
        &self.string_pool
    }

    pub fn identifier_pool(&self) -> &[String] {
        &self.identifier_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("fn main() { }");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token_type, TokenType::Fn);
        assert_eq!(tokens[1].token_type, TokenType::Identifier(0));
        assert_eq!(tokens[2].token_type, TokenType::LeftParen);
        assert_eq!(tokens[3].token_type, TokenType::RightParen);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Integer(42)));
        assert!(matches!(tokens[1].token_type, TokenType::Float(f) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize().unwrap();
        match tokens[0].token_type {
            TokenType::String(idx) => {
                assert_eq!(lexer.get_string(idx).unwrap(), "hello world");
            }
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != -> =>");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[4].token_type, TokenType::DoubleEquals);
        assert_eq!(tokens[5].token_type, TokenType::NotEquals);
        assert_eq!(tokens[6].token_type, TokenType::Arrow);
        assert_eq!(tokens[7].token_type, TokenType::FatArrow);
    }
}
