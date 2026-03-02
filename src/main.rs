mod ast;
mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    // The input source language
    let program_source = "
        fname = \"hello.txt\";
        fp = fopen(fname, \"r\");
        if (fp) {
            status = fread(buf, 1, 10, fp);
        }
        fclose(fp);
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
