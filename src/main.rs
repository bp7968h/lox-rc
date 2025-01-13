use lox_rc::vm::VM;
use lox_rc::InterpretError;
use std::env;
use std::fs;
use std::process;

fn main() {
    if let Some(args) = env::args().nth(1) {
        match fs::read_to_string(&args) {
            Ok(content) => {
                let mut vm = VM::new();
                if env::var("DEBUG").is_ok() {
                    vm.set_debug(true);
                }

                match vm.interpret(&content) {
                    Ok(_) => (),
                    Err(e) => match e {
                        InterpretError::CompileError => process::exit(65),
                        InterpretError::RuntimeError => process::exit(70),
                    },
                }
            }
            Err(e) => {
                eprintln!("Error Reading Filer: [{}]", e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Usage: jlox-rc <source_file>");
        process::exit(1);
    }
}
