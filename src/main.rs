extern crate rustyline;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::env::args;
use std::process::Command;

fn main() {
    // TODO use a real arg parser
    let program = args().nth(1).expect("no program specified");
    let mut line_editor = Editor::<()>::new();
    loop {
        let line_result = line_editor.readline(format!(">{} ", program.clone()).as_str());
        match line_result {
            Ok(line) => {
                let mut process_result = Command::new(&program)
                    // TODO deal with quoted strings
                    .args(line.split(" ").collect::<Vec<&str>>())
                    .spawn();
                match process_result {
                    Ok(mut process) => {
                        // TODO maybe display return code?
                        let _ = process.wait();
                    },
                    Err(err) => println!("error while trying to run command: {:?}", err),
                }
            },
            Err(ReadlineError::Interrupted) => {
                return;
            },
            Err(ReadlineError::Eof) => {
                return;
            },
            Err(err) => {
                eprintln!("{:?}", err);
                return;
            }
        }
    }
}
