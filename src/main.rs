extern crate rustyline;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::completion::Completer;
use std::env::args;
use std::process::Command;

struct BashCompleter{
    command: String,
}

impl Completer for BashCompleter {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let words: Vec<&str> = line.split(" ").collect();
        let last_word = words.iter().nth(words.len()-1).unwrap();
        let count_words = words.len();

        let script = format!(
                    // COMP_WORDS must be specified in a bash context because normal environment
                    // variables can't be arrays
                    "
                    COMP_CWORD={}
                    COMP_POINT={}
                    COMP_LINE=\"{} {}\"
                    COMP_WORDS=({} {})
                    _xfunc git _git
                    printf \"%s\\n\" \"${{COMPREPLY[@]}}\"
                    ",
                    count_words,
                    pos + self.command.len() + 1,
                    self.command,
                    line,
                    self.command,
                    line,
                );
        let completion_result = Command::new("bash")
            .args(vec![
                "-lic",
                script.as_str(),
            ])
            .output();
        let completion_str = match completion_result {
            Ok(completion_string) => String::from_utf8(completion_string.stdout).unwrap(),
            Err(err) => {
                return Err(ReadlineError::Io(err));
            },
        };

        let word_len = last_word.len();
        let completions: Vec<String> = completion_str.split("\n").map(|s| s.to_string()).collect();

        Ok((pos-word_len, completions))
    }
}

fn main() {
    // TODO use a real arg parser
    let program = args().nth(1).expect("no program specified");
    let mut line_editor = Editor::<BashCompleter>::new();
    line_editor.set_completer(Some(BashCompleter{command: program.to_string()}));
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
