use std::io::{stdin, stdout, Write};

fn main() {
    shell_loop()
}

fn shell_loop() {
    while let Some(line) = shell_read_line() {
        println!("{}", line)
    }
}

fn shell_read_line() -> Option<String> {
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
