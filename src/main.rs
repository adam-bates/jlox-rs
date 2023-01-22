mod lox;
mod scanner;
mod token;
mod token_type;

use std::{env, io};

fn main() -> io::Result<()> {
    let mut args = env::args().collect::<Vec<String>>();
    args.pop();

    return lox::run(args);
}
