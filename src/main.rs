use std::{env, process};

const HELP_COMMANDS: [(&str, &str, u8, bool); 7] = [
    ("help", "this help", 0, true),
    ("version", "program version", 0, true),
    ("plan", "pipeline plan", 1, false),
    ("validate", "validate cinder.yaml", 5, false),
    ("graph", "dependecy graph", 6, false),
    ("run", "start pipeline", 7, false),
    ("tui", "fullscreen tui", 17, false),
];

fn print_help() {
    println!("cinderci - local CI/CD, which is building itself 🔥\n");
    println!("Commands:");
    for (name, desc, etap, avail) in HELP_COMMANDS {
        if avail {
            println!("   {:<10}    {:<30} ({:<02})", name, desc, etap);
        } else {
            println!("   {:<10} 🔒 {:<30} ({:<02})", name, desc, etap)
        }
    }
}

fn main() {
    let command: Option<String> = env::args().nth(1);

    match command.as_deref() {
        Some("-V" | "version" | "--version") => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        }
        Some("-h" | "help" | "--help") | None => print_help(),
        Some(command) => {
            for (name, _desc, etap, avail) in HELP_COMMANDS {
                if command == name && !avail {
                    eprintln!(
                        "Command {} not available, will be implemented at stage {}",
                        name, etap
                    );
                    print_help();
                    process::exit(2);
                }
            }

            eprintln!(
                "Command {} not available, please use one of the following commands:",
                command
            );
            print_help();
            process::exit(2);
        }
    }
}
