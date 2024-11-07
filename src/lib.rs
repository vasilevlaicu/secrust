pub mod cfg_builder;
pub mod dot_generator;
pub mod path_finder;
pub mod substitution;
pub mod verifier;

pub use cfg_builder::*;
pub use dot_generator::*;
pub use path_finder::*;
pub use substitution::*;
pub use verifier::*;

use std::path::{PathBuf, Path};
use syn::{visit::Visit};

use std::fs::{self, File};
use std::io::Write;


// Exporting macros for users
#[macro_export] 
macro_rules! pre {
    ($($t:tt)*) => {{}};
}

#[macro_export]
macro_rules! post {
    ($($t:tt)*) => {{}};
}

#[macro_export]
macro_rules! invariant {
    ($($t:tt)*) => {{}};
}

pub fn run_verification(file_path: &PathBuf, generate_dot: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("file path: {:?}", file_path);
    let content = std::fs::read_to_string(&file_path)?;
    println!("File content (first 100 characters):\n{}", &content[..content.len().min(100)]);

    // parse file and build ast
    let ast = syn::parse_file(&content)?;
    println!("AST successfully parsed for file {:?}", file_path);

    // visit ast
    let mut builder = CfgBuilder::new();
    builder.visit_file(&ast);

    // post process cfg
    builder.post_process();

    let simple_paths = builder.generate_simple_paths();

    let updated_postconditions = builder.apply_substitution(&simple_paths);
    for (i, postcondition) in updated_postconditions.iter().enumerate() {
        println!("---------");
        println!("Updated Postcondition for Path {}: {}", i + 1, postcondition);
        verifier::verify_conditions_for_paths(postcondition);
        println!("Verification completed for {:?}", postcondition);
        println!("---------");
        println!("");
    }

    if generate_dot {
        // Save the DOT file and simple paths in the directory named after the input file
        let output_base_path = Path::new("src/graphs");
        let file_stem = file_path.file_stem().unwrap(); // Get the file name without extension
        let output_dir = output_base_path.join(file_stem); // Create directory path as "src/graphs/filename"

        // Generate the DOT format for the entire CFG
        let dot_format = builder.to_dot();

        // Save all simple paths inside the output directory
        builder.write_paths_to_dot_files(simple_paths, &output_dir);

        // Save the main DOT file in the same directory
        let dot_file_path = output_dir.join(format!("{}.dot", file_stem.to_string_lossy()));
        let mut dot_file = File::create(&dot_file_path).expect("Unable to create DOT file");
        dot_file.write_all(dot_format.as_bytes()).expect("Unable to write to DOT file");

        println!("DOT graph saved as: {:?}", dot_file_path);
    }

    Ok(())
}