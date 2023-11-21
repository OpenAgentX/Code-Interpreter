use std::collections::HashMap;
use std::error::Error;
use std::io::{stdout, Write};

use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionFunctionsArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, FinishReason, Role,
};
use async_openai::Client;

use async_openai::config::OpenAIConfig;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use serde_json::json;
use derive_builder::Builder;

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


    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo-0613")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(&instructions)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                // .content("What operating system are we on?")
                .content("What's the weather like in beijing?")
                .build()?
                .into()])
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

    let mut lock = stdout().lock();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices {
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
                            // println!(
                            //     "function_call language:\n {} \nfn_args: {}",
                            //     fn_name, fn_args
                            // );

                            let result: ChatCompletionRespondAssistantMessage = ChatCompletionRespondAssistantMessageArgs::default()
                                .language(&fn_name)
                                .code(&fn_args)
                                .build()?
                                .into();

                            println!("result is {:?}", result);


                            // 执行代码

                            // call_fn(&client, &fn_name, &fn_args).await?;
                        }
                    } else if let Some(content) = &chat_choice.delta.content {
                        // 直接返回消息
                        write!(lock, "{}", content).unwrap();
                    }
                }
            }
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        stdout().flush()?;
    }

    Ok(())
}