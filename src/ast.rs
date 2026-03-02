use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    Equal,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(i32),
    Variable(String),
    StrLiteral(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

impl Expr {
    fn get_type(&self) -> VariableType {
        match self {
            Expr::Literal(_) => VariableType::Integer,
            Expr::StrLiteral(_) => VariableType::Pointer,
            Expr::Variable(_) => VariableType::Integer, // Defaulting to Int for simplicity
            Expr::Binary { .. } => VariableType::Integer,
            Expr::Call { .. } => VariableType::Pointer,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Call(Expr),
}

#[derive(Debug, Clone)]
struct BasicBlock {
    pub id: usize,
    /// The C-code side effect (e.g., "vars[0] = 5")
    pub effect: Option<String>,
    /// The logic to determine the next state ID
    pub next_state: NextState,
}

#[derive(Debug, Clone)]
enum NextState {
    /// Always go to this ID
    Static(usize),
    /// A C-style ternary: (cond) ? id_true : id_false
    Conditional {
        cond: String,
        true_id: usize,
        false_id: usize,
    },
    Exit,
}

struct Flattener {
    next_id: usize,
    blocks: Vec<BasicBlock>,
}

#[derive(PartialEq, Eq, Hash)]
enum VariableType {
    Integer,
    Pointer,
}

#[derive(PartialEq, Eq, Hash)]
struct Variable {
    name: String,
    var_type: VariableType,
}

impl Flattener {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            blocks: Vec::new(),
        }
    }

    fn gen_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn push_sorted(&mut self, new_block: BasicBlock) {
        // 1. Find the correct index using binary search
        let index = self
            .blocks
            .binary_search_by_key(&new_block.id, |b| b.id)
            .unwrap_or_else(|insert_at| insert_at);

        // 2. Insert the element at that position
        self.blocks.insert(index, new_block);
    }

    /// Flattens a sequence of statements.
    /// `exit_id` is where this sequence should jump when finished.
    /// returns the first state that will be executed, along with all variable names within the program.
    pub fn flatten_stmts(
        &mut self,
        stmts: &[Stmt],
        exit_id: usize,
        vars: &mut HashSet<Variable>,
    ) -> usize {
        let mut current_exit = exit_id;

        // Process in reverse to maintain correct jump IDs
        for stmt in stmts.iter().rev() {
            current_exit = self.flatten_stmt(stmt, current_exit, vars);
        }
        current_exit
    }

    fn flatten_stmt(&mut self, stmt: &Stmt, next_id: usize, vars: &mut HashSet<Variable>) -> usize {
        match stmt {
            Stmt::Assignment { name, value } => {
                // Record the variable name
                vars.insert(Variable {
                    name: name.clone(),
                    var_type: value.get_type(),
                });

                let id = self.gen_id();
                self.push_sorted(BasicBlock {
                    id,
                    effect: Some(format!("{}={}", name, self.expr_to_c(value))),
                    next_state: NextState::Static(next_id),
                });
                id
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let id = self.gen_id();
                // Recurse into branches and collect variables
                let then_id = self.flatten_stmts(then_branch, next_id, vars);
                let else_id = if let Some(eb) = else_branch {
                    self.flatten_stmts(eb, next_id, vars)
                } else {
                    next_id
                };

                self.push_sorted(BasicBlock {
                    id,
                    effect: None,
                    next_state: NextState::Conditional {
                        cond: self.expr_to_c(condition),
                        true_id: then_id,
                        false_id: else_id,
                    },
                });
                id
            }
            Stmt::While { condition, body } => {
                let head_id = self.gen_id();
                // Recurse into loop body and collect variables
                let body_id = self.flatten_stmts(body, head_id, vars);

                self.push_sorted(BasicBlock {
                    id: head_id,
                    effect: None,
                    next_state: NextState::Conditional {
                        cond: self.expr_to_c(condition),
                        true_id: body_id,
                        false_id: next_id,
                    },
                });
                head_id
            }
            Stmt::Block(stmts) => self.flatten_stmts(stmts, next_id, vars),
            Stmt::Call(expr) => {
                let id = self.gen_id();
                self.push_sorted(BasicBlock {
                    id,
                    effect: Some(self.expr_to_c(expr)),
                    next_state: NextState::Static(next_id),
                });
                id
            }
        }
    }

    fn expr_to_c(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(n) => n.to_string(),
            Expr::Variable(v) => v.clone(),
            Expr::Binary { left, op, right } => {
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::LessThan => "<",
                    BinaryOp::GreaterThan => ">",
                    BinaryOp::Equal => "==",
                };
                // ADD PARENTHESES HERE
                format!(
                    "(({}){}({}))",
                    self.expr_to_c(left),
                    op_str,
                    self.expr_to_c(right)
                )
            }
            Expr::Call { name, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.expr_to_c(a)).collect();
                format!("{}({})", name, arg_strs.join(","))
            }
            Expr::StrLiteral(s) => format!("\"{}\"", s), // Ensure strings are quoted
        }
    }
}

fn generate_ternary_tree_main(blocks: &[BasicBlock]) -> String {
    if blocks.is_empty() {
        return "state".to_string(); // Fallback/No-op
    }

    if blocks.len() == 1 {
        let block = &blocks[0];

        // The "Action" part: (side_effect, next_state_logic)
        let action = match &block.next_state {
            NextState::Static(id) => {
                if let Some(effect) = &block.effect {
                    format!("(({}),{})", effect, id)
                } else {
                    format!("({})", id)
                }
            }
            NextState::Conditional {
                cond,
                true_id,
                false_id,
            } => {
                // If there's an effect, include it,
                // then evaluate the ternary for the next state
                if let Some(effect) = &block.effect {
                    format!("({},({})?{}:{})", effect, cond, true_id, false_id)
                } else {
                    format!("({}?{}:{})", cond, true_id, false_id)
                }
            }
            NextState::Exit => {
                if let Some(effect) = &block.effect {
                    format!("({},0)", effect)
                } else {
                    format!("0")
                }
            }
        };

        // Base case: check if current state matches this block's ID
        //format!("(state == {} ? {} : state)", block.id, action)

        format!("{}", action)
    } else {
        // Split the slice to build a balanced tree
        let mid = blocks.len() / 2;
        let left_slice = &blocks[..mid];
        let right_slice = &blocks[mid..];

        // Use the ID of the middle element as the pivot
        let pivot = blocks[mid].id;

        format!(
            "(state<{}?{}:{})",
            pivot,
            generate_ternary_tree_main(left_slice),
            generate_ternary_tree_main(right_slice)
        )
    }
}

pub fn generate_c_program(ast: Vec<Stmt>) -> String {
    let mut flattener = Flattener::new();
    let mut variable_list: HashSet<Variable> = HashSet::new();
    let entry_id = flattener.flatten_stmts(&ast, 0, &mut variable_list);
    let ternary_tree = generate_ternary_tree_main(&flattener.blocks);

    let mut ret = String::from("#include <stdio.h>\n#include <stdlib.h>\n\nint main() {\n");

    // Declare variables with appropriate C types
    for var in &variable_list {
        match var.var_type {
            VariableType::Integer => {
                ret.push_str(&format!("    int {} = 0;\n", var.name));
            }
            VariableType::Pointer => {
                ret.push_str(&format!("    void* {} = NULL;\n", var.name));
            }
        }
    }

    ret.push_str(&format!("\n    int state = {};\n", entry_id));
    ret.push_str("    while (state != 0) {\n");
    ret.push_str(&format!("        state = {};\n", ternary_tree));
    ret.push_str("    }\n\n    return 0;\n}\n");

    ret
}
