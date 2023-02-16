mod ast;
mod environment;
mod interpreter;
mod lox;
mod lox_callable;
mod lox_function;
mod parser;
mod runtime_value;
mod scanner;
mod string;
mod token;
mod token_type;

use std::{env, io};

fn main() -> io::Result<()> {
    let mut args = env::args().collect::<Vec<String>>();
    args.remove(0);

    return lox::run_lox(args);
}
