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

## Tutorial: Verifying `sum_first_n`

The following example demonstrates how to verify a simple Rust function using `secrust`.

### Example Code
Save the following code as `src/main.rs`:
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

## Running Example: Verifying `sum_first_n`

To showcase how Secrust works, let’s walk through the verification process for a simple Rust function that calculates the sum of the first \( n \) integers.

### Annotated Rust Code

```rust
use secrust::{pre, post, invariant};

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
    let n = 0;
    let sum = sum_first_n(n);
    println!("Sum is: {}", sum);
}
```

- **`pre!`** and **`post!`** annotate the function with input and output conditions.
- **`invariant!`** provides the loop invariant, which must hold before and after each iteration of `while`.

---

### 1. Generating a CFG and Basic Paths

When you run Secrust with the `--dot` flag:
```bash
cargo secrust-verify src/main.rs --dot
```
Secrust parses the AST to build a **Control Flow Graph (CFG)**, placing **cut points** at each annotation or branch. Then it extracts **basic paths**, each corresponding to a distinct route through the function.


<summary>Example Overall CFG</summary>

% image 1 here

*Figure: CFG that highlights all basic paths extracted by Secrust.*


Among the paths generated, let’s focus on **Path 3**, the route taken when the `while` condition `i <= n` is **true**.

<summary>Sample Basic Path (Path 3)</summary>

% image 2 here

*Figure: Visualization of the DOT representation for basic path 3.*

---

### 2. Deriving the Weakest Precondition

Secrust verifies each basic path by **traversing it backward** from the postcondition, repeatedly applying WP rules for:
- **Assignment**: Substitute updated variables in the postcondition.
- **Assume**: Add `condition => ...` logic.
- **Assert**: Strengthen the precondition with `condition && ...`.
- **Composition**: Combine consecutive statements via `wp(S1, wp(S2, Q))`.



#### Example: Backward Substitution
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
   `i <= n => ((i + 1) <= n + 1 AND (sum + i) == ((i + 1) - 1) * (i + 1) / 2)`

4. **Finally**, we link it back to the loop’s *starting* invariant (the path’s precondition).

5. Hence, the **final logical implication** for Path 3 is:
`(i <= n + 1 AND sum == (i - 1) * i / 2) => (i <= n => ((i + 1) <= n + 1 AND (sum + i) == ((i + 1) - 1) * (i + 1) / 2))`
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