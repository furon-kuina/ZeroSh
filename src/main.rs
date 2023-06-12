use nix::sys::wait::waitpid;
use nix::unistd::execvp;
use nix::unistd::{fork, ForkResult};
use std::ffi::CString;
use std::io::{stdin, stdout, Write};

mod parser;
use parser::parse_command_line;

fn main() {
    shell_loop()
}

fn shell_loop() {
    while let Some(input) = read_line() {
        let piped_commands = match parse_command_line(input) {
            Some(action) => action,
            None => continue,
        };
        execute_piped_commands(piped_commands);
    }
}

fn execute_piped_commands(piped_commands: Vec<String>) {
    for command_string in piped_commands {
        let command = command_string
            .trim()
            .split(' ')
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        execute_unit_command(command)
    }
}

fn execute_unit_command(command: Vec<String>) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            match waitpid(child, None) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("wait error: {}", e);
                }
            };
        }
        Ok(ForkResult::Child) => {
            let args = command
                .into_iter()
                .map(|c| CString::new(c).unwrap())
                .collect::<Vec<_>>();
            match execvp(&args[0], &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "exec error: {}, filename: {:?}, args: {:?}",
                        e, &args[0], &args
                    );
                }
            };
        }
        Err(e) => {
            eprintln!("fork error: {}", e);
        }
    }
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
