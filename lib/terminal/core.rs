use std::{
    fmt,
    io::BufReader,
    process::{Command, ExitStatus, Stdio},
};

pub type TerminalOutput = BufReader<std::process::ChildStdout>;

pub enum TerminalExecutionResult {
    // ReadOutput - capture stdout and return the result
    ReadOutput {
        status: ExitStatus,
        output: TerminalOutput,
    },

    // TerminationStatus - captures the result of a terminal execution
    // and returns the result.
    TerminationStatus {
        status: ExitStatus,
    },
}
pub struct TerminalExecution {
    pub command: String,
    pub args: Vec<String>,
}

impl TerminalExecution {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self { command, args }
    }

    pub fn run(&self, return_stdout: bool) -> Result<TerminalExecutionResult, ()> {
        if return_stdout {
            return self.run_return_output();
        }
        self.run_return_status()
    }

    fn run_return_output(&self) -> Result<TerminalExecutionResult, ()> {
        let child = std::process::Command::new(&self.command)
            .args(self.args.iter())
            .spawn()
            .expect("failed to run command");

        let output = child.wait_with_output().expect("command execution failed");
        Ok(TerminalExecutionResult::TerminationStatus {
            status: output.status,
        })
    }

    fn run_return_status(&self) -> Result<TerminalExecutionResult, ()> {
        let mut child = Command::new(&self.command)
            .args(self.args.iter())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to run command");
        let stdout = child.stdout.take().expect("command did not have output");
        let output = BufReader::new(stdout);
        let status = child.wait().expect("command execution failed");
        Ok(TerminalExecutionResult::ReadOutput { status, output })
    }
}

impl fmt::Display for TerminalExecution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = self.command.clone();
        let args = self.args.clone().join(" ");
        if args.len().ge(&0) && !args.is_empty() {
            res.push(' ');
            res.push_str(&args);
        }
        write!(f, "{}", res)
    }
}
