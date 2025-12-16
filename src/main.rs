// Application entry point. Handles Read-Eval-Print Loop (REPL)
// initializes the enrionment, and parses input using Pest parser


extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::rc::Rc;
use std::cell::RefCell;

mod types;
mod eval;

use types::{Lval, Lenv};
use eval::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LispyParser;

fn read(pair: pest::iterators::Pair<Rule>) -> Lval {
    match pair.as_rule() {
        Rule::number => {
            let n = pair.as_str().parse::<i64>().unwrap();
            Lval::Num(n)
        },
        Rule::symbol => {
            Lval::Sym(pair.as_str().to_string())
        },
        Rule::sexpr => {
            let mut cells = Vec::new();
            for inner_pair in pair.into_inner() {
                cells.push(read(inner_pair));
            }
            Lval::Sexpr(cells)
        },
        Rule::qexpr => {
            let inner_pair = pair.into_inner().next().unwrap();
            let val = read(inner_pair);
            Lval::Qexpr(vec![val])
        },
        Rule::expr => {
            read(pair.into_inner().next().unwrap())
        },
        Rule::lispy => {
            Lval::Err("Should not call read on lispy rule directly".to_string())
        },
        _ => Lval::Err("Unknown rule".to_string()),
    }
}

fn main() {
    // initialize rustyline
    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    // Create global environment to store variables and functions
    let env = Rc::new(RefCell::new(Lenv::new()));
    add_builtins(env.clone());

    println!("Lispy Version 0.1.0");
    println!("Press Ctrl+c to exit\n");

    // REPL
    loop {

        // Read line of input
        let readline = rl.readline("lispy> ");
        let line = match readline {
            Ok(l) => {
                let _ = rl.add_history_entry(l.as_str());
                l
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(ReadlineError::Io(ref e)) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Fallback for debuggers/environments where rustyline cannot access the console
                use std::io::{self, Write};
                print!("lispy> ");
                let _ = io::stdout().flush();
                let mut buffer = String::new();
                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => buffer.trim().to_string(),
                    Err(_) => break,
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        // Prase line of input
        let parse_result = LispyParser::parse(Rule::lispy, &line);
        match parse_result {
            Ok(mut pairs) => {
                let lispy_pair = pairs.next().unwrap(); // lispy rule
                for pair in lispy_pair.into_inner() {
                    if pair.as_rule() == Rule::EOI { continue; }
                    
                    // convert parse tree to Lval using read
                    let lval = read(pair);

                    // evaluate Lval 
                    let result = lval_eval(env.clone(), lval);
    
                    // print line output
                    println!("{}", result);
                }
            },
            Err(e) => println!("Error: {}", e),
        }
    }
    rl.save_history("history.txt").unwrap();
}


// Registers bult-in functions
fn add_builtins(e: Rc<RefCell<Lenv>>) {
    e.borrow_mut().put("eval".to_string(), Lval::Fun(builtin_eval));
    e.borrow_mut().put("join".to_string(), Lval::Fun(builtin_join));
    
    e.borrow_mut().put("+".to_string(), Lval::Fun(builtin_add));
    e.borrow_mut().put("-".to_string(), Lval::Fun(builtin_sub));
    e.borrow_mut().put("*".to_string(), Lval::Fun(builtin_mul));
    e.borrow_mut().put("/".to_string(), Lval::Fun(builtin_div));
    
    e.borrow_mut().put("def".to_string(), Lval::Fun(builtin_def));
    e.borrow_mut().put("=".to_string(), Lval::Fun(builtin_put));
    e.borrow_mut().put("set".to_string(), Lval::Fun(builtin_put));
    e.borrow_mut().put("setq".to_string(), Lval::Fun(builtin_put));
    e.borrow_mut().put("\\".to_string(), Lval::Fun(builtin_lambda));
    
    e.borrow_mut().put("car".to_string(), Lval::Fun(builtin_head));
    e.borrow_mut().put("cdr".to_string(), Lval::Fun(builtin_tail));
    e.borrow_mut().put("cons".to_string(), Lval::Fun(builtin_join));
    e.borrow_mut().put("defun".to_string(), Lval::Fun(builtin_lambda));
    e.borrow_mut().put("eq".to_string(), Lval::Fun(builtin_eq));
    e.borrow_mut().put("equal".to_string(), Lval::Fun(builtin_equal));
    e.borrow_mut().put("neq".to_string(), Lval::Fun(builtin_ne));
    e.borrow_mut().put("cond".to_string(), Lval::Fun(builtin_cond));
}
