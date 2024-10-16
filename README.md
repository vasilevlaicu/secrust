# secured-rust
Adding annotations to rust.

# Run
## Install secrust
cargo uninstall secrust
cargo install --path ../secrust

### Run without generating dot file CFG
cargo secrust-verify main.rs 

### Run generating dot file CFG
cargo secrust-verify src/factorial.rs --dot

dot files are created in the src/graphs/filename directory for the filename.rs in argument.