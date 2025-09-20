use crate::terminal::core::TerminalExecution;
use bon::builder;

#[derive(PartialEq, Eq)]
pub(crate) enum TerminalCommand {
    Which {
        program: String,
    },

    Grep {
        pattern: String,
        files: Vec<String>,
        invert_match: Option<bool>,
        case_insensitive: Option<bool>,
        show_count: Option<bool>,
        search_directories_recursively: Option<bool>,
    },
    Cat {
        files: Vec<String>,
        number_lines: Option<bool>,
        show_ends: Option<bool>,
    },
    GoTest {
        package: Option<String>,
        test_file: Option<String>,
        verbose: Option<bool>,
        test_pattern: Option<String>,
        build_tags: Option<Vec<String>>,
    },
}

#[bon::bon]
impl TerminalCommand {
    #[builder]
    pub fn which(program: String) -> Self {
        Self::Which { program }
    }

    #[builder]
    pub fn grep(
        pattern: String,
        files: Vec<String>,
        invert_match: Option<bool>,
        case_insensitive: Option<bool>,
        show_count: Option<bool>,
        search_directories_recursively: Option<bool>,
    ) -> Self {
        Self::Grep {
            pattern,
            files,
            invert_match,
            case_insensitive,
            show_count,
            search_directories_recursively,
        }
    }

    #[builder]
    pub fn cat(files: Vec<String>, number_lines: Option<bool>, show_ends: Option<bool>) -> Self {
        Self::Cat {
            files,
            number_lines,
            show_ends,
        }
    }

    #[builder]
    pub fn go_test(
        package: Option<String>,
        test_file: Option<String>,
        verbose: Option<bool>,
        test_pattern: Option<String>,
        build_tags: Option<Vec<String>>,
    ) -> Self {
        Self::GoTest {
            package,
            test_file,
            verbose,
            test_pattern,
            build_tags,
        }
    }
    pub fn available(&self) -> bool {
        let cmd_which = Self::which_command();
        let cmd_name = match self {
            TerminalCommand::Which { .. } => cmd_which,
            TerminalCommand::Grep { .. } => Self::grep_command(),
            TerminalCommand::Cat { .. } => Self::cat_command(),
            TerminalCommand::GoTest { .. } => "go",
        };

        if cmd_name == cmd_which {
            return true;
        }

        std::process::Command::new(cmd_which)
            .arg(cmd_name)
            .output()
            .is_ok()
    }

    pub fn to_terminal_execution(&self) -> TerminalExecution {
        match self {
            TerminalCommand::Which { program } => {
                TerminalExecution::new(Self::which_command().to_string(), vec![program.clone()])
            }
            TerminalCommand::Grep {
                pattern,
                files,
                case_insensitive,
                show_count,
                search_directories_recursively,
                invert_match,
            } => {
                let mut args = Vec::new();

                if let Some(true) = case_insensitive {
                    args.push("-i".to_string());
                }
                if let Some(true) = show_count {
                    args.push("-c".to_string());
                }
                if let Some(true) = search_directories_recursively {
                    args.push("-r".to_string());
                }
                if let Some(true) = *invert_match {
                    args.push("-v".to_string());
                }

                args.push(pattern.clone());
                args.extend(files.clone());

                TerminalExecution::new(Self::grep_command().to_string(), args)
            }
            TerminalCommand::Cat {
                files,
                number_lines,
                show_ends,
            } => {
                let mut args = Vec::new();

                if let Some(true) = number_lines {
                    args.push("-n".to_string());
                }
                if let Some(true) = show_ends {
                    args.push("-E".to_string());
                }

                args.extend(files.clone());

                TerminalExecution::new(Self::cat_command().to_string(), args)
            }
            TerminalCommand::GoTest {
                package,
                test_file,
                verbose,
                test_pattern,
                build_tags,
            } => {
                let mut args = vec!["test".to_string()];

                if let Some(true) = verbose {
                    args.push("-v".to_string());
                }

                if let Some(pattern) = test_pattern {
                    args.push("-run".to_string());
                    args.push(pattern.clone());
                }

                if let Some(mut tags) = build_tags.clone() {
                    let tags = tags.iter_mut().reduce(|expr, tag| {
                        expr.push(',');
                        expr.push_str(tag);
                        expr
                    });
                    let mut arg: String = "-tags=".to_string();
                    arg.insert_str(0, tags.unwrap());
                    args.push("-tags".to_string());
                }

                // Add package or test file
                if let Some(file) = test_file {
                    args.push(file.clone());
                } else if let Some(pkg) = package {
                    args.push(pkg.clone());
                } else {
                    args.push("./...".to_string()); // Default to all packages
                }

                TerminalExecution::new("go".to_string(), args)
            }
        }
    }

    fn which_command() -> &'static str {
        "which"
    }

    fn cat_command() -> &'static str {
        "cat"
    }

    fn grep_command() -> &'static str {
        "grep"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use googletest::assert_that;
    use googletest::prelude::*;

    use rstest::rstest;

    #[gtest]
    #[rstest]
    #[case(TerminalCommand::Which {program: "go".to_string()}, "which go")]
    #[case(TerminalCommand::Which {program: "".to_string()}, "which")]
    fn which_command(#[case] command: TerminalCommand, #[case] expected: &str) {
        assert_that!(command.to_terminal_execution().to_string(), eq(expected))
    }

    #[gtest]
    #[rstest]
    #[case(TerminalCommand::grep()
        .pattern("\"needle\"".to_string())
        .files(vec!["haystack.txt".to_string()])
        .call(), 
        "grep \"needle\" haystack.txt")]
    #[case(TerminalCommand::grep()
        .pattern("\"needle\"".to_string())
        .files(vec!["haystack.txt".to_string()])
        .case_insensitive(true)
        .call(), 
        "grep -i \"needle\" haystack.txt")]
    #[case(TerminalCommand::grep()
        .pattern("\"needle\"".to_string())
        .files(vec!["haystack.txt".to_string()])
        .case_insensitive(true)
        .call(), 
        "grep -i \"needle\" haystack.txt")]
    #[case(TerminalCommand::grep()
        .pattern("\"needle\"".to_string())
        .files(vec!["haystack.txt".to_string()])
        .show_count(true)
        .call(), 
        "grep -c \"needle\" haystack.txt")]
    #[case(TerminalCommand::grep()
        .pattern("\"needle\"".to_string())
        .files(vec!["haystack.txt".to_string()])
        .invert_match(true)
        .call(), 
        "grep -v \"needle\" haystack.txt")]
    fn grep_command(#[case] command: TerminalCommand, #[case] expected: &str) {
        assert_that!(command.to_terminal_execution().to_string(), eq(expected))
    }

    #[gtest]
    #[rstest]
    #[case(TerminalCommand::cat()
        .files(vec!["file1.txt".to_string()])
        .call(),
        "cat file1.txt")]
    #[case(TerminalCommand::cat()
        .files(vec!["file1.txt".to_string(), "file2.txt".to_string()])
        .call(),
        "cat file1.txt file2.txt")]
    #[case(TerminalCommand::cat()
        .files(vec!["file.txt".to_string()])
        .number_lines(true)
        .call(),
        "cat -n file.txt")]
    #[case(TerminalCommand::cat()
        .files(vec!["file.txt".to_string()])
        .show_ends(true)
        .call(),
        "cat -E file.txt")]
    #[case(TerminalCommand::cat()
        .files(vec!["file.txt".to_string()])
        .number_lines(true)
        .show_ends(true)
        .call(),
        "cat -n -E file.txt")]
    fn cat_command(#[case] command: TerminalCommand, #[case] expected: &str) {
        assert_that!(command.to_terminal_execution().to_string(), eq(expected))
    }

    #[gtest]
    #[rstest]
    #[case(TerminalCommand::go_test()
        .call(),
        "go test ./...")]
    #[case(TerminalCommand::go_test()
        .package("./cmd/app".to_string())
        .call(),
        "go test ./cmd/app")]
    #[case(TerminalCommand::go_test()
        .test_file("main_test.go".to_string())
        .call(),
        "go test main_test.go")]
    #[case(TerminalCommand::go_test()
        .verbose(true)
        .call(),
        "go test -v ./...")]
    #[case(TerminalCommand::go_test()
        .test_pattern("TestFoo".to_string())
        .call(),
        "go test -run TestFoo ./...")]
    #[case(TerminalCommand::go_test()
        .verbose(true)
        .test_pattern("TestBar".to_string())
        .package("./pkg/utils".to_string())
        .call(),
        "go test -v -run TestBar ./pkg/utils")]
    fn go_test_command(#[case] command: TerminalCommand, #[case] expected: &str) {
        assert_that!(command.to_terminal_execution().to_string(), eq(expected))
    }
}
