use nix::{
    errno::Errno,
    sys::{
        signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal},
        wait::waitpid,
    },
    unistd::{close, execvp, fork, getpgrp, pipe, read, setpgid, tcsetpgrp, ForkResult},
};
use std::{
    ffi::CString,
    io::{stdin, stdout, Write},
};
mod parser;
use parser::parse_command_line;

fn main() {
    shell_loop()
}

fn shell_loop() {
    ignore_tty_signals();

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
    let (pipe_read, pipe_write) = pipe().unwrap();

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            // 子プロセスをプロセスグループリーダーにする
            setpgid(child, child).unwrap();

            // 子プロセスのプロセスグループをフォアグラウンドプロセスに設定する
            tcsetpgrp(0, child).unwrap();

            // 子プロセスとの同期を終了する
            close(pipe_read).unwrap();
            close(pipe_write).unwrap();

            waitpid(child, None).ok();

            // 自分のプロセスグループをフォアグラウンドプロセスに戻す
            tcsetpgrp(0, getpgrp()).unwrap();
        }
        Ok(ForkResult::Child) => {
            restore_tty_signals();
            close(pipe_write).unwrap();

            loop {
                let mut buf = [0];
                match read(pipe_read, &mut buf) {
                    Err(e) if e == Errno::EINTR => (),
                    _ => break,
                }
            }

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

// 無視設定にする関数
fn ignore_tty_signals() {
    let sa = SigAction::new(SigHandler::SigIgn, SaFlags::empty(), SigSet::empty());
    unsafe {
        sigaction(Signal::SIGTSTP, &sa).unwrap();
        sigaction(Signal::SIGTTIN, &sa).unwrap();
        sigaction(Signal::SIGTTOU, &sa).unwrap();
    }
}

// デフォルト設定に戻す関数
fn restore_tty_signals() {
    let sa = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());
    unsafe {
        sigaction(Signal::SIGTSTP, &sa).unwrap();
        sigaction(Signal::SIGTTIN, &sa).unwrap();
        sigaction(Signal::SIGTTOU, &sa).unwrap();
    }
}
