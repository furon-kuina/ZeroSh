use nix::unistd::execvp;
use std::ffi::CString;
use std::io::{stdin, stdout, Write};

fn main() {
    shell_loop()
}

fn shell_loop() {
    while let Some(line) = read_line() {
        let command = match parse_line(line) {
            Some(action) => action,
            None => continue,
        };
        execute_unit_command(command);
    }
}

fn execute_unit_command(command: Vec<String>) {
    let args = command
        .into_iter()
        .map(|c| CString::new(c).unwrap())
        .collect::<Vec<_>>();
    execvp(&args[0], &args).unwrap();
}

enum Action {
    UnitCommand(Vec<String>),
    Commands(Vec<Vec<String>>),
}

fn parse_line(line: String) -> Option<Vec<String>> {
    match line.is_empty() {
        true => None,
        false => {
            let commands = line.split(' ').map(|s| s.to_string()).collect();
            Some(commands)
        }
    }
}

fn read_line() -> Option<String> {
    print!("> ");
    stdout().flush().unwrap();
    let mut result = String::new();
    match stdin().read_line(&mut result) {
        Ok(size) => {
            if size == 0 {
                None
            } else {
                let result = result.trim_end();
                Some(result.to_string())
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    }
}
