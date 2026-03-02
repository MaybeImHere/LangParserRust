use crate::ast::{BinaryOp, Expr, Stmt};
use crate::lexer::Token;
use std::collections::HashSet;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub vars: HashSet<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            vars: HashSet::new(),
        }
    }

    // --- Entry Point ---
    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    // --- Statement Parsing ---
    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::LBrace => Stmt::Block(self.parse_block()),
            Token::Ident(_) => {
                // Peek ahead to see if this is an assignment or a standalone call
                // Since our Lexer is greedy, we can look at pos + 1
                if let Some(Token::Assign) = self.tokens.get(self.pos + 1) {
                    self.parse_assignment()
                } else {
                    self.parse_call_stmt()
                }
            }
            _ => panic!("Unexpected token in statement: {:?}", self.peek()),
        }
    }

    fn parse_call_stmt(&mut self) -> Stmt {
        let expr = self.parse_primary(); // This will trigger parse_function_call
        self.consume(Token::Semicolon, "Expected ';' after function call");
        Stmt::Call(expr)
    }

    fn parse_assignment(&mut self) -> Stmt {
        let name = if let Token::Ident(n) = self.advance() {
            n
        } else {
            panic!("Expected variable name at {:?}", self.peek());
        };

        self.vars.insert(name.clone());
        self.consume(Token::Assign, "Expected '='");
        let value = self.parse_expr(0);
        self.consume(Token::Semicolon, "Expected ';' after assignment");

        Stmt::Assignment { name, value }
    }

    fn parse_if(&mut self) -> Stmt {
        self.advance(); // consume 'if'
        self.consume(Token::LParen, "Expected '('");
        let condition = self.parse_expr(0);
        self.consume(Token::RParen, "Expected ')'");

        let then_branch = self.parse_maybe_block();
        let mut else_branch = None;

        if self.peek() == Token::Else {
            self.advance();
            else_branch = Some(self.parse_maybe_block());
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn parse_while(&mut self) -> Stmt {
        self.advance(); // consume 'while'
        self.consume(Token::LParen, "Expected '('");
        let condition = self.parse_expr(0);
        self.consume(Token::RParen, "Expected ')'");

        let body = self.parse_maybe_block();
        Stmt::While { condition, body }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.consume(Token::LBrace, "Expected '{'");
        let mut stmts = Vec::new();
        while self.peek() != Token::RBrace && !self.is_at_end() {
            stmts.push(self.parse_stmt());
        }
        self.consume(Token::RBrace, "Expected '}'");
        stmts
    }

    // Helper to handle both { stmt; } and single stmt;
    fn parse_maybe_block(&mut self) -> Vec<Stmt> {
        if self.peek() == Token::LBrace {
            self.parse_block()
        } else {
            vec![self.parse_stmt()]
        }
    }

    // --- Expression Parsing (Precedence Climbing) ---
    fn parse_expr(&mut self, min_prec: i8) -> Expr {
        let mut left = self.parse_primary();

        while let Some(op_info) = self.get_op_info(&self.peek()) {
            let (prec, op) = op_info;
            if prec < min_prec {
                break;
            }

            self.advance(); // consume operator
            let right = self.parse_expr(prec + 1);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Token::Int(val) => Expr::Literal(val),
            Token::Ident(name) => {
                // If the next token is '(', it's a function call!
                if self.peek() == Token::LParen {
                    self.parse_function_call(name)
                } else {
                    // It's just a standard variable
                    self.vars.insert(name.clone());
                    Expr::Variable(name)
                }
            }
            Token::LParen => {
                let expr = self.parse_expr(0);
                self.consume(Token::RParen, "Expected ')'");
                expr
            }
            Token::Str(s) => Expr::StrLiteral(s),
            t => panic!("Unexpected token in expression: {:?}", t),
        }
    }

    fn parse_function_call(&mut self, name: String) -> Expr {
        self.consume(Token::LParen, "Expected '(' in function call");

        let mut args = Vec::new();
        if self.peek() != Token::RParen {
            loop {
                args.push(self.parse_expr(0));
                if self.peek() == Token::Comma {
                    self.advance(); // consume ','
                } else {
                    break;
                }
            }
        }

        self.consume(Token::RParen, "Expected ')' after arguments");
        Expr::Call { name, args }
    }

    fn get_op_info(&self, tok: &Token) -> Option<(i8, BinaryOp)> {
        match tok {
            Token::EqualEqual => Some((1, BinaryOp::Equal)),
            Token::Less => Some((2, BinaryOp::LessThan)),
            Token::Greater => Some((2, BinaryOp::GreaterThan)),
            Token::Plus => Some((3, BinaryOp::Add)),
            Token::Minus => Some((3, BinaryOp::Sub)),
            Token::Star => Some((4, BinaryOp::Mul)),
            Token::Slash => Some((4, BinaryOp::Div)),
            _ => None,
        }
    }

    // --- Navigation Helpers ---
    fn advance(&mut self) -> Token {
        let tok = self.peek();
        if tok != Token::EOF {
            self.pos += 1;
        }
        tok
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    fn consume(&mut self, expected: Token, msg: &str) {
        if self.peek() == expected {
            self.advance();
        } else {
            panic!("{} - Found {:?}", msg, self.peek());
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Token::EOF
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> (Vec<Stmt>, HashSet<String>) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        (parser.parse_program(), parser.vars)
    }

    #[test]
    fn test_operator_precedence() {
        // x = 1 + 2 * 3;
        // Should be: x = (1 + (2 * 3))
        let (stmts, _) = parse("x = 1 + 2 * 3;");
        if let Stmt::Assignment { value, .. } = &stmts[0] {
            if let Expr::Binary { op, right, .. } = value {
                assert_eq!(format!("{:?}", op), "Add");
                // Right side of Add should be the Mul binary expr
                if let Expr::Binary { op: r_op, .. } = &**right {
                    assert_eq!(format!("{:?}", r_op), "Mul");
                } else {
                    panic!("Right side of Add should be a Binary expression (Mul)");
                }
            }
        }
    }

    #[test]
    fn test_variable_discovery() {
        let (_, vars) = parse("x = y + z; if (condition) { a = 1; }");
        assert!(vars.contains("x"));
        assert!(vars.contains("y"));
        assert!(vars.contains("z"));
        assert!(vars.contains("condition"));
        assert!(vars.contains("a"));
        assert_eq!(vars.len(), 5);
    }

    #[test]
    fn test_nested_while_if() {
        let input = "
            while (x < 10) {
                if (x == 5) {
                    found = 1;
                }
                x = x + 1;
            }
        ";
        let (stmts, _) = parse(input);
        assert_eq!(stmts.len(), 1); // Top level is just the while loop
        if let Stmt::While { body, .. } = &stmts[0] {
            assert_eq!(body.len(), 2); // if_stmt and x = x + 1
            match &body[0] {
                Stmt::If { .. } => {}
                _ => panic!("First statement in while should be an If"),
            }
        }
    }

    #[test]
    fn test_else_branch() {
        let input = "if (x) { y = 1; } else { y = 2; }";
        let (stmts, _) = parse(input);
        if let Stmt::If { else_branch, .. } = &stmts[0] {
            assert!(else_branch.is_some());
            let branch = else_branch.as_ref().unwrap();
            assert_eq!(branch.len(), 1);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parentheses() {
        // x = (1 + 2) * 3;
        // Should force Add to be the child of Mul
        let (stmts, _) = parse("x = (1 + 2) * 3;");
        if let Stmt::Assignment { value, .. } = &stmts[0] {
            if let Expr::Binary { op, left, .. } = value {
                assert_eq!(format!("{:?}", op), "Mul");
                if let Expr::Binary { op: l_op, .. } = &**left {
                    assert_eq!(format!("{:?}", l_op), "Add");
                } else {
                    panic!("Left side of Mul should be the parenthesized Add");
                }
            }
        }
    }
}
