extern crate rustyline;
use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
    let mut line_editor = Editor::<()>::new();
    loop {
        let line_result = line_editor.readline("> ");
        match line_result {
            Ok(line) => println!("{}", line),
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
