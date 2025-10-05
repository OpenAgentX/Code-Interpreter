
use std::error::Error;

// use base64::decode;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use async_stream::try_stream;
use futures_core::stream::Stream;

use tracing::{debug, info, warn};

use jupyter_client::commands::Command as JupyterCommand;
use jupyter_client::responses::{ExecutionState, IoPubResponse, Response, ShellResponse, Status};
use jupyter_client::Client;
use std::collections::HashMap;


use viuer::Config as ViuerConfig;


#[derive(Debug)]
struct Message {
    raw_message: IoPubResponse,
    data_type: String,
    content: String,
}



/// send code to jupyter kernel
async fn send_code_to_jupyter(code: &str) {
    let client = Client::existing().expect("creating jupyter connection");
    // send code to jupyter kernel
    let command = JupyterCommand::Execute {
        code: code.into(),
        silent: false,
        store_history: true,
        user_expressions: HashMap::new(),
        allow_stdin: true,
        stop_on_error: false,
    };
    let response = client.send_shell_command(command).expect("sending command");
    if let &Response::Shell(ShellResponse::Execute { ref content, .. }) = &response {
        match content.status {
            Status::Ok | Status::Abort => {
                // debug!("Response: {:#?}", response)
            }
            Status::Error => {
                eprintln!("Error: {}", content.evalue.as_ref().unwrap());
                for line in content.traceback.as_ref().unwrap() {
                    eprintln!("{}", line);
                }
            }
        }
    } else {
        panic!("unexpected response type");
    }
    info!("finished:\n{:#?}", response);
}

/// an asynchronous function to get data
async fn get_data(tx: mpsc::Sender<IoPubResponse>) {
    let client = Client::existing().expect("creating jupyter connection");
    let receiver = client.iopub_subscribe().unwrap();

    let mut received_message_ending = false;

    let mut io_pub_responses: Vec<IoPubResponse> = Vec::new();
    let mut execution_state = ExecutionState::Idle;
    for msg in &receiver {
        match msg {
            Response::IoPub(data) => {
                // debug!("----------------- 订阅线程\n {:#?}", data);
                // Send the data to the main thread
                match &data {
                    IoPubResponse::Status {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        // todo!()
                        // status = parent_header.msg_type.clone();
                        // debug!("【Status】: \n {:#?}", io_pub_responses);
                        if execution_state == ExecutionState::Busy
                            && content.execution_state == ExecutionState::Idle
                        {
                            // debug!("finished \n {:#?}", &data);
                            received_message_ending = true;
                            // break;
                        }
                        execution_state = content.execution_state;

                        // tx.send(data);
                    }
                    IoPubResponse::ExecuteInput {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        // debug!("finished ExecuteInput\n {:#?}", data);
                        io_pub_responses.push(data);
                        // debug!(" ExecuteInput: \n {:#?}", io_pub_responses);
                        // shared_state.lock().await.push(data.clone());
                        // shared_state.lock().await.push("value".into());
                    }
                    IoPubResponse::Stream {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        todo!()
                    }
                    IoPubResponse::ExecuteResult {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        // todo!()
                        io_pub_responses.push(data);
                        // debug!(" ExecuteResult: \n {:#?}", io_pub_responses);
                        // tx.send(data.clone());
                        // tx.send(IoPubResponse::Data(format!("Data {}", i)))
                        // .await
                        // .unwrap();
                    }
                    IoPubResponse::Error {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        io_pub_responses.push(data);
                        // debug!(" Error: \n {:#?}", io_pub_responses);
                        // tx.send(data.clone());
                    }
                    IoPubResponse::ClearOutput {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        io_pub_responses.push(data);
                        // debug!(" ClearOutput: \n {:#?}", io_pub_responses);
                        // tx.send(data.clone());
                    }
                    IoPubResponse::DisplayData {
                        header,
                        parent_header,
                        metadata,
                        content,
                    } => {
                        io_pub_responses.push(data);
                        // debug!(" DisplayData: \n {:#?}", io_pub_responses);
                        // tx.send(data.clone());
                    }
                }
            }
            Response::Shell(shell) => {
                // debug!("finished\n {:#?}", shell);
            }
        }
        if received_message_ending {
            // debug!("finished");
            break;
        } else {
            // debug!("deal with one received message");
        }
    }

    for response in io_pub_responses {
        tx.send(response.clone()).await.unwrap();
    }
}


/// 
/// # Data Example
/// ```
/// let code_example1 = String::from(
///     "import pandas as pd\n\
///     import numpy as np\n\
///     df = pd.DataFrame(np.random.rand(10, 5))\n\
///     # For HTML output\n\
///     display(df)\n\
///     # For image output using matplotlib\n\
///     import matplotlib.pyplot as plt\n\
///     plt.figure()\n\
///     plt.plot(df)\n\
///     plt.savefig('plot.png')  # Save the plot as a .png file\n\
///     plt.show()"
/// );
/// let code_example2 = String::from("import pandas as pd\nimport numpy as np\nf = pd.DataFrame(np.random.rand(10, 5))\ndisplay(df)"); 
/// let code_example3 = String::from("import time\nimport pandas as pd\nimport numpy as np\ndf = pd.DataFrame(np.random.rand(10, 5))\ndisplay(df)\ntime.sleep(2)");
/// ```
pub async fn python_vision_interpreter(code: &str) -> Result<String, Box<dyn Error>> {
    // For Python code highlighting.
    // Load these once at the start of your program
    debug!("\n =================================================");
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

    debug!("\n\n =================================================");


    // Create a shared state
    let shared_state = Arc::new(Mutex::new(Vec::new()));

    // Clone a reference to the shared state for the asynchronous task
    let shared_state_for_async = Arc::clone(&shared_state);

    // Create a channel for communication between the main thread and the asynchronous task
    let (tx, mut rx) = mpsc::channel::<IoPubResponse>(20);

    // Spawn the asynchronous task
    tokio::spawn(async move {
        get_data(tx).await;
    });

    tokio::spawn(async move {
        send_code_to_jupyter(&hl_code).await;
    });

    
    // Main thread logic to consume the received data
    while let Some(response) = rx.recv().await {
        // Lock the shared state and add the response
        let mut shared_state = shared_state_for_async.lock().unwrap();
        // println!("{:#?}", response);
        match &response {
            IoPubResponse::DisplayData { header, parent_header, metadata, content } => {
                
                let msg = content.data.get("text/plain").unwrap();

                // if let Some(hl_code) = content.data.get("text/html") {
                //     println!("\n{}", msg);
                //     let syntax = ps.find_syntax_by_extension("html").unwrap();
                //     let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);
                //     // let hl_code = "\n".to_string() + code;
                //     for line in LinesWithEndings::from(&hl_code) {
                //         let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                //         let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                //         print!("{}", escaped);
                //     }
                // }

                if let Some(base64_image_str) = content.data.get("image/png") {

                    // Decode Base64 to bytes
                    let decoded_bytes = BASE64.decode(base64_image_str)?;
                    // Create a DynamicImage from the decoded bytes
                    let dynamic_image = image::load_from_memory(&decoded_bytes)?;

                    // Now you can work with the DynamicImage
                    // For example, save it to a file
                    // dynamic_image.save("decoded_image.png").unwrap();
                    // println!("{} x {}", dynamic_image.width(), dynamic_image.height());
                    let mut width = dynamic_image.width();
                    let mut height = dynamic_image.height();
                    
                    if height > 50 {
                        width = 50;
                        height = ((width as f32 / dynamic_image.width() as f32)  * height as f32) as u32;
                    }

                    let cfg = &ViuerConfig{
                        transparent: false,
                        absolute_offset: false,
                        x: 0,
                        y: 0,
                        restore_cursor: false,
                        width: Some(width),
                        height: Some(height),
                        truecolor: false,
                        use_kitty: true,
                        use_iterm: true,
                        premultiplied_alpha: false,
                    };
                    
                    viuer::print(&dynamic_image, cfg)?;

                }
                // let msg = content.data.get("text/plain").unwrap();
            },
            _ => {
                
            }
        }
        shared_state.push(response);
    }

    // Print the final shared state
    let final_state: std::sync::MutexGuard<'_, Vec<IoPubResponse>> = shared_state.lock().unwrap();
    // println!("Final Shared State: {:#?}", *final_state);
    let mut output_str: String = String::new();

    Ok(output_str)
}


fn python_interpreter(code: &str)
    -> impl Stream<Item = Result<String,  Box<dyn Error>>>
{
    try_stream! {
        // let mut listener = TcpListener::bind(addr).await?;
        let client = Client::existing().expect("creating jupyter connection");
        let receiver = client.iopub_subscribe().unwrap();
    
        let mut received_message_ending = false;
    
        let mut io_pub_responses: Vec<IoPubResponse> = Vec::new();
        let mut execution_state = ExecutionState::Idle;
        for msg in &receiver {
        // loop {
            // let (stream, addr) = listener.accept().await?;
            // println!("received on {:?}", addr);
            yield "msg".to_string();
        }
        // }
    }
}



#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
  use super::*;

    #[test]
    fn test_python_vision_interpreter() {
        let code = String::from(
            "import pandas as pd\n\
            import numpy as np\n\
            df = pd.DataFrame(np.random.rand(10, 5))\n\
            # For HTML output\n\
            display(df)\n\
            # For image output using matplotlib\n\
            import matplotlib.pyplot as plt\n\
            plt.figure()\n\
            plt.plot(df)\n\
            plt.savefig('plot.png')  # Save the plot as a .png file\n\
            plt.show()"
        );
        // let code = String::new();
        let _ = python_vision_interpreter(&code);
    }
}

