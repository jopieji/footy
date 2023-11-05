use std::env;
use std::process;

use log::Level;

use footy::Command;



fn main() {
    println!("\nGlobal Football CLI\n============================");

    simple_logger::init_with_level(Level::Info).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();

    let command = Command::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let future = footy::run(command);

    rt.block_on(future);
}
