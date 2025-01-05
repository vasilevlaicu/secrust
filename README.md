# Secrust  
**Secrust** is a Rust crate designed to add **formal verification** to Rust code. By adding lightweight annotations to Rust functions, Secrust enables developers to verify invariants, preconditions, and postconditions directly in their source code.

Secrust leverages Rust's syntax and ecosystem, integrating with the language's tooling to provide an intuitive developer experience. The crate uses the Z3 SMT solver to reason about program correctness and generates Control Flow Graphs (CFGs) to visualize execution paths, making it easier to identify and eliminate logical errors. 

### **Supported Syntax**  
Secrust currently supports simple Rust code:
- **Arithmetic operations**: Verifying computations involving addition, subtraction, multiplication, and division.
- **Conditional statements**: Handling `if`/`else` branches to ensure correctness across all execution paths.
- **Loops**: Reasoning about loop invariants and termination conditions to verify iterative logic.

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

3. Install `secrust` pre-release using Cargo:
   ```bash
   cargo install secrust --version 0.1.0-alpha.1
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
   cargo install secrust
   ```
   Or if you download the repo:
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

You can visualize DOT code online on [edotor.net](https://edotor.net/?engine=dot)

## How it works: Verifying `sum_first_n`

The following example demonstrates how to verify a simple Rust function using `secrust`.

### Example Code
Save the following code as `src/main.rs`, and make sure to annotate it with pre!, invariant! and post! assertions:
```rust
use secrust::{build_cfg, invariant, old, post, pre};

fn sum_first_n(n: i32) -> i32 {
    pre!(n >= 0);
    let mut sum = 0;
    let mut i = 1;
    invariant!(i <= n + 1 && sum == (i - 1) * i / 2);
    while i <= n {
        sum = sum + i;
        i = i + 1;
    }
    post!(sum == n * (n + 1) / 2);
    return sum;
}

fn main() {
    let n = 5;
    let sum = sum_first_n(n);
    println!("Sum is: {}", sum);
}
```

### Run Verification
Run the `secrust` verification on this file:
```bash
cargo secrust-verify src/main.rs --dot
```

### Outputs
1. **Verification Results**: The terminal will display the results of the verification, including logical implications and their validity status.
2. **DOT Graphs**: Control Flow Graphs (CFGs) will be generated in the `src/graphs/main` directory.

For example:
- `main.dot` will contain the CFG for the `main` function.
- `basic_path_0.dot` will contain the graph for the first basic execution path of the annotated `sum_first_n` function.

To generate a DOT format CFG for any method without adding logical annotations, add the ```build_cfg!();``` macro at the start of the method.

### Analyze the DOT Graph
Use tools like `Graphviz` to visualize the DOT files:
```bash
dot -Tpng src/graphs/main/basic_path_0.dot -o basic_path_0.png
```
Or paste the DOT code on an online editor like [edotor.net](https://edotor.net/?engine=dot).
### Expected Behavior
- Verification checks the validity of the derived weakest precondition
- Generated graphs provide a clear view of the control flow and verification conditions.

## How it works: Verifying `sum_first_n`

To showcase how Secrust works, let’s walk through the verification process for a simple Rust function that calculates the sum of the first \( n \) integers.

- **`pre!`** and **`post!`** annotate the function with input and output conditions.
- **`invariant!`** provides the loop invariant, which must hold before and after each iteration of `while`.

---

### 1. Generating a CFG and Basic Paths

When you run Secrust with the `--dot` flag:
```bash
cargo secrust-verify src/main.rs --dot
```
Secrust parses the AST to build a **Control Flow Graph (CFG)**. Then it extracts **basic paths**, each corresponding to a distinct route through the function.

<img src="data:image/webp;base64,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" alt="Example Image" />

*Figure: CFG that was manually highlighted to show all basic paths extracted by Secrust. Secrust will also save each basic path in a separate .DOT file*

Among the paths generated, let’s focus on **Path 3**, the route taken when the `while` condition `i <= n` is **true**.

### 2. Deriving the Weakest Precondition

Secrust analyzes each basic path by **traversing it backward** from the postcondition, repeatedly applying WP rules.

1. **Start** from the loop invariant at the “bottom” of the path:
   `i <= n + 1 AND sum == (i - 1) * i / 2`

2. **Move upward** through assignments like:
   ```rust
   sum = sum + i;
   i = i + 1;
   ```
   which update `sum` to `sum + i` and `i` to `i + 1`. Secrust substitutes these into the invariant, yielding:
   `(i + 1) <= n + 1 AND (sum + i) == ((i + 1) - 1) * (i + 1) / 2`

3. **Encounter the `while i <= n`** (true branch). This adds an assumption:
   `(i <= n) => ((i + 1) <= n + 1 AND (sum + i) == ((i + 1) - 1) * (i + 1) / 2)`

4. **Finally**, we link it back to the loop’s *starting* invariant (the path’s precondition).

5. Hence, the **final logical implication** for Path 3 is:
`(i <= n + 1 AND sum == (i - 1) * i / 2) => (i <= n) => ((i + 1) <= n + 1 AND (sum + i) == ((i + 1) - 1) * (i + 1) / 2))`
---

### 3. Z3 Verification

After deriving this **implication**, Secrust:
1. Builds a **Z3 AST** representing the formula in SMT-LIB syntax.
2. Asserts its **negation** in Z3. If **unsatisfiable**, the original implication holds, thus verifying the path.

Below is a simplified example of the final formula in SMT-LIB:
```smt
(=> (and (<= i (+ n 1)) (= sum (div (* (- i 1) i) 2)))
    (=> (<= i n)
        (and (<= (+ i 1) (+ n 1))
             (= (+ sum i) (div (* (- (+ i 1) 1) (+ i 1)) 2)))))
```
Because the solver reports **unsatisfiable** for its negation, Path 3 is verified. Repeating this process for all basic paths ensures the entire function satisfies its preconditions, invariants, and postconditions.

---

### Summary

1. **Annotated Rust Source** → `sum_first_n` with `pre!`, `post!`, and `invariant!`.
2. **CFG Construction** → Identify cut points (annotations + loop edges).  
3. **Basic Paths** → Distill distinct routes through the function.  
4. **WP Backward Analysis** → Combine assignments, assumes, and asserts to derive a final logical condition.  
5. **Z3 Check** → If all path formulas are valid, the function is verified.

# License  
Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))