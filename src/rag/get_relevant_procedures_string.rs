use std::collections::HashMap;
use async_openai::types::ChatCompletionRequestUserMessage;
use tracing::debug;
use anyhow::{Ok, Result};
use reqwest;
use serde::{Deserialize,Serialize};

#[derive(Debug, Deserialize,Serialize)]
struct RelevantProcedures {
    procedures: Vec<String>
}

/// - Open Procedures is an open-source database of tiny, up-to-date coding tutorials.
/// - We can query it semantically and append relevant tutorials/procedures to our system message: 
pub async fn get_relevant_procedures_string(message: &ChatCompletionRequestUserMessage) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("query", vec![message]);

    let client = reqwest::Client::new();
    let url = "https://open-procedures.replit.app/search/";
    let res = client.post(url).json(&map).send().await?.text().await?;

    let relevant_procedures: RelevantProcedures = serde_json::from_str(&res).unwrap();

    let relevant_procedures_str = format!(
        "[Recommended Procedures]\n{}\nIn your plan, include steps and, for relevant deprecation notices, **EXACT CODE SNIPPETS** -- these notices will VANISH once you execute your first line of code, so WRITE THEM DOWN NOW if you need them.",
        relevant_procedures.procedures.join("\n---\n")
    );

    // debug!("================================================");
    debug!("relevant_procedures: \n{}", relevant_procedures_str);
    // println!("================================================");

    Ok(relevant_procedures_str)
}
