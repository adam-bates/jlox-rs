use crate::{scanner::Scanner, token::Token, token_type::TokenType};

use std::{fs, io, path, process};

pub fn run(mut args: Vec<String>) -> io::Result<()> {
    if args.len() > 1 {
        println!("Usage: jlox [script]");
        process::exit(64);
    } else if args.len() == 1 {
        run_file(args.remove(0))?;
    } else {
        run_prompt()?;
    }

    Ok(())
}

static mut HAD_ERROR: bool = false;

pub fn had_error() -> bool {
    return unsafe { HAD_ERROR };
}

fn run_file(path: String) -> io::Result<()> {
    let content = fs::read_to_string(path::PathBuf::from(path))?;

    run_source(content);

    if had_error() {
        // Indicate an error in the exit code
        process::exit(65);
    }

    Ok(())
}

fn run_prompt() -> io::Result<()> {
    loop {
        // Flushing normally only happens on new-line,
        // Have to force in order to print on same line as accepting input
        print!("> ");
        io::Write::flush(&mut io::stdout())?;

        let mut line = String::new();
        let bytes_read = io::stdin().read_line(&mut line)?;

        if bytes_read == 0 {
            break;
        }

        run_source(line);

        unsafe {
            HAD_ERROR = false;
        }
    }

    Ok(())
}

fn run_source(source: String) {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{token:?}");
    }
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn token_error(token: Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(token.line, " at end", message);
    } else {
        report(token.line, &format!("at '{}'", token.lexeme), message);
    }
}

fn report(line: usize, where_: &str, message: &str) {
    eprintln!("[line {line}] Error{where_}: {message}");

    unsafe {
        HAD_ERROR = true;
    }
}
