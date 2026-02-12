//! Command execution utilities
//!
//! Provides cross-platform command execution that hides console windows on Windows.

use std::ffi::OsStr;
use std::process::{Command, Output, Stdio};
use std::io::Result;

/// Create a new Command that won't show a console window on Windows
pub fn new_command<S: AsRef<OsStr>>(program: S) -> Command {
    let mut cmd = Command::new(program);

    // On Windows, hide the console window
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd
}

/// Create a new Command with hidden console and specified arguments
pub fn new_command_with_args<S, I, A>(program: S, args: I) -> Command
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = A>,
    A: AsRef<OsStr>,
{
    let mut cmd = new_command(program);
    cmd.args(args);
    cmd
}

/// Run a command and capture its output, hiding console on Windows
pub fn run_command<S: AsRef<OsStr>>(program: S, args: &[&str]) -> Result<Output> {
    new_command_with_args(program, args).output()
}

/// Run a command with environment variables
pub fn run_command_with_env<S: AsRef<OsStr>>(
    program: S,
    args: &[&str],
    env_vars: &[(&str, &str)],
) -> Result<Output> {
    let mut cmd = new_command_with_args(program, args);
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    cmd.output()
}

/// Spawn a command without waiting for it to complete
pub fn spawn_command<S: AsRef<OsStr>>(program: S, args: &[&str]) -> Result<std::process::Child> {
    new_command_with_args(program, args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// Spawn a command with stdout/stderr piped for reading
pub fn spawn_command_with_output<S: AsRef<OsStr>>(program: S, args: &[&str]) -> Result<std::process::Child> {
    new_command_with_args(program, args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
