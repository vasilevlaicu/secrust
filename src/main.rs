use std::path::PathBuf;
use std::process::exit;
use clap::{Arg, Command};
use secrust::run_verification;

fn main() {
    // print args
    let raw_args: Vec<String> = std::env::args().collect();
    println!("Raw arguments: {:?}", raw_args);

    let adjusted_args: Vec<String> = raw_args
        .iter()
        .skip_while(|arg| !arg.contains("secrust-verify")) 
        .skip(1) 
        .map(|arg| arg.clone())
        .collect();
    
    // parsing args using clap
    let matches = Command::new("Secrust Verification Tool")
        .version("1.0")
        .author("Vasile")
        .about("Verifies Rust code using Secrust analysis and optionally generates a DOT graph")
        .arg(
            Arg::new("file")
                .help("The input file to verify")
                .required(true)
                .index(1),  // positional file arg
        )
        .arg(
            Arg::new("dot")
                .long("dot")
                .help("Generate a DOT graph representation of the CFG")
                .action(clap::ArgAction::SetTrue),  // check the flag is here
        )
        .try_get_matches_from(&adjusted_args)
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
            exit(1);
        });

    // handle file argument
    let file = matches.get_one::<String>("file").unwrap();
    let file_path = PathBuf::from(file);

    // check if the dot flag was provided
    let generate_dot = *matches.get_one::<bool>("dot").unwrap_or(&false);

    println!("Running Secrust verification on file: {:?}", file_path);
    println!("Generate DOT graph: {}", generate_dot);

    // run verification function with the provided file and generate_dot flag
    if let Err(e) = run_verification(&file_path, generate_dot) {
        eprintln!("Verification failed: {}", e);
        exit(1);
    } else {
        println!("Verification completed successfully.");
    }
}
