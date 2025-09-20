use crate::terminal::core::TerminalExecution;

pub(crate) enum Commands {
    Which {
        program: String,
    },
    Grep {
        pattern: String,
        files: Vec<String>,
        case_insensitive: Option<bool>,
        line_numbers: Option<bool>,
        recursive: Option<bool>,
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

impl Commands {
    pub fn available(&self) -> bool {
        let cmd_which = Self::which_command();
        let cmd_name = match self {
            Commands::Which { .. } => cmd_which,
            Commands::Grep { .. } => Self::grep_command(),
            Commands::Cat { .. } => Self::cat_command(),
            Commands::GoTest { .. } => "go",
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
            Commands::Which { program } => {
                TerminalExecution::new(Self::which_command().to_string(), vec![program.clone()])
            }
            Commands::Grep {
                pattern,
                files,
                case_insensitive,
                line_numbers,
                recursive,
            } => {
                let mut args = Vec::new();

                if let Some(true) = case_insensitive {
                    args.push("-i".to_string());
                }
                if let Some(true) = line_numbers {
                    args.push("-n".to_string());
                }
                if let Some(true) = recursive {
                    args.push("-r".to_string());
                }

                args.push(pattern.clone());
                args.extend(files.clone());

                TerminalExecution::new(Self::grep_command().to_string(), args)
            }
            Commands::Cat {
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
            Commands::GoTest {
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
