// Defines the "atoms" of the language

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;


// defines a standard way to make duplicates of an object (deep copy)
// tells rust complier to write code to implement the Clone trait
#[derive(Clone)]

// Represents any value in the Lisp Language
// in rust each variant of an enum can hold different kinds of data
pub enum Lval {
    Num(i64),
    Sym(String),
    Err(String),
    Fun(Builtin),
    Lambda(Lenv, Box<Lval>, Box<Lval>), // Env, Formals, Body
    Sexpr(Vec<Lval>),
    Qexpr(Vec<Lval>),
    T,
    NIL,
}

pub type Builtin = fn(Rc<RefCell<Lenv>>, Vec<Lval>) -> Lval;

// Represents the scope/environment
#[derive(Clone)]
pub struct Lenv {
    // rust sandwich
    pub par: Option<Rc<RefCell<Lenv>>>,     // Option states that it might not exist. RC states that it has multiple owners. RefCell states that I can change it, even if its shared. 
    pub data: HashMap<String, Lval>,
}

// In rust you define the data (struct) and behavior (Impl) separately.
// impl is a why to inherent implementation
impl Lenv {
    pub fn new() -> Self {
        Lenv {
            par: None,
            data: HashMap::new(),
        }
    }
    
    pub fn get(&self, k: &str) -> Option<Lval> {
        if let Some(val) = self.data.get(k) {
            Some(val.clone())
        } else if let Some(ref par) = self.par {
            par.borrow().get(k)
        } else {
            None
        }
    }

    pub fn put(&mut self, k: String, v: Lval) {
        self.data.insert(k, v);
    }

    pub fn def(&mut self, k: String, v: Lval) {
        if let Some(ref par) = self.par {
            par.borrow_mut().def(k, v);
        } else {
            self.put(k, v);
        }
    }
    
    pub fn copy(&self) -> Self {
        Lenv {
            par: self.par.clone(),
            data: self.data.clone(),
        }
    }
}

/// function for converting an Lval into a user-friendly String.
/// This is equivalent to overriding toString()
impl fmt::Display for Lval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lval::Num(n) => write!(f, "{}", n),
            Lval::Sym(s) => write!(f, "{}", s),
            Lval::Err(e) => write!(f, "Error: {}", e),
            Lval::Fun(_) => write!(f, "<function>"),
            Lval::Lambda(env, formals, body) => {
                write!(f, "(\\ {})", formals) // Simplified printing for lambda
            },
            Lval::Sexpr(cell) => {
                write!(f, "(")?;
                for (i, c) in cell.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", c)?;
                }
                write!(f, ")")
            },
            Lval::Qexpr(cell) => {
                write!(f, "'")?;
                for (i, c) in cell.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", c)?;
                }
                Ok(())
            },
            Lval::T => write!(f, "T"),
            Lval::NIL => write!(f, "NIL"),
        }
    }
}

impl fmt::Debug for Lval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// provides functionality for checking if two lval variables are equal
/// equivalent to overriding .equals()
impl PartialEq for Lval {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Lval::Num(a), Lval::Num(b)) => a == b,
            (Lval::Sym(a), Lval::Sym(b)) => a == b,
            (Lval::Err(a), Lval::Err(b)) => a == b,
            (Lval::Sexpr(a), Lval::Sexpr(b)) => a == b,
            (Lval::Qexpr(a), Lval::Qexpr(b)) => a == b,
            (Lval::T, Lval::T) => true,
            (Lval::NIL, Lval::NIL) => true,
            // Functions and Lambdas are hard to compare, usually false or pointer equality
            _ => false,
        }
    }
}
