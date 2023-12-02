use std::env;
use std::process::{Command, Stdio};

// SubprocessCodeInterpreter is assumed to be implemented elsewhere
pub trait SubprocessCodeInterpreter {
    // Define the necessary methods for subprocess code interpretation
}

pub struct Shell {
    config: Config,
    start_cmd: String,
}

pub struct Config {
    // Define the configuration structure
    // Add any configuration parameters needed for Shell
}

impl Shell {
    const FILE_EXTENSION: &'static str = "sh";
    const PROPER_NAME: &'static str = "Shell";

    fn new(config: Config) -> Self {
        let start_cmd = if cfg!(windows) {
            "cmd.exe".to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "bash".to_string())
        };

        Shell { config, start_cmd }
    }

    fn preprocess_code(&self, code: &str) -> String {
        preprocess_shell(code)
    }

    fn line_postprocessor(&self, line: &str) -> String {
        line.to_string()
    }

    fn detect_active_line(&self, line: &str) -> Option<usize> {
        if line.contains("##active_line") {
            let active_line_str = line.split("##active_line").nth(1)?.split("##").next()?;
            active_line_str.parse().ok()
        } else {
            None
        }
    }

    fn detect_end_of_execution(&self, line: &str) -> bool {
        line.contains("##end_of_execution##")
    }
}

fn preprocess_shell(code: &str) -> String {
    // Your implementation for preprocessing goes here
    let mut processed_code = add_active_line_prints(code);
    processed_code.push_str("\necho \"##end_of_execution##\"");
    processed_code
}

fn add_active_line_prints(code: &str) -> String {
    // Your implementation for adding active line prints goes here
    let mut lines: Vec<String> = code.lines().enumerate().map(|(index, line)| {
        format!("echo \"##active_line{}##\"\n{}", index + 1, line)
    }).collect();
    lines.join("\n")
}

fn has_multiline_commands(script_text: &str) -> bool {
    // Your implementation for detecting multiline commands goes here
    let continuation_patterns = vec![
        r"\\$",
        r"\|$",
        r"&&\s*$",
        r"\|\|\s*$",
        r"<\($",
        r"\($",
        r"{\s*$",
        r"\bif\b",
        r"\bwhile\b",
        r"\bfor\b",
        r"do\s*$",
        r"then\s*$",
    ];

    script_text.lines().any(|line| {
        continuation_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern).unwrap().is_match(line.trim_end())
        })
    })
}

// fn main() {
//     // Example usage
//     let config = Config {
//         // Initialize configuration parameters
//     };
//     let shell = Shell::new(config);

//     let code = "ls";
//     let processed_code = shell.preprocess_code(code);

//     // Your code execution logic goes here

//     let result = Command::new(&shell.start_cmd)
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         // .arg(&shell.start_cmd)
//         .arg("-c")
//         .arg(&processed_code)
//         .output()
//         .expect("Failed to execute shell command");

//     let output = String::from_utf8_lossy(&result.stdout);
//     println!("Output:\n{}", output);

//     // Further processing based on the interpreter's methods
// }
