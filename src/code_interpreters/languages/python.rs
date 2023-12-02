// use error_chain::error_chain;

// use std::collections::HashSet;
use std::io::Write;
use std::error::Error;

use std::process::{Command, Stdio};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};


// error_chain!{
//    errors { CmdError }
//    foreign_links {
//        Io(std::io::Error);
//        Utf8(std::string::FromUtf8Error);
//    }
// }

pub fn python_interpreter(code: &str) -> Result<String, Box<dyn Error>> {
    // For Python code highlighting.
    // Load these once at the start of your program
    println!("\n =================================================");
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension("py").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);
    let hl_code = "\n".to_string() + code;
    for line in LinesWithEndings::from(&hl_code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        print!("{}", escaped);
    }

    // let plain = Style::default();

    // let style = Style {
    //     foreground: Color::WHITE,
    //     background: Color::BLACK,
    //     font_style: FontStyle::default(),
    // };
    // let s = as_24_bit_terminal_escaped(&[(plain, "")], true);
    // print!("{}", s);
    // Python code highlighting end.

    println!("\n\n =================================================");
    let mut child;
    if code.starts_with("!") {
        // println!("==================== pip ====================");

        let cmd_code = code.replace("!pip ", "");
        let cmd_code: Vec<_> = cmd_code.split(" ").collect();
        // println!("================{:?}", cmd_code);
        child = Command::new("pip3")
            .args(cmd_code)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
    } else {
        let cmd_args: Vec<_> = "-i -q -u ".split(" ").collect();
        child = Command::new("python3")
            .args(cmd_args)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        // let mut code = code.as_bytes();
        // code += code + "\n";
        // child.stdin
        //     .as_mut()
        //     .ok_or("Child process stdin has not been captured!")?
        //     .write(code)?;

        // child.stdin
        //     .as_mut()
        //     .ok_or("Child process stdin has not been captured!")?
        //     .write_all(b"\t")?;

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(err) = stdin.write_all(code.as_bytes()) {
                eprintln!("Error writing to process stdin: {}", err);
            }
            if let Err(err) = stdin.flush() {
                eprintln!("Error flushing process stdin: {}", err);
            }
        }

        // println!("==================== python ====================");
        // let filename = "test.py";
        // let mut filebuf = File::create(filename)?;

        // write!(filebuf, "{}", code)?;

        // // filebuf.write_all(code);
        // child = Command::new("python3")
        //     .args(&[filename])
        //     .stdout(Stdio::piped())
        //     .stderr(Stdio::piped())
        //     .spawn()
        //     .unwrap();
    }

    let output = child.wait_with_output().expect("failed to wait on child");
    // println!("output: \n{:?}", output);

    let mut output_str: String = String::new();
    if output.status.success() {
        output_str = String::from_utf8(output.stdout)?;
        // let words = output_str.split_whitespace()
        //     .map(|s| s.to_lowercase())
        //     .collect::<HashSet<_>>();
        // println!("Found {} unique words:", words.len());
        // println!("{:#?}", output_str);
    } else {
        output_str = String::from_utf8(output.stderr)?;
        // println!("{:#?}", output_str);
    }

    // println!("output_str: \n{:?}", output_str);

    Ok(output_str)
}



// fn main() -> Result<()> {
//     let mut child = Command::new("python").stdin(Stdio::piped())
//         .stderr(Stdio::piped())
//         .stdout(Stdio::piped())
//         .spawn()?;

//     child.stdin
//         .as_mut()
//         .ok_or("Child process stdin has not been captured!")?
//         .write_all(b"import this; copyright(); credits(); exit()")?;

//     let output = child.wait_with_output()?;

//     if output.status.success() {
//         let raw_output = String::from_utf8(output.stdout)?;
//         let words = raw_output.split_whitespace()
//             .map(|s| s.to_lowercase())
//             .collect::<HashSet<_>>();
//         println!("Found {} unique words:", words.len());
//         println!("{:#?}", words);
//         Ok(())
//     } else {
//         let err = String::from_utf8(output.stderr)?;
//         error_chain::bail!("External command failed:\n {}", err)
//     }
// }