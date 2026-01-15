use std::env;

use emolang::repl;

fn main() {
    println!("Hello {}, Welcome to EMO programming language!", get_user_name());
    println!("Feel free to start coding ⌨️");
    repl::start();
}

fn get_user_name() -> String {
    for envname in ["USER", "USERNAME", "LOGNAME"] {
        if let Ok(username) = env::var(envname) && !username.is_empty() {
            return username;
        }
    }
    String::from("🧑🏻‍💻")
}
