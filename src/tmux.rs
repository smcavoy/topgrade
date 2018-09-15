use super::executor::Executor;
use super::terminal::Terminal;
use super::utils::which;
use super::utils::{Check, PathExt};
use directories::BaseDirs;
use failure::Error;
use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

pub fn run_tpm(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(tpm) = base_dirs
        .home_dir()
        .join(".tmux/plugins/tpm/bin/update_plugins")
        .if_exists()
    {
        terminal.print_separator("tmux plugins");

        let success = || -> Result<(), Error> {
            Executor::new(&tpm, dry_run).arg("all").spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("tmux", success));
    }

    None
}

fn has_session(tmux: &Path, session_name: &str) -> Result<bool, io::Error> {
    Ok(Command::new(tmux)
        .args(&["has-session", "-t", session_name])
        .spawn()?
        .wait()?
        .success())
}

fn run_in_session(tmux: &Path, command: &str) -> Result<(), Error> {
    Command::new(tmux)
        .args(&["new-window", "-a", "-t", "topgrade:1", command])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_in_tmux() -> ! {
    let tmux = which("tmux").expect("Could not find tmux");
    let command = env::args().collect::<Vec<String>>().join(" ");

    if has_session(&tmux, "topgrade").expect("Error launching tmux") {
        run_in_session(&tmux, &command).expect("Error launching tmux");

        let err = Command::new(tmux).args(&["attach", "-t", "topgrade"]).exec();

        panic!("{:?}", err);
    } else {
        let err = Command::new(tmux)
            .args(&[
                "new-session",
                "-s",
                "topgrade",
                "-n",
                "topgrade",
                &command,
                ";",
                "set",
                "remain-on-exit",
                "on",
            ]).exec();

        panic!("{:?}", err);
    }
}
