# secured-rust
Adding annotations to Rust.

# Run
## Install secrust

### Add Z3 on MacOS
1. Install Z3 using Homebrew:
   ```bash
   brew install z3
   brew info z3
   ```
2. Set environment variables to point to the Z3 paths (modify as needed for your setup):
   ```bash
   export Z3_SYS_Z3_HEADER=/opt/homebrew/Cellar/z3/4.13.3/include/z3.h
   export Z3_SYS_Z3_LIB_DIR=/opt/homebrew/Cellar/z3/4.13.3/lib
   export LIBRARY_PATH=/opt/homebrew/Cellar/z3/4.13.3/lib:$LIBRARY_PATH
   export LD_LIBRARY_PATH=/opt/homebrew/Cellar/z3/4.13.3/lib:$LD_LIBRARY_PATH
   ```

3. Install `secrust` using Cargo:
   ```bash
   cargo install --path ../secrust
   ```

### Add Z3 on Windows
1. **Download Z3:**
   - Get the latest precompiled Z3 binary for Windows from the [Z3 GitHub releases page](https://github.com/Z3Prover/z3/releases).
   - Extract the ZIP file to a directory, e.g., `C:\z3`.
     - Ensure the extracted directory contains:
       - `bin` folder: Includes `z3.exe` and `.dll` files.
       - `include` folder: Contains header files like `z3.h`.

2. **Set Environment Variables:**
   - Add the following to your system environment variables:
     ```cmd
     Z3_SYS_Z3_HEADER=C:\z3\include\z3.h
     Z3_SYS_Z3_LIB_DIR=C:\z3\bin
     LIBRARY_PATH=C:\z3\bin
     LD_LIBRARY_PATH=C:\z3\bin
     ```
   - Add `C:\z3\bin` to your `PATH`.

3. **Ensure You Have GCC Installed:**
   - If you don't have GCC installed, you can install it using MSYS2:
     1. Download and install [MSYS2](https://www.msys2.org/).
     2. Open the MSYS2 terminal and run:
        ```bash
        pacman -Syu
        pacman -S mingw-w64-x86_64-toolchain
        ```
     3. Add the following to your system `PATH`:
        ```cmd
        C:\msys64\mingw64\bin
        ```

4. **Ensure You Have LLVM Installed:**
   - Install LLVM from [LLVM's official website](https://releases.llvm.org/download.html).
   - Add the `bin` directory of LLVM (e.g., `C:\LLVM\bin`) to your `PATH`.
   - Set the `LIBCLANG_PATH` environment variable:
     ```cmd
     LIBCLANG_PATH=C:\LLVM\bin
     ```

5. **Install `secrust` using Cargo:**
   ```cmd
   cargo install --path ..\secrust
   ```

### Verify Installation
Run the following command to ensure `secrust` is installed correctly:
```bash
cargo secrust-verify --help
```

## Usage

### Run without generating DOT file CFG
Analyze a file without generating Control Flow Graphs:
```bash
cargo secrust-verify main.rs
```

### Run generating DOT file CFG
Analyze a file and generate DOT files for the Control Flow Graph:
```bash
cargo secrust-verify src/main.rs --dot
```
DOT files are created in the `src/graphs/filename` directory for the specified file (e.g., `src/main.rs`).