// Defines the evaluation logic of the program

use crate::types::{Lval, Lenv, Builtin};
use std::rc::Rc;
use std::cell::RefCell;


pub fn lval_eval(e: Rc<RefCell<Lenv>>, v: Lval) -> Lval {
    match v {
        Lval::Sym(s) => {
            if s == "T" || s == "t" { return Lval::T; }
            if s == "nil" { return Lval::NIL; }
            if let Some(val) = e.borrow().get(&s) {
                val
            } else {
                Lval::Err(format!("Unbound symbol '{}'", s))
            }
        },
        Lval::Sexpr(mut cells) => {
            if cells.is_empty() { return Lval::Sexpr(cells); }

            if let Lval::Sym(ref s) = cells[0] {
                if s == "quote" {
                    return builtin_quote(e, cells);
                }
                if s == "setq" {
                    return builtin_putq(e, cells);
                }
                if s == "defun" {
                    return builtin_defun(e, cells);
                }
                if s == "cond" {
                    return builtin_cond(e, cells[1..].to_vec());
                }
            }

            let mut evaluated = Vec::new();
            for cell in cells {
                evaluated.push(lval_eval(e.clone(), cell));
            }
            
            // check for error lvals
            for cell in &evaluated {
                if let Lval::Err(_) = cell {
                    return cell.clone();
                }
            }

            if evaluated.is_empty() { return Lval::Sexpr(evaluated); }

            if evaluated.len() == 1 {
                return evaluated.remove(0);
            }

            // once the sexpr is evaluate make function call to evaluate the statement
            let f = evaluated.remove(0);
            lval_call(e, f, evaluated)
        },
        _ => v,
    }
}

pub fn lval_call(e: Rc<RefCell<Lenv>>, f: Lval, args: Vec<Lval>) -> Lval {
    match f {
        Lval::Fun(func) => func(e, args),   // evaluate builtin functions
        Lval::Lambda(env, formals, body) => {       // evaluate custom user defined functions
            let mut f_env = env;
            let given = args.len();
            let total = if let Lval::Qexpr(ref cells) = *formals { cells.len() } else { 0 };
            
            let mut args_iter = args.into_iter();
            let mut formals_vec = if let Lval::Qexpr(cells) = *formals { cells } else { vec![] };
            
            while let Some(arg) = args_iter.next() {
                if formals_vec.is_empty() {
                    return Lval::Err(format!("Function passed too many arguments. Got {}, Expected {}.", given, total));
                }
                
                let sym = formals_vec.remove(0);
                if let Lval::Sym(s) = sym {
                    f_env.put(s, arg);
                } else {
                     return Lval::Err("Formal should be a symbol".to_string());
                }
            }
            
            if formals_vec.is_empty() {
                f_env.par = Some(e.clone());
                let env_rc = Rc::new(RefCell::new(f_env));
                let body_sexpr = Lval::Sexpr(if let Lval::Qexpr(cells) = *body { cells } else { vec![] });
                lval_eval(env_rc, body_sexpr)
            } else {
                Lval::Lambda(f_env, Box::new(Lval::Qexpr(formals_vec)), body)
            }
        },
        _ => Lval::Err("S-expression starts with incorrect type".to_string()),
    }
}

pub fn builtin_add(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval { builtin_op(e, args, "+") }
pub fn builtin_sub(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval { builtin_op(e, args, "-") }
pub fn builtin_mul(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval { builtin_op(e, args, "*") }
pub fn builtin_div(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval { builtin_op(e, args, "/") }

fn builtin_op(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>, op: &str) -> Lval {
    for arg in &args {
        if let Lval::Num(_) = arg { } else { return Lval::Err("Non-number".to_string()); }
    }
    
    let mut args_iter = args.into_iter();
    let mut x = match args_iter.next() {
        Some(Lval::Num(n)) => n,
        _ => return Lval::Err("No arguments".to_string()),
    };
    
    if op == "-" && args_iter.len() == 0 {
        return Lval::Num(-x);
    }

    while let Some(arg) = args_iter.next() {
        let y = match arg { Lval::Num(n) => n, _ => 0 };
        match op {
            "+" => x += y,
            "-" => x -= y,
            "*" => x *= y,
            "/" => {
                if y == 0 { return Lval::Err("Division by zero".to_string()); }
                x /= y;
            },
            _ => {},
        }
    }
    Lval::Num(x)
}

pub fn builtin_def(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval { builtin_var(e, args, "def") }
pub fn builtin_put(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval { builtin_var(e, args, "=") }
pub fn builtin_putq(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 3 { return Lval::Err("Function 'setq' passed incorrect number of arguments.".to_string()); }
    
    let sym = args[1].clone();
    let val_expr = args[2].clone();
    
    if let Lval::Sym(_) = sym {
        // ok
    } else {
        return Lval::Err("First argument to setq must be a symbol".to_string());
    }
    
    let val = lval_eval(e.clone(), val_expr);
    if let Lval::Err(_) = val { return val; }
    
    builtin_var(e, vec![Lval::Qexpr(vec![sym]), val], "=")
}

pub fn builtin_defun(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 4 { return Lval::Err("Function 'defun' passed incorrect number of arguments.".to_string()); }
    
    let sym = args[1].clone();
    let formals = args[2].clone();
    let body = args[3].clone();
    
    if let Lval::Sym(_) = sym {
        // ok
    } else {
        return Lval::Err("First argument to defun must be a symbol".to_string());
    }
    
    let formals_q = match formals {
        Lval::Sexpr(v) => Lval::Qexpr(v),
        Lval::Qexpr(v) => Lval::Qexpr(v),
        _ => return Lval::Err("Second argument to defun must be a list".to_string()),
    };

    let body_q = match body {
        Lval::Sexpr(v) => Lval::Qexpr(v),
        Lval::Qexpr(v) => Lval::Qexpr(v),
        _ => return Lval::Err("Third argument to defun must be a list".to_string()),
    };
    
    let lambda = Lval::Lambda(Lenv::new(), Box::new(formals_q), Box::new(body_q));
    
    builtin_var(e, vec![Lval::Qexpr(vec![sym]), lambda], "def")
}

fn builtin_var(e: Rc<RefCell<Lenv>>, args: Vec<Lval>, func: &str) -> Lval {
    if args.is_empty() { return Lval::Err("Too few args".to_string()); }
    
    let mut args_iter = args.into_iter();
    let syms = args_iter.next().unwrap();
    
    let syms_vec = match syms {
        Lval::Qexpr(mut v) => {
            if v.len() == 1 {
                match v.pop().unwrap() {
                    Lval::Sexpr(inner) => inner,
                    other => {
                        v.push(other);
                        v
                    }
                }
            } else {
                v
            }
        },
        _ => return Lval::Err("First arg must be Qexpr".to_string()),
    };
    
    for (i, sym) in syms_vec.iter().enumerate() {
        let val = match args_iter.next() {
            Some(v) => v,
            None => return Lval::Err("Too few values".to_string()),
        };
        
        if let Lval::Sym(s) = sym {
            if func == "def" {
                e.borrow_mut().def(s.clone(), val);
            } else {
                e.borrow_mut().put(s.clone(), val);
            }
        } else {
            return Lval::Err("Cannot define non-symbol".to_string());
        }
    }
    
    Lval::Sexpr(vec![])
}

pub fn builtin_lambda(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    let mut args_iter = args.into_iter();
    let formals = args_iter.next().unwrap();
    let body = args_iter.next().unwrap();
    
    if let Lval::Qexpr(_) = body { } else { return Lval::Err("Body must be Qexpr".to_string()); }
    
    let new_formals = match formals {
        Lval::Qexpr(mut cells) => {
            if cells.len() == 1 {
                match cells.pop().unwrap() {
                    Lval::Sexpr(inner_cells) => Lval::Qexpr(inner_cells),
                    other => {
                        cells.push(other);
                        Lval::Qexpr(cells)
                    }
                }
            } else {
                Lval::Qexpr(cells)
            }
        },
        _ => return Lval::Err("Formals must be Qexpr".to_string()),
    };
    
    Lval::Lambda(Lenv::new(), Box::new(new_formals), Box::new(body))
}

/// used for car operation
pub fn builtin_head(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let a = args.into_iter().next().unwrap();
    match a {
        Lval::Qexpr(mut cells) => {
            if cells.is_empty() { return Lval::Err("Argument is empty".to_string()); }
            
            if cells.len() != 1 { return Lval::Err("Expected single list in Qexpr".to_string()); }

            match &mut cells[0] {
                Lval::Sexpr(children) => {
                    if children.is_empty() { return Lval::Err("List is empty".to_string()); }
                    let first = children.remove(0);
                    first
                },
                _ => Lval::Err("Argument must be a list (Sexpr)".to_string()),
            }
        },
        Lval::Sexpr(mut cells) => {
            if cells.is_empty() { return Lval::Err("List is empty".to_string()); }
            cells.remove(0)
        },
        _ => Lval::Err("Argument must be Qexpr or Sexpr".to_string()),
    }
}

/// used for cdr operation
pub fn builtin_tail(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let a = args.into_iter().next().unwrap();
    match a {
        Lval::Qexpr(mut cells) => {
            if cells.is_empty() { return Lval::Err("Argument is empty".to_string()); }
            
            // The Qexpr should contain exactly one element, which is the list (Sexpr) we want to operate on
            if cells.len() != 1 { return Lval::Err("Expected single list in Qexpr".to_string()); }
            
            let mut child = cells.remove(0);
            match child {
                Lval::Sexpr(ref mut children) => {
                    if children.is_empty() { return Lval::Err("List is empty".to_string()); }
                    children.remove(0);
                },
                _ => return Lval::Err("Argument must be a list (Sexpr)".to_string()),
            }
            
            // Return the modified Sexpr directly
            child
        },
        Lval::Sexpr(mut cells) => {
            if cells.is_empty() { return Lval::Err("List is empty".to_string()); }
            cells.remove(0);
            Lval::Sexpr(cells)
        },
        _ => Lval::Err("Argument must be Qexpr or Sexpr".to_string()),
    }
}

pub fn builtin_eval(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let mut a = args.into_iter().next().unwrap();

    // println!("Eval input: {:?}", a);
    // Recursively unwrap single-element Qexprs to handle nested quoting from car/head
    while let Lval::Qexpr(ref cells) = a {
        if cells.len() == 1 {
            // Check if the single child is a Qexpr OR an Sexpr
            // If it's an Sexpr, we might want to evaluate it?
            // But read() converts Sexpr to Qexpr when quoting.
            
            // If we have {{+ 1 2}}, we want {+ 1 2}.
            if let Lval::Qexpr(_) = cells[0] {
                if let Lval::Qexpr(mut c) = a {
                    a = c.remove(0);
                    // println!("Unwrapped to: {:?}", a);
                    continue;
                }
            }
        }
        break;
    }

    match a {
        Lval::Qexpr(cells) => {
            let x = Lval::Sexpr(cells);
            lval_eval(e, x)
        },
        _ => Lval::Err("Argument must be Qexpr".to_string()),
    }
}


pub fn builtin_cons(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    
    let mut joined = Vec::new();
    
    for arg in args {
        match arg {
            Lval::Qexpr(cells) => {
                for cell in cells {
                    match cell {
                        Lval::Sexpr(children) => joined.extend(children),
                        _ => joined.push(cell),
                    }
                }
            },
            Lval::Sexpr(cells) => {
                joined.extend(cells);
            },
            Lval::NIL => {},
            _ => joined.push(arg),
        }
    }
    
    Lval::Sexpr(joined)
}

pub fn builtin_eq(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    let mut iter = args.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    
    match (a, b) {
        (Lval::Num(v1), Lval::Num(v2)) => if v1 == v2 { Lval::T } else { Lval::NIL },
        (Lval::Sym(v1), Lval::Sym(v2)) => if v1 == v2 { Lval::T } else { Lval::NIL },
        (Lval::T, Lval::T) => Lval::T,
        (Lval::NIL, Lval::NIL) => Lval::T,
        // For composite objects (Sexpr, Qexpr) and others, eq checks identity.
        // Since values are cloned from environment, they are distinct objects.
        _ => Lval::NIL,
    }
}

pub fn builtin_equal(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    let mut iter = args.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    if a == b { Lval::T } else { Lval::NIL }
}

pub fn builtin_ne(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    let mut iter = args.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    if a != b { Lval::T } else { Lval::NIL }
}

pub fn builtin_null(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let a = args.into_iter().next().unwrap();
    match a {
        Lval::NIL => Lval::T,
        Lval::Sexpr(cells) if cells.is_empty() => Lval::T,
        Lval::Qexpr(cells) if cells.is_empty() => Lval::T,
        _ => Lval::NIL,
    }
}

pub fn builtin_cond(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    for arg in args {
        let mut cells = match arg {
            Lval::Qexpr(c) => c,
            Lval::Sexpr(c) => c,
            _ => return Lval::Err("Cond branches must be Qexpr or Sexpr".to_string()),
        };
        
        if cells.len() == 1 {
            if let Lval::Sexpr(inner) = &cells[0] {
                cells = inner.clone();
            }
        }
        
        if cells.len() < 2 { return Lval::Err("Cond branch too short".to_string()); }
        
        // takes first item of the branch and evaluates it
        let cond = cells[0].clone();
        let res = lval_eval(e.clone(), cond);
        
        if let Lval::Err(_) = res { return res; }
        
        // checks truth of evaluated expression
        let is_true = match res {
            Lval::NIL => false,
            Lval::Num(0) => false,
            _ => true,
        };
        
        // if the condition is true then execute body
        if is_true {
            let body = cells[1].clone();
            return lval_eval(e.clone(), body);
        }
    }
    Lval::Sexpr(vec![])
}

pub fn builtin_quote(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Function 'quote' passed incorrect number of arguments.".to_string()); }
    return Lval::Qexpr(vec![args[1].clone()])
}

pub fn builtin_print(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{}", arg);
    }
    Lval::Void
}