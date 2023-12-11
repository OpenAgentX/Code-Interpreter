use std::env;
// use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;

// SubprocessCodeInterpreter is assumed to be implemented elsewhere
pub trait SubprocessCodeInterpreter {
    // Define the necessary methods for subprocess code interpretation
}

pub struct AppleScript {
    config: Config,
    start_cmd: String,
}

pub struct Config {
    // Define the configuration structure
    // Add any configuration parameters needed for AppleScript
}

impl AppleScript {
    const FILE_EXTENSION: &'static str = "sh";
    const PROPER_NAME: &'static str = "Shell";

    fn new(config: Config) -> Self {
        let start_cmd = if cfg!(windows) {
            "cmd.exe".to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "bash".to_string())
        };

        AppleScript { config, start_cmd }
    }

    fn preprocess_code(&self, code: &str) -> String {
        preprocess_code(code)
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

fn preprocess_code(code: &str) -> String {
    // Your implementation for preprocessing goes here
    // let mut processed_code = add_active_line_prints(code);
    // processed_code.push_str("\necho \"##end_of_execution##\"");

    // Escape double quotes
    let mut code = code.replace("\"", "\\\"");

    // Wrap in double quotes
    code = format!("\"{}\"", code);

    // Prepend start command for AppleScript
    format!("osascript -e {}", code)
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

pub async fn run_applescript_command(shell_code: &str) -> Result<String> {

    let config = Config {
        // Initialize configuration parameters
    };
    let shell = AppleScript::new(config);
    // println!("\n\n =================================================");
    let processed_code = shell.preprocess_code(shell_code);
    // println!("─❯: {}", &processed_code);
    // println!("\n\n =================================================");
    let child = Command::new(&shell.start_cmd)
        .arg("-c")
        .arg(&processed_code)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output().expect("failed to wait on child");

    let mut output_str;
    if output.status.success() {
        output_str = "Successfully, Code output:".to_string();
        output_str += &String::from_utf8(output.stdout)?;

    } else {
        output_str = String::from_utf8(output.stderr)?;
        // println!("{:#?}", output_str);
    }

    // println!("output_str: \n{:?}", output_str);

    Ok(output_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shell_ls() {
        // Example usage
        let code = "ls";
        let _ = run_applescript_command(code).await;
        // Further processing based on the interpreter's methods
    }
}
