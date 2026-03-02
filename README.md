This project is a **Transpiler** written in Rust. It takes a String, converts it to a high-level Abstract Syntax Tree (AST) representing common programming constructs (like loops, conditionals, and assignments) and transforms them into a "flattened" C program.

By utilizing a state-machine approach and a balanced ternary-tree dispatcher, the project obscures the original program logic into a single `while` loop. Not practical for anything, but a fun hobby project. Additionally, I wanted to see what the assembly output would look like.

---

## 🛠 Features

* **AST Implementation**: Supports integer and string literals, binary operations, variable assignments, `if/else` logic, and `while` loops.
* **Automatic Type Inference**: Variables are automatically detected and typed (as `int` or `char*`) based on the expressions assigned to them.
* **Control Flow Flattening**: Converts nested logic into a linear state machine.
* **Balanced Dispatcher**: Instead of a massive `switch` statement, it generates a balanced ternary tree of expressions (e.g., `state < pivot ? ... : ...`) to determine the next state efficiently.

---

## 🏗 Architecture

The transformation process follows these steps:

1. **Flattening**: The `Flattener` traverses the AST in reverse, assigning unique IDs to "Basic Blocks."
2. **Type Collection**: During traversal, a `HashSet<Variable>` is populated, inferring whether a variable is an `Integer` or `String`.
3. **Ternary Tree Generation**: The sorted blocks are converted into a nested ternary string used as the state transition logic.
4. **C Codegen**: A C template is filled with the variable declarations, the initial entry state, and the dispatcher loop.

---

## 💻 Usage

### Defining an AST

You can define your program logic using the `Stmt` and `Expr` enums:

```rust
let program = vec![
    Stmt::Assignment {
        name: "counter".to_string(),
        value: Expr::Literal(0),
    },
    Stmt::While {
        condition: Expr::Binary {
            left: Box::new(Expr::Variable("counter".to_string())),
            op: BinaryOp::LessThan,
            right: Box::new(Expr::Literal(10)),
        },
        body: vec![
            Stmt::Assignment {
                name: "counter".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Variable("counter".to_string())),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Literal(1)),
                },
            },
        ],
    },
];

```

### Generating C Code

Pass the AST to the `generate_c_program` function:

```rust
let c_code = generate_c_program(program);
println!("{}", c_code);

```

---

## 📄 Output Example

The generated C code will look similar to this:

```c
#include <stdio.h>
#include <stdlib.h>

int main() {
    int counter = 0;
    int state = 3; // Entry ID

    while (state != 0) {
        state = (state<2?(counter=0,2):(state<3?(counter<10?1:0):(counter=counter+1,2)));
    }

    return 0;
}

```

---

## 🚀 Future Enhancements

* **Expression Obfuscation**: Adding MBA (Mixed Boolean-Arithmetic) to the `expr_to_c` generator.
* **Function Support**: Expanding the flattener to handle multiple function definitions and call stacks.
* **Optimization Pass**: Merging basic blocks that have no branching logic to reduce state transitions.