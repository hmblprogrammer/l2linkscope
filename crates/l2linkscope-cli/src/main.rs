#![deny(unsafe_code)]

use std::env;
use std::process::ExitCode;

const HELP: &str = "\
L2LinkScope network discovery utility

Usage:
  l2linkscope --help
  l2linkscope --version

Discovery commands have not yet been implemented.
";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let first = args.next();
    let has_extra = args.next().is_some();

    if has_extra {
        eprintln!("error: unexpected extra argument");
        return ExitCode::from(2);
    }

    match first.as_deref() {
        None | Some("--help" | "-h") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("l2linkscope {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(command) => {
            eprintln!("error: '{command}' is not implemented in this scaffold");
            eprintln!("Run 'l2linkscope --help' for available placeholder commands.");
            ExitCode::from(2)
        }
    }
}
