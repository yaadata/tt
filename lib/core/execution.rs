use crate::core::errors::CommandExecutionError;
use std::{collections::BTreeMap, path::PathBuf};

pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CommandExecutionStatus {
    Started,
    Failed {
        code: Option<i8>,
        message: Option<String>,
    },
    Cancelled,
    Succeeded,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CommandStreamKind {
    Info,
    Diagnostic,
    Named(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CommandExecutionOutput {
    pub stream: CommandStreamKind,
    pub bytes: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CommandEvent {
    Executing(CommandExecutionOutput),
    Completed(CommandExecutionStatus),
}

pub struct CommandExecutionResult {
    pub output: Vec<CommandExecutionOutput>,
    pub status: CommandExecutionStatus,
}

pub trait CommandExecutor {
    fn execute(
        &self,
        command: &CommandSpec,
    ) -> Result<CommandExecutionResult, CommandExecutionError>;
}

pub trait CommandExecutionStream {
    fn next_event(&mut self) -> Result<Option<CommandEvent>, CommandExecutionError>;
    fn cancel(&mut self) -> Result<(), CommandExecutionError>;
}

pub trait StreamingCommandExecutor {
    fn execute(
        &self,
        command: &CommandSpec,
    ) -> Result<Box<dyn CommandExecutionStream>, CommandExecutionError>;
}

#[cfg(test)]
mod test {
    use std::ops::Index;

    use super::*;

    const PROGRAM: &str = "test-runner";
    const ARG: &str = "--parallel=1";
    const ACTIVE_DIR_KEY: &str = "ACTIVE_DIR";
    const ACTIVE_DIR_VALUE: &str = "/repository/tt/";

    fn env() -> BTreeMap<String, String> {
        BTreeMap::from([(ACTIVE_DIR_KEY.to_string(), ACTIVE_DIR_VALUE.to_string())])
    }

    fn command_spec() -> CommandSpec {
        CommandSpec {
            program: PROGRAM.to_string(),
            args: vec![ARG.to_string()],
            cwd: Some(PathBuf::from(ACTIVE_DIR_VALUE)),
            env: env(),
        }
    }

    struct StubCommandExecutor {
        result: CommandExecutionResult,
    }

    impl CommandExecutor for StubCommandExecutor {
        fn execute(
            &self,
            command: &CommandSpec,
        ) -> Result<CommandExecutionResult, CommandExecutionError> {
            assert_eq!(command.program, PROGRAM);
            assert_eq!(command.args, vec![ARG]);
            assert_eq!(command.cwd, Some(PathBuf::from(ACTIVE_DIR_VALUE)));
            assert_eq!(
                command.env.get(ACTIVE_DIR_KEY),
                Some(&ACTIVE_DIR_VALUE.to_string())
            );

            Ok(CommandExecutionResult {
                output: self.result.output.clone(),
                status: self.result.status.clone(),
            })
        }
    }

    #[test]
    fn command_executor_returns_configured_result() {
        // ========= [A]rrange =========
        const OUTPUT: &[u8] = b"finished";
        const STATUS: CommandExecutionStatus = CommandExecutionStatus::Succeeded;

        let executor = StubCommandExecutor {
            result: CommandExecutionResult {
                output: vec![CommandExecutionOutput {
                    stream: CommandStreamKind::Info,
                    bytes: OUTPUT.to_vec(),
                }],
                status: STATUS,
            },
        };

        // ========= [A]ct     =========
        let result = executor.execute(&command_spec());

        // ========= [A]ssert  =========
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.output.len(), 1);
        let output = result.output.index(0);
        assert_eq!(output.stream, CommandStreamKind::Info);
        assert_eq!(output.bytes, OUTPUT);
        assert_eq!(result.status, STATUS);
    }

    struct StubCommandStream {
        events: Vec<CommandEvent>,
        cancelled: bool,
    }

    impl CommandExecutionStream for StubCommandStream {
        fn next_event(&mut self) -> Result<Option<CommandEvent>, CommandExecutionError> {
            if self.events.is_empty() || self.cancelled {
                return Ok(None);
            }

            Ok(Some(self.events.remove(0)))
        }

        fn cancel(&mut self) -> Result<(), CommandExecutionError> {
            self.cancelled = true;
            Ok(())
        }
    }

    const COMMANDEVENT_BYTES: &[u8; 7] = b"started";

    struct StubStreamingCommandExecutor;

    impl StreamingCommandExecutor for StubStreamingCommandExecutor {
        fn execute(
            &self,
            _command: &CommandSpec,
        ) -> Result<Box<dyn CommandExecutionStream>, CommandExecutionError> {
            Ok(Box::new(StubCommandStream {
                events: vec![
                    CommandEvent::Executing(CommandExecutionOutput {
                        stream: CommandStreamKind::Info,
                        bytes: COMMANDEVENT_BYTES.to_vec(),
                    }),
                    CommandEvent::Completed(CommandExecutionStatus::Succeeded),
                ],
                cancelled: false,
            }))
        }
    }

    #[test]
    fn streaming_executor_yeilds_events_in_order() {
        // ========= [A]rrange =========
        let executor = StubStreamingCommandExecutor;
        let mut stream = executor.execute(&command_spec()).unwrap();
        // ========= [A]ct     =========
        match stream.next_event().unwrap() {
            Some(CommandEvent::Executing(output)) => {
                assert_eq!(output.stream, CommandStreamKind::Info);
                assert_eq!(output.bytes, COMMANDEVENT_BYTES.to_vec());
            }
            _ => panic!("expected executing event"),
        }
        // ========= [A]ssert  =========
        assert_eq!(
            stream.next_event().unwrap(),
            Some(CommandEvent::Completed(CommandExecutionStatus::Succeeded))
        );
        assert_eq!(stream.next_event().unwrap(), None);
    }

    #[test]
    fn command_stream_can_be_cancelled() {
        // ========= [A]rrange =========
        let executor = StubStreamingCommandExecutor;
        let mut stream = executor.execute(&command_spec()).unwrap();
        // ========= [A]ct     =========
        stream.cancel();
        // ========= [A]ssert  =========
        assert_eq!(stream.next_event().unwrap(), None);
    }
}
