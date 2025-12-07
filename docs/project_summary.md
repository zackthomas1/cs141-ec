# Rust Lisp Interpreter Project Summary

## Overview
This project is a functional Lisp interpreter implemented in Rust. It is a direct port and modernization of a C-based Lisp interpreter (`c-lisp`). The project aims to leverage Rust's safety and expression capabilities while maintaining the core logic of the original implementation.

## Architecture

The application is structured into modular components separating parsing, data representation, and evaluation logic.

### 1. Entry Point & REPL (`src/main.rs`)
- **REPL**: Implements a Read-Eval-Print Loop using the `rustyline` crate for robust command-line interaction and history management.
- **Parser Integration**: Utilizes the `pest` parser generator to transform raw text input into an Abstract Syntax Tree (AST) based on the grammar defined in `grammar.pest`.
- **Execution Loop**: 
    1. Reads input.
    2. Parses input into a Pest parse tree.
    3. Transforms the parse tree into internal `Lval` structures.
    4. Evaluates the `Lval` against the global environment.
    5. Prints the result.

### 2. Data Types (`src/types.rs`)
Defines the core data structures that represent Lisp values and the execution environment.

- **`Lval` Enum**: Represents all possible Lisp values:
    - `Num`: 64-bit Integers.
    - `Sym`: Symbols (variable names, operators).
    - `Sexpr`: Symbolic Expressions (lists of code to be evaluated).
    - `Qexpr`: Quoted Expressions (lists of data).
    - `Fun`: Built-in Rust function pointers.
    - `Lambda`: User-defined functions (closures), storing parameters, body, and a captured environment.
    - `Err`: Error messages.

- **`Lenv` Struct**: Represents the environment (scope).
    - Stores a mapping of symbol strings to `Lval`s.
    - Maintains a reference to a parent environment (`par`) to support lexical scoping.
    - Uses `Rc<RefCell<Lenv>>` to allow shared mutable access to environments, essential for recursive functions and global state.

### 3. Evaluation Logic (`src/eval.rs`)
Contains the core interpreter logic and standard library.

- **`lval_eval`**: The main evaluation function.
    - Resolves symbols from the environment.
    - Evaluates S-Expressions by recursively evaluating children and applying the first element as a function.
- **`lval_call`**: Handles function application.
    - Invokes built-in functions directly.
    - Handles Lambda application by binding arguments to a new local environment and evaluating the function body. Supports partial application (currying).
- **Built-ins**:
    - **Math**: `+`, `-`, `*`, `/`.
    - **List Operations**: `head` (`car`), `tail` (`cdr`), `list`, `join` (`cons`), `eval`.
    - **Variables**: `def` (global definition), `=` / `set` / `setq` (local definition).
    - **Logic**: `eq`, `neq`, `cond`.
    - **Functions**: `\` / `defun` (lambda creation).

### 4. Grammar (`src/grammar.pest`)
Defines the syntax of the language using PEG (Parsing Expression Grammar).
- Handles whitespace and comments.
- Defines rules for Numbers, Symbols, S-Expressions `(...)`, and Q-Expressions `'{...}`.

## Key Features
- **Interactive Shell**: Full REPL experience with history.
- **Symbolic Processing**: First-class support for manipulating code as data (homoiconicity).
- **Error Handling**: Propagates errors through the evaluation chain.
- **Functional Programming**: Supports higher-order functions, lambdas, and partial application.

## Usage
To start the interpreter:
```bash
cd rustlisp
cargo run
```

## Comparison with C Implementation
- **Memory Management**: Replaced manual `malloc`/`free` with Rust's ownership model and `Rc` smart pointers, eliminating memory leaks.
- **Parsing**: Replaced the custom `mpc` parser combinator library with the robust `pest` crate.
- **Safety**: Leverages Rust's type system to prevent common C errors like null pointer dereferences and buffer overflows.
