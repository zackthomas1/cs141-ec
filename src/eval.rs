// Defines the evaluation logic of the program

use crate::types::{Lval, Lenv, Builtin};
use std::rc::Rc;
use std::cell::RefCell;


pub fn lval_eval(e: Rc<RefCell<Lenv>>, v: Lval) -> Lval {
    match v {
        Lval::Sym(s) => {
            if let Some(val) = e.borrow().get(&s) {
                val
            } else {
                Lval::Err(format!("Unbound symbol '{}'", s))
            }
        },
        Lval::Sexpr(mut cells) => {
            if cells.is_empty() { return Lval::Sexpr(cells); }

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

            let f = evaluated.remove(0);
            lval_call(e, f, evaluated)
        },
        _ => v,
    }
}

pub fn lval_call(e: Rc<RefCell<Lenv>>, f: Lval, args: Vec<Lval>) -> Lval {
    match f {
        Lval::Fun(func) => func(e, args),
        Lval::Lambda(env, formals, body) => {
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

fn builtin_var(e: Rc<RefCell<Lenv>>, args: Vec<Lval>, func: &str) -> Lval {
    if args.is_empty() { return Lval::Err("Too few args".to_string()); }
    
    let mut args_iter = args.into_iter();
    let syms = args_iter.next().unwrap();
    
    let syms_vec = match syms {
        Lval::Qexpr(v) => v,
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
    
    if let Lval::Qexpr(_) = formals { } else { return Lval::Err("Formals must be Qexpr".to_string()); }
    if let Lval::Qexpr(_) = body { } else { return Lval::Err("Body must be Qexpr".to_string()); }
    
    Lval::Lambda(Lenv::new(), Box::new(formals), Box::new(body))
}

pub fn builtin_head(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let a = args.into_iter().next().unwrap();
    match a {
        Lval::Qexpr(mut cells) => {
            if cells.is_empty() { return Lval::Err("Argument is empty".to_string()); }
            let first = cells.remove(0);
            Lval::Qexpr(vec![first])
        },
        _ => Lval::Err("Argument must be Qexpr".to_string()),
    }
}

pub fn builtin_tail(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let a = args.into_iter().next().unwrap();
    match a {
        Lval::Qexpr(mut cells) => {
            if cells.is_empty() { return Lval::Err("Argument is empty".to_string()); }
            cells.remove(0);
            Lval::Qexpr(cells)
        },
        _ => Lval::Err("Argument must be Qexpr".to_string()),
    }
}

pub fn builtin_list(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    Lval::Qexpr(args)
}

pub fn builtin_eval(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 1 { return Lval::Err("Expected 1 arg".to_string()); }
    let a = args.into_iter().next().unwrap();
    match a {
        Lval::Qexpr(cells) => {
            let x = Lval::Sexpr(cells);
            lval_eval(e, x)
        },
        _ => Lval::Err("Argument must be Qexpr".to_string()),
    }
}

pub fn builtin_join(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    let mut joined = Vec::new();
    for arg in args {
        match arg {
            Lval::Qexpr(cells) => joined.extend(cells),
            _ => return Lval::Err("Arguments must be Qexpr".to_string()),
        }
    }
    Lval::Qexpr(joined)
}

pub fn builtin_eq(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    let mut iter = args.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    if a == b { Lval::Num(1) } else { Lval::Num(0) }
}

pub fn builtin_equal(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    builtin_eq(e, args)
}

pub fn builtin_ne(_e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    if args.len() != 2 { return Lval::Err("Expected 2 args".to_string()); }
    let mut iter = args.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    if a != b { Lval::Num(1) } else { Lval::Num(0) }
}

pub fn builtin_cond(e: Rc<RefCell<Lenv>>, args: Vec<Lval>) -> Lval {
    for arg in args {
        let cells = match arg {
            Lval::Qexpr(c) => c,
            _ => return Lval::Err("Cond branches must be Qexpr".to_string()),
        };
        
        if cells.len() < 2 { return Lval::Err("Cond branch too short".to_string()); }
        
        let cond = cells[0].clone();
        let res = lval_eval(e.clone(), cond);
        
        if let Lval::Err(_) = res { return res; }
        
        let is_true = match res {
            Lval::Num(0) => false,
            Lval::Num(_) => true,
            _ => true,
        };
        
        if is_true {
            let body = cells[1].clone();
            return lval_eval(e.clone(), body);
        }
    }
    Lval::Sexpr(vec![])
}
