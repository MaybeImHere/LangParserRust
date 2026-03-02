mod ast;
mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    // The input source language
    let program_source = "
a0_num = 1; a0_den = 1;
a1_num = 1; a1_den = 2;
a2_num = 1; a2_den = 3;
a3_num = 1; a3_den = 4;
a4_num = 1; a4_den = 5;

a0_num = 1 * (a0_num * a1_den - a1_num * a0_den);
a0_den = a0_den * a1_den;

a1_num = 2 * (a1_num * a2_den - a2_num * a1_den);
a1_den = a1_den * a2_den;

a2_num = 3 * (a2_num * a3_den - a3_num * a2_den);
a2_den = a2_den * a3_den;

a3_num = 4 * (a3_num * a4_den - a4_num * a3_den);
a3_den = a3_den * a4_den;

a0_num = 1 * (a0_num * a1_den - a1_num * a0_den);
a0_den = a0_den * a1_den;

a1_num = 2 * (a1_num * a2_den - a2_num * a1_den);
a1_den = a1_den * a2_den;

a2_num = 3 * (a2_num * a3_den - a3_num * a2_den);
a2_den = a2_den * a3_den;

a0_num = 1 * (a0_num * a1_den - a1_num * a0_den);
a0_den = a0_den * a1_den;

a1_num = 2 * (a1_num * a2_den - a2_num * a1_den);
a1_den = a1_den * a2_den;

a0_num = 1 * (a0_num * a1_den - a1_num * a0_den);
a0_den = a0_den * a1_den;
    ";

    // Step 1: Lexical Analysis
    let mut lexer = Lexer::new(program_source);
    let tokens = lexer.tokenize();

    // Step 2: Parsing into AST
    let mut parser = Parser::new(tokens);
    let ast_nodes = parser.parse_program();

    // Step 3: Convert AST to Cursed C
    // We pass parser.vars so we know which variables to declare in C
    let c_program = ast::generate_c_program(ast_nodes);

    // Step 4: Output the result
    println!("{}", c_program);
}
