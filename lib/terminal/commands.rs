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
        build_tags: Option<String>,
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
