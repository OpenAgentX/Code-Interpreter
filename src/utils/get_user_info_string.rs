use std::env;

pub fn get_user_info_string() -> String {
    let username = whoami::username();
    let binding = std::env::current_dir().unwrap();
    let current_working_directory = binding.to_string_lossy();
    let operating_system = os_info::get().os_type().to_string();
    let default_shell = env::var("SHELL").unwrap_or_else(|_| String::from(""));

    format!(
        "[User Info]\nName: {}\nCWD: {}\nSHELL: {}\nOS: {}",
        username, current_working_directory, default_shell, operating_system
    )
}

