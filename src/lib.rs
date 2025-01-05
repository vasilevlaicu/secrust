pub mod cfg_builder;
pub mod verifier;
pub mod wp_calculus;

pub use cfg_builder::*;
pub use verifier::*;

use std::path::{Path, PathBuf};

use std::fs::File;
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

#[macro_export]
macro_rules! build_cfg {
    ($($t:tt)*) => {{}};
}

#[macro_export]
macro_rules! old {
    ($($t:tt)*) => {{}};
}

pub fn run_verification(
    file_path: &PathBuf,
    generate_dot: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("file path: {:?}", file_path);
    let content = std::fs::read_to_string(&file_path)?;

    // parse file and build ast
    let ast = syn::parse_file(&content)?;
    println!("AST successfully parsed for file {:?}", file_path);

    // visit ast
    let mut builder = CfgBuilder::new();

    builder.build_cfg(&ast);

    let basic_paths = builder.generate_basic_paths();

    let final_implication = builder.apply_wp_calculus(&basic_paths);
    for (i, implication) in final_implication.iter().enumerate() {
        println!("---------");
        println!("Final implication for Path {}: {}", i + 1, implication);
        verifier::verify_str_implication(implication);
        println!("Verification completed for {:?}", implication);
        println!("---------");
        println!("");
    }

    if generate_dot {
        // Save the DOT file and basic paths in the directory named after the input file
        let output_base_path = Path::new("src/graphs");
        let file_stem = file_path.file_stem().unwrap(); // Get the file name without extension
        let output_dir = output_base_path.join(file_stem); // Create directory path as "src/graphs/filename"

        // Generate the DOT format for the entire CFG
        let dot_format = builder.to_dot();

        // Save all basic paths inside the output directory
        builder.write_paths_to_dot_files(basic_paths, &output_dir);

        // Save the main DOT file in the same directory
        let dot_file_path = output_dir.join(format!("{}.dot", file_stem.to_string_lossy()));
        let mut dot_file = File::create(&dot_file_path).expect("Unable to create DOT file");
        dot_file
            .write_all(dot_format.as_bytes())
            .expect("Unable to write to DOT file");

        println!("DOT graph saved as: {:?}", dot_file_path);
    }

    Ok(())
}
