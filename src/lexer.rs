#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    If,
    Else,
    While,

    // Identifiers and Literals
    Ident(String),
    Int(i32),
    Str(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash, // +, -, *, /
    Less,
    Greater,
    EqualEqual, // <, >, ==
    Assign,     // =

    // Delimiters
    LParen,
    RParen, // ( )
    LBrace,
    RBrace, // { }
    Semicolon,
    Comma,

    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == Token::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let ch = match self.peek() {
            Some(c) => c,
            None => return Token::EOF,
        };

        match ch {
            // Single-character symbols
            '+' => {
                self.advance();
                Token::Plus
            }
            '-' => {
                self.advance();
                Token::Minus
            }
            '*' => {
                self.advance();
                Token::Star
            }
            '/' => {
                self.advance();
                Token::Slash
            }
            '<' => {
                self.advance();
                Token::Less
            }
            '>' => {
                self.advance();
                Token::Greater
            }
            '(' => {
                self.advance();
                Token::LParen
            }
            ')' => {
                self.advance();
                Token::RParen
            }
            '{' => {
                self.advance();
                Token::LBrace
            }
            '}' => {
                self.advance();
                Token::RBrace
            }
            ';' => {
                self.advance();
                Token::Semicolon
            }
            ',' => {
                self.advance();
                Token::Comma
            }

            // Potential multi-character symbols
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqualEqual
                } else {
                    Token::Assign
                }
            }

            '"' => self.lex_string(),

            // Numeric literals
            _ if ch.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Token::Int(num_str.parse().unwrap())
            }

            // Identifiers and Keywords
            _ if ch.is_alphabetic() || ch == '_' => {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    _ => Token::Ident(ident),
                }
            }

            _ => panic!("Unexpected character: {}", ch),
        }
    }

    fn lex_string(&mut self) -> Token {
        self.advance(); // Consume the opening quote '"'
        let mut string_content = String::new();

        // We keep the quotes in the string so the C generator
        // treats them as literals: "content"
        string_content.push('"');

        while let Some(c) = self.peek() {
            if c == '"' {
                string_content.push('"');
                self.advance(); // Consume the closing quote
                break;
            }
            string_content.push(c);
            self.advance();

            if self.pos >= self.input.len() {
                panic!("Unterminated string literal");
            }
        }

        Token::Str(string_content)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_tokens(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize()
    }

    #[test]
    fn test_basic_operators() {
        let input = "+ - * / = == < >";
        let tokens = get_tokens(input);
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Assign,
                Token::EqualEqual,
                Token::Less,
                Token::Greater,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_keywords_and_idents() {
        let input = "if x else while_loop while";
        let tokens = get_tokens(input);
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Ident("x".to_string()),
                Token::Else,
                Token::Ident("while_loop".to_string()), // Should NOT be the keyword While
                Token::While,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_numeric_literals() {
        let input = "0 123 456789";
        let tokens = get_tokens(input);
        assert_eq!(
            tokens,
            vec![
                Token::Int(0),
                Token::Int(123),
                Token::Int(456789),
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let input = "x = (y + 10) * 2 / 5;";
        let tokens = get_tokens(input);
        assert_eq!(
            tokens,
            vec![
                Token::Ident("x".to_string()),
                Token::Assign,
                Token::LParen,
                Token::Ident("y".to_string()),
                Token::Plus,
                Token::Int(10),
                Token::RParen,
                Token::Star,
                Token::Int(2),
                Token::Slash,
                Token::Int(5),
                Token::Semicolon,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_control_flow_structure() {
        let input = "while (count < 5) { count = count + 1; }";
        let tokens = get_tokens(input);
        assert_eq!(
            tokens,
            vec![
                Token::While,
                Token::LParen,
                Token::Ident("count".to_string()),
                Token::Less,
                Token::Int(5),
                Token::RParen,
                Token::LBrace,
                Token::Ident("count".to_string()),
                Token::Assign,
                Token::Ident("count".to_string()),
                Token::Plus,
                Token::Int(1),
                Token::Semicolon,
                Token::RBrace,
                Token::EOF
            ]
        );
    }

    #[test]
    #[should_panic(expected = "Unexpected character: @")]
    fn test_invalid_char() {
        get_tokens("x = y @ 5;");
    }
}
