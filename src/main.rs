extern crate rustyline;
extern crate serde;
extern crate serde_json;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::completion::Completer;
use std::env::args;
use std::path::PathBuf;
use std::process::Command;
use std::env::home_dir;
use std::fs::create_dir;
use std::fs::File;
use std::collections::HashMap;

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

    let (history_file_option, prompts_config_path_option) = match home_dir() {
        Some(home) => {
            let config_dir: PathBuf = [home, PathBuf::from(".tpol")].iter().collect();
            if !config_dir.exists() {
                println!("creating config dir at {}", config_dir.to_str().unwrap());
                create_dir(&config_dir).unwrap();
            }

            (
                {
                    let history_dir: PathBuf = [&config_dir, &PathBuf::from("history")].iter().collect();
                    if !history_dir.exists() {
                        println!("creating history dir at {}", history_dir.to_str().unwrap());
                        create_dir(&history_dir).unwrap();
                    }

                    let command_history: PathBuf = [history_dir, PathBuf::from(&program)].iter().collect();
                    if !command_history.exists() {
                        println!("creating history file for {} at {}", &program, command_history.to_str().unwrap());
                        let _ = File::create(&command_history).unwrap();
                    }

                    Some(command_history)
                },
                {
                    let prompts_config_path: PathBuf = [&config_dir, &PathBuf::from("prompts.json")].iter().collect();
                    Some(prompts_config_path)
                },
            )
        }
        None => (None, None)
    };

    if let Some(ref history_file) = history_file_option.clone() {
        if let Err(err) = line_editor.load_history(history_file) {
            eprintln!("Warning: error while trying to load history file {}: {}", history_file.to_str().unwrap(), err);
        }
    } else {
        eprintln!("Warning: can't find home directory, no history will be saved");
    }

    let program_to_prompt_string_command = match prompts_config_path_option {
        Some(prompts_config_path) => {
            File::open(prompts_config_path)
                .map(|file| {
                    let prompts_config: HashMap<String, String> = serde_json::from_reader(file)
                        .unwrap_or(HashMap::new());
                    prompts_config
                })
                .unwrap_or(HashMap::new())
        }
        None => HashMap::new(),
    };

    let prompt_string_command_option = program_to_prompt_string_command.get(&program);

    loop {
        let prompt_string = match prompt_string_command_option.as_ref() {
            Some(ref command) => {
                match Command::new("bash").args(vec!["-lic", command]).output() {
                    Ok(output) => String::from_utf8(output.stdout).unwrap(),
                    Err(_) => {
                        // TODO log error
                        "".to_string()
                    }
                }
            },
            None => "".to_string(),
        };

        let line_result = line_editor.readline(format!("{}>{} ", prompt_string, &program).as_str());
        match line_result {
            Ok(line) => {
                if line == "" || line == "!" {
                    // ignore empty commands
                } else {
                    let mut command = if line.starts_with("!") {
                        let parts = line.split(" ");
                        let command = &parts.clone().nth(0).unwrap()[1..];
                        let args: Vec<&str> = parts.skip(1).collect();
                        println!("{} {:?}", command, args);
                        let mut command = Command::new(command);
                        command.args(args);
                        command
                    } else {
                        let mut command = Command::new(&program);
                        // TODO deal with quoted strings
                        command.args(line.split(" ").collect::<Vec<&str>>());
                        command
                    };
                    let mut process_result = command.spawn();
                    match process_result {
                        Ok(mut process) => {
                            // TODO maybe display return code?
                            let _ = process.wait();
                            // TODO maybe only save in history if it's successful?
                            line_editor.add_history_entry(&line);
                        },
                        Err(err) => println!("error while trying to run command: {:?}", err),
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                break;
            },
            Err(ReadlineError::Eof) => {
                break;
            },
            Err(err) => {
                eprintln!("{:?}", err);
                break;
            }
        }
    }

    if let Some(ref history_file) = history_file_option {
        if let Err(err) = line_editor.save_history(history_file) {
            eprintln!("Warning: error while trying to load history file {}: {}", history_file.to_str().unwrap(), err);
        }
    }
}
