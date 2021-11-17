//! A concise trait for executable programs.
use std::io;
use std::process;

#[derive(Debug)]
pub enum ExecutionError {

    /// We encountered an error while launching or waiting on the child process.
    Io(io::Error),

    /// The child process ran, but returned a non-zero exit code.
    Exit(process::ExitStatus)

}

impl From<io::Error> for ExecutionError {
    /// Wrap an [`io::Error`] in a [`ExecutionError::Io`]
    fn from(other: io::Error) -> ExecutionError {
        ExecutionError::Io(other)
    }
}

/// Convert a nonzero exit status into an [`ExecutionError`]
fn assert_success(status: process::ExitStatus) -> Result<(),ExecutionError> {
    match status.success() {
        true => Ok(()),
        false => Err(ExecutionError::Exit(status))
    }
}

/// This is the behavior we really need from [`process::Command`].
///
/// [`process::Command`] is very abstract, and that's great, but our needs are simpler. We
/// define that smaller interface here. This allows us to do two things:
///
/// 1. Add some simplifying behaviors to [`process::Command`] via its implementation of this trait.
/// 2. Inject mock [`Execute`] objects into [`crate::Git`] so that we do not actually have to
///    launch a subprocess for every unit test.
pub trait Execute {

    /// Execute the command with the provided `args`.
    ///
    /// Use this when we want to run a command, but we don't care about its output. If the
    /// implementation encounters any problems, it must communicate that by raising an
    /// [`ExecutionError`].
    fn capture_nothing(&mut self, args: &[&str]) -> Result<(),ExecutionError>;

    /// Capture the command's output as a [`String`].
    ///
    /// Use this when we want to run a command and capture its output for further processing. 
    fn capture_stdout(&mut self, args: &[&str]) -> Result<String,ExecutionError>;
}


impl Execute for process::Command {

    /// Execute command with its `stdout` & `stderr` forwarded to the console.
    fn capture_nothing(&mut self, args: &[&str]) -> Result<(),ExecutionError> {
        let status = self.args(args).status()?;
        assert_success(status)?;

        Ok(())
    }

    /// Execute command with its `stderr` forwarded to the console, and its `stdout` returned (on
    /// success) as a [`String`].
    fn capture_stdout(&mut self, args: &[&str]) -> Result<String,ExecutionError> {
        let output = self.args(args).output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
    }
}
