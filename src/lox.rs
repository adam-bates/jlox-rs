use crate::{
    interpreter::Interpreter, parser::Parser, runtime_value::RuntimeError, scanner::Scanner,
    token::Token, token_type::TokenType,
};

use std::{fs, io, path, process};

pub fn run_lox(mut args: Vec<String>) -> io::Result<()> {
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
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn had_error() -> bool {
    return unsafe { HAD_ERROR };
}

pub fn had_runtime_error() -> bool {
    return unsafe { HAD_RUNTIME_ERROR };
}

fn run_file(path: String) -> io::Result<()> {
    let mut interpreter = Interpreter;

    let content = fs::read_to_string(path::PathBuf::from(path))?;

    run(&mut interpreter, content);

    // Indicate an error in the exit code
    if had_error() {
        process::exit(65);
    }

    if had_runtime_error() {
        process::exit(70);
    }

    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let mut interpreter = Interpreter;

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

        run(&mut interpreter, line);

        unsafe {
            HAD_ERROR = false;
        }
    }

    Ok(())
}

fn run(interpreter: &mut Interpreter, source: String) {
    let scanner = Scanner::new(source.into());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let expr = parser.parse();

    // Stop if there was a syntax error
    if had_error() {
        return;
    }

    if let Some(mut expr) = expr {
        interpreter.interpret(&mut expr);
    }
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn runtime_error(error: RuntimeError) {
    eprintln!("{}", error);

    unsafe { HAD_RUNTIME_ERROR = true };
}

pub fn token_error(token: Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(token.line, " at end", message);
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

fn report(line: usize, where_: &str, message: &str) {
    eprintln!("[line {line}] Error{where_}: {message}");

    unsafe {
        HAD_ERROR = true;
    }
}
