mod ast;
mod evaluator;
mod memory;
mod restrict;
mod transpiler;
mod type_checker;

use evaluator::*;
use type_checker::*;

use clap::Parser;

/// A C interpreter to detect UB via C99 restrict
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the C source file to be interpreted
    #[clap(short, long)]
    source_file: String,
}

fn main() {
    // Honor RUST_LOG level, i.e. error, warning, info, debug, trace
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    env_logger::init();

    let args = Args::parse();
    let transpiler = transpiler::Transpiler::default();

    match transpiler.transpile(&args.source_file) {
        // Transpile the C source code to our Clight AST.
        // Fails if unsupported language constructs are used.
        Ok(mut ast) => {
            log::trace!("{:#?}", ast);
            // Type check the program and annotate expressions with their types.
            // Fails if the AST could not be typed.
            match ast.type_check(&mut TypingContext::new()) {
                Ok(()) => match evaluate_program(&mut ast) {
                    Ok(res) => log::debug!("{:?}", res),
                    Err(e) => println!("{:?}", e),
                },
                Err(e) => {
                    log::error!("Error during type checking:\n {}", e);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to transpile the program!\n{:?}", e);
        }
    }
}
