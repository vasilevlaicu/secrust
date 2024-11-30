# secured-rust
Adding annotations to rust.

# Run
## Install secrust
### Add z3 on MacOS
brew install z3
brew info z3
check for the z3 path and point `Z3_SYS_Z3_HEADER` and `Z3_SYS_Z3_LIB_DIR`
for example:
```
export Z3_SYS_Z3_HEADER=/opt/homebrew/Cellar/z3/4.13.3/include/z3.h
export Z3_SYS_Z3_LIB_DIR=/opt/homebrew/Cellar/z3/4.13.3/lib
export LIBRARY_PATH=/opt/homebrew/Cellar/z3/4.13.3/lib:$LIBRARY_PATH
export LD_LIBRARY_PATH=/opt/homebrew/Cellar/z3/4.13.3/lib:$LD_LIBRARY_PATH

```
cargo install --path ../secrust

### Run without generating dot file CFG
cargo secrust-verify main.rs 

### Run generating dot file CFG
cargo secrust-verify src/factorial.rs --dot

dot files are created in the src/graphs/filename directory for the filename.rs in argument.


# TODO
- [ ] handle more than ints (for z3 parser)