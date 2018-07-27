extern crate rustyline;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::process::Command;

fn main() {
    let mut line_editor = Editor::<()>::new();
    loop {
        let line_result = line_editor.readline("> ");
        match line_result {
            Ok(line) => {
                let mut process_result = Command::new("git")
                    .args(line.split(" ").collect::<Vec<&str>>())
                    .spawn();
                match process_result {
                    Ok(mut process) => {
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
