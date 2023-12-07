use tracing::warn;

pub fn check_environments() -> bool {
    if let Some(_) = std::env::var("OPENAI_API_KEY").ok() {
        // The environment variable "OPENAI_API_KEY" exists, and its value was successfully obtained.
        // The variable api_key contains the value of the environment variable.
        // You can perform some operations using api_key here.
        true
    } else {
        // The environment variable "OPENAI_API_KEY" does not exist or retrieving its value failed.
        // You can handle the case of a missing environment variable here.

        warn!("
        There might be an issue with your API key(s).

        To reset your API key (we'll use OPENAI_API_KEY for this example, but you may need to reset your ANTHROPIC_API_KEY, HUGGINGFACE_API_KEY, etc):
            Mac/Linux: 'export OPENAI_API_KEY=your-key-here',
            Windows: 'setx OPENAI_API_KEY your-key-here' then restart terminal.
        ");
        false
    }
}
