use crate::{
    compiler::{Compiler, SymbolTable},
    lexer::Lexer,
    object::{AllObjects, Object},
    parser::{Parser, TRACING_ENABLED},
    vm,
};
use clap::Parser as ClapParser;
use std::io::{self, BufRead, Write};

const PROMPT: &str = ">> ";

/// The monkey programming language REPL (Read -> Evaluate -> Print -> Loop)
#[derive(ClapParser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enables tracing for parsing expressions
    #[clap(short, long, value_parser, default_value_t = false)]
    tracing: bool,
}

pub fn start_repl<T: BufRead, U: Write>(input: &mut T, output: &mut U) -> io::Result<()> {
    let args = Args::parse();
    unsafe {
        TRACING_ENABLED = args.tracing;
    }
    greet(output)?;

    let mut text = String::new();

    let mut constants = Vec::new();
    let mut globals = Vec::new();
    let mut symbol_table = SymbolTable::new();

    loop {
        write!(output, "{}", PROMPT)?;
        output.flush()?;

        input.read_line(&mut text)?;

        let trimmed = text.trim();
        if trimmed == r"\q" {
            writeln!(output, "bye")?;
            break;
        }

        if !trimmed.is_empty() {
            let result = execute_line_for_repl(&text, output, constants, globals, symbol_table)?;
            (constants, globals, symbol_table) = result.unwrap();
        }

        text.clear();
    }

    Ok(())
}

fn greet<U: Write>(output: &mut U) -> io::Result<()> {
    writeln!(
        output,
        "Hello {}!, This is the Monkey programming language!",
        whoami::username()
    )?;
    writeln!(output, "Feel free to type in commands")?;
    Ok(())
}

fn write_parser_errors<U: Write>(errors: &[String], output: &mut U) -> io::Result<()> {
    writeln!(output, "{}", MONKEY_FACE)?;
    writeln!(output, "Woops! We ran into some monkey business here 🥴")?;
    writeln!(output, "parser Errors:")?;
    for e in errors {
        writeln!(output, "\t- {}", e)?;
    }
    Ok(())
}

const MONKEY_FACE: &str = r#"
            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'
"#;

pub fn execute_program<U: Write>(text: &str, output: &mut U) -> io::Result<()> {
    let l = Lexer::new(text);
    let mut p = Parser::new(l);
    let program = p.parse_program();

    if !p.errors.is_empty() {
        write_parser_errors(&p.errors, output)?;
        return Ok(());
    }

    let mut comp = Compiler::new();
    if let Err(e) = comp.compile(program.make_node()) {
        write!(output, "Woops! Compilation failed:\n {}\n", e)?;
        return Ok(());
    }

    let mut machine = vm::VM::new(comp.byte_code());
    if let Err(e) = machine.run() {
        write!(output, "Woops! Executing bytecode failed:\n {}\n", e)?;
        return Ok(());
    }

    let Some(stack_top) = machine.result() else {
        writeln!(output, "Woops! Stack top is empty")?;
        return Ok(());
    };

    writeln!(output, "{}", stack_top.inspect())?;

    Ok(())
}

pub fn execute_line_for_repl<U: Write>(
    text: &str,
    output: &mut U,
    constants: Vec<AllObjects>,
    globals: Vec<AllObjects>,
    symbol_table: SymbolTable,
) -> io::Result<Option<(Vec<AllObjects>, Vec<AllObjects>, SymbolTable)>> {
    let l = Lexer::new(text);
    let mut p = Parser::new(l);
    let program = p.parse_program();

    if !p.errors.is_empty() {
        write_parser_errors(&p.errors, output)?;
        return Ok(None);
    }

    let mut comp = Compiler::new_with_state(symbol_table, constants);
    if let Err(e) = comp.compile(program.make_node()) {
        write!(output, "Woops! Compilation failed:\n {}\n", e)?;
        return Ok(None);
    }
    let modified_constants = comp.constants.clone();
    let modified_symbol_table = comp.symbol_table.clone();

    let mut machine = vm::VM::new_with_global_store(comp.byte_code(), globals);
    if let Err(e) = machine.run() {
        write!(output, "Woops! Executing bytecode failed:\n {}\n", e)?;
        return Ok(None);
    }

    let Some(stack_top) = machine.result() else {
        writeln!(output, "Woops! Stack top is empty")?;
        return Ok(None);
    };

    writeln!(output, "{}", stack_top.inspect())?;

    Ok(Some((
        modified_constants,
        machine.globals,
        modified_symbol_table,
    )))
}
