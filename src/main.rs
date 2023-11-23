use core::time;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::process::{Command, Output, Stdio};
use std::thread::{sleep, self};

use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionFunctionsArgs, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestFunctionMessage, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, FinishReason, FunctionCall, Role,
};
use async_openai::Client;
use async_openai::config::OpenAIConfig;

use derive_builder::Builder;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style, Color, FontStyle};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};


#[derive(Debug, Serialize, Deserialize, Default, Clone, Builder, PartialEq)]
#[builder(name = "ChatCompletionRespondAssistantMessageArgs")]
#[builder(pattern = "mutable")]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "OpenAIError"))]
pub struct ChatCompletionRespondAssistantMessage {
    /// The role of the messages author, in this case `assistant`.
    #[builder(default = "Role::Assistant")]
    pub role: Role,
    /// The contents of the assistant message.
    pub message: Option<String>,
    /// The language of the assistant message.
    pub language: Option<String>,
    /// The code of the assistant message.
    pub code: Option<String>,
    /// Executes code on the user's machine, **in the users local environment**, and returns the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

enum Languages {
    Python,
    Shell,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let interpreter = Interpreter::default();
    // println!("{:?}", interpreter);

    let client = Client::new();

    let instructions = String::from(
        "You are Open Interpreter, a world-class programmer that can complete any goal by executing code.\n\
        First, write a plan. **Always recap the plan between each code block** (you have extreme short-term memory loss, so you need to recap the plan between each message block to retain it).\n\
        When you execute code, it will be executed **on the user\'s machine**. The user has given you **full and complete permission** to execute any code necessary to complete the task.\n\
        If you want to send data between programming languages, save the data to a txt or json.\n\
        You can access the internet. Run **any code** to achieve the goal, and if at first you don\'t succeed, try again and again.\n\
        You can install new packages.\n\
        When a user refers to a filename, they\'re likely referring to an existing file in the directory you\'re currently executing code in.\n\
        Write messages to the user in Markdown.\n\
        In general, try to **make plans** with as few steps as possible. As for actually executing code to carry out that plan, for *stateful* languages (like python, javascript, shell, but NOT for html which starts from 0 every time) **it\'s critical not to try to do everything in one code block.** You should try something, print information about it, then continue from there in tiny, informed steps. You will never get it on the first try, and attempting it in one go will often lead to errors you cant see.\n\
        You are capable of **any** task.\n\n\
        [User Info]\n\
        Name: shihua\n\
        CWD: /Users/shihua/Code/open-interpreter\n\
        SHELL: /bin/zsh\n\
        OS: Darwin\n\
        [Recommended Procedures]\n\
        If you encounter a traceback, don\'t try to use an alternative method yet. Instead:\n\n\
        **Write a message to the user explaining what happened and theorizing why. Do not try to run_code immediately after run_code has errored.**\n\n\
        If a solution is apparent (and is not simply changing methods / using a new package) attempt it.\n\
        If not, list these steps in a message to the user, then follow them one-by-one:\n\n\
        1. Create and run a minimal reproducible example.\n\
        2. Use dir() to verify correct imports. There may be a better object to import from the module.\n\
        3. Print docstrings of functions/classes using print(func.__doc__).\n\n\
        4. Print the functions results.
        Only then are you permitted to use an alternative method.\n\
        ---\n\
        To make a simple app, use HTML/Bulma CSS/JS.\n\
        First, plan. Think deeply about the functionality, what the JS will need to do, and how it will need to work with the HTML.\n\
        Then, **all in one** `html` code block (DO NOT `run_code` more than once, and NEVER use placeholders like \"// Javascript code here\" -- you\'re going to write the HTML/JS in one `run_code` function call):\n\
        Put Bulma CSS and anything else you need in <head>, write the <body> of the app (add lots of padding on the body with Bulma), write the JS into the <script> tag.\n\n\
        You probably want to center the app in a box with a border and make sure the body fills up the whole height of the page!\n\n\
        Write **LOTS of <!--comments--> throughout the HTML and // Javascript** to the user knows what\'s going on, and use whitespace/indentation properly.\n\n\
        This will automatically open the HTML file / simple app on the user\'s machine.\n\
        In your plan, include steps and, for relevant deprecation notices, **EXACT CODE SNIPPETS** -- these notices will VANISH once you execute your first line of code, so WRITE THEM DOWN NOW if you need them."
    );

    // Create a vector to store the messages
    let mut message_vec: Vec<ChatCompletionRequestMessage> = Vec::new();

    // Create instances of your message types
    let system_message = ChatCompletionRequestSystemMessageArgs::default()
        .content(&instructions)
        .build()?;

    let user_message = ChatCompletionRequestUserMessageArgs::default()
        .content("can you summarize the GitHub repository? https://github.com/KillianLucas/open-interpreter/")
        .build()?;

    // Add messages to the vector
    message_vec.push(system_message.into());
    message_vec.push(user_message.into());

    let _ = python_interpreter("import requests\n\n# Function to get the repository description\ndef get_repo_description(url):\n    response = requests.get(url)\n    description = response.json()['description']\n    return description\n\n# Get the repository description\nrepo_url = 'https://api.github.com/repos/KillianLucas/open-interpreter'\ndescription = get_repo_description(repo_url)\ndescription \n\n");

    let mut flag = 3;
    while flag < 3 {
        flag += 1;
        // Print the vector for demonstration
        println!(
            "\nStep {:?} : \n {}",
            flag,
            serde_json::to_string(&message_vec).unwrap()
        );
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-3.5-turbo-1106")
            .messages(message_vec.clone())
            .functions([ChatCompletionFunctionsArgs::default()
                .name("execute")
                .description("Executes code on the user's machine, **in the users local environment**, and returns the output")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "language": {
                            "type": "string",
                            "description": "The programming language (required parameter to the `execute` function)",
                            "enum": [
                                "python",
                                "R",
                                "shell",
                                "applescript",
                                "javascript",
                                "html",
                                "powershell",
                            ],
                        },
                        "code": {"type": "string", "description": "The code to execute (required)"},
                    },
                    "required": ["language", "code"],
                }))
                .build()?])
            .function_call("auto")
            .temperature(0.0)
            .build()?;

        let mut stream = client.chat().create_stream(request).await?;

        let mut fn_name = String::new();
        let mut fn_args = String::new();
        let mut fn_contents = String::new();

        let mut lock = stdout().lock();
        while let Some(result) = stream.next().await {
            match result {
                Ok(ref response) => {
                    for chat_choice in &response.choices {
                        if let Some(fn_call) = &chat_choice.delta.function_call {
                            // write!(lock, "function_call: {:?}", fn_call).unwrap();
                            if let Some(name) = &fn_call.name {
                                fn_name = name.clone();
                            }
                            if let Some(args) = &fn_call.arguments {
                                fn_args.push_str(args);
                            }
                        }
                        if let Some(finish_reason) = &chat_choice.finish_reason {
                            if matches!(finish_reason, FinishReason::FunctionCall) {

                                // for display purposes
                                // let result: ChatCompletionRespondAssistantMessage = ChatCompletionRespondAssistantMessageArgs::default()
                                //     .language(&fn_name)
                                //     .code(&fn_args)
                                //     .build()?
                                //     .into();
                                // println!("result is\n {:?}", function_call_res.clone());

                                // Construct message object from openai function_call message for chat completion
                                let assistant_msg =
                                    ChatCompletionRequestAssistantMessageArgs::default()
                                        .content(fn_contents.clone())
                                        .function_call(FunctionCall {
                                            name: fn_name.clone(),
                                            arguments: fn_args.clone(),
                                        })
                                        .build()?;
                                message_vec.push(assistant_msg.into());

                                // Parse function call arguments and get language
                                let function_call_res: Value = serde_json::from_str(&fn_args)?;

                                if let Some(language) = function_call_res.get("language") {
                                    if let Some(language) = language.as_str() {
                                        // execute the code and get response.
                                        let output;
                                        match language {
                                            "python" => {
                                                println!("Found Python code!");

                                                if let Some(code) = function_call_res.get("code") {
                                                    if let Some(code) = code.as_str() {
                                                        // Execute the function call and get the answer message
                                                        output = python_interpreter(code);

                                                        match output {
                                                            Ok(output_msg) => {
                                                                println!(
                                                                    "stdout String: {}",
                                                                    output_msg
                                                                );
                                                                let function_msg: ChatCompletionRequestFunctionMessage = ChatCompletionRequestFunctionMessageArgs::default()
                                                                    .name("execute")
                                                                    .content(output_msg)
                                                                    .build()?;
                                                                // Add function message to history
                                                                message_vec
                                                                    .push(function_msg.into());
                                                            }
                                                            Err(err) => {

                                                                // 发送结果到llm 寻求下一步的解决方案
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            "shell" => {
                                                println!("Found a script for shell!")
                                            }
                                            _ => println!("No match found"),
                                        }

                                        println!(
                                            "Execute the function call and get the answer message"
                                        );
                                    }
                                }
                                // call_fn(&client, &fn_name, &fn_args).await?;
                            }
                        } else if let Some(content) = &chat_choice.delta.content {
                            // 直接返回消息
                            write!(lock, "{}", content).unwrap();
                            fn_contents.push_str(content);
                        }
                    }
                }
                Err(err) => {
                    writeln!(lock, "error: {err}").unwrap();
                }
            }
            stdout().flush()?;
        }
        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);
    }

    Ok(())
}

fn python_interpreter(code: &str) -> Result<String, Box<dyn Error>> {

    // For Python code highlighting.
    // Load these once at the start of your program
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

    println!("=================================================");
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
