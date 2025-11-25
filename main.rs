use std::io;
use std::collections::HashMap; 

fn parse(statement: &str) -> Vec<i32>{ 

}

fn eval(statement: &str, alist: &HashMap<String, String>){
    println!("Entered: {}", statement);
}

fn eval_atom( alist: &HashMap<String, String>){

}

fn repl() -> {
    let mut running: bool = true; 
    let mut alist: HashMap<String, String> = HashMap::new()

    // Print version and exit information
    println!("Lispy Version 0.0.0.0.1");
    println!("Press Ctrl+c to Exit\n");

    while running{

        // output prompy
        print!("lispy>");

        // read line of user input
        let mut user_input = String::new(); 
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line"); 

        let statment: &str = user_input.trim();
        if statment == "exit"{
            break;
        }else{
            tokens = parse(statement); 
            eval(tokens, alist);
        }
    }
}

fn main(){
    repl();
}