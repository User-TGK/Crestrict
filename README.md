# NWI-IMC029 Master Thesis SWS

This project is submitted as a master thesis for the Software Science (SWS) track at Radboud University Nijmegen (NL).
The entrypoint for this repository is the thesis `mscthesis.pdf`.
The slides of the presentation (from May 7th) are available as the `presentation.pdf`.

---

## Using the interpreter

### Prerequisites
The interpreter has been developed under Ubuntu 22.04 and compiles with (at least) `rustc 1.77.2` for the `stable-x86_64-unknown-linux-gnu` target.
We expect the code to compile on other platforms with a recent Rust compiler[^1] as well.

### Interpreting a Crestrict program

The interpreter only accepts programs in the Crestrict language (a simplified C-like language). Building and running the interpreter is as simple as navigating to the interpreter directory and invoking cargo:
```sh
cd interp
cargo build # Not needed if you invoke the cargo run command below,
            # but a compiled binary is required for running the test suite
cargo run -- --source-file <SOURCE_FILE>
```

<`SOURCE_FILE`> should be replaced with the path to the Crestrict source file to be interpreted.

## Evaluation

### Prerequisites
Depending on whether you want to use the automation script to run all test programs consecutively, you need to install some additional software.
First, we use the compilers `gcc` (tested with versions 11.4 and 13.2) and `clang` (tested with version 14.0.0) to compile the test programs against (for reference output).
Secondly, we use `make` (tested with version 4.3) and `python` (tested with version 3.10.12) for auxiliary scripts to compile and run all test programs.

### Running the test suite 
With the additional software installed, the automation script can be invoked with the commands below.
This will compile all source files with gcc and clang (1) and then run all test programs (the gcc and clang binaries) as well as the Crestrict interpreter (2).
For the well-defined programs (`examples/DB`), the script checks whether the results of a program match the expected results.
For the undefined programs (`examples/UB`), the script checks whether the interpreter correctly reports that the program has undefined behavior.
It also notes whether the compiled binaries by gcc and clang give different results.

```
cd examples
make
python auto-exec.py
```

## Structure

* `examples` contains the programs of the test suite (thesis section 7.2)
    * `DB` contains the programs which we consider to be well-defined
    * `UB` contains the programs which we consider to have undefined behavior
    * `Makefile` is an auxiliary build script used to compile all test programs into (optimized) binaries with both GCC and Clang 
    * `auto-exec.py` is an auxiliary script to invoke GCC, Clang and the Crestrict interpreter on all programs of the test suite
* `interp` contains the interpreter source code (written in Rust)
    * `src/ast.rs` contains the Crestrict syntax (thesis section 5.1)
    * `src/evaluator.rs` contains the operational semantics and some of the restrict checks (thesis sections 5.2 and 5.4)
    * `src/memory.rs` contains the memory operations (thesis section 5.3)
    * `src/restrict.rs` contains some of the restrict operations (thesis section 5.4)
    * `src/transpiler.rs` contains the elaborator (from a C11 AST to a Crestrict AST) (thesis section 6.1)
    * `src/type_checker.rs` contains the type checker of the Crestrict AST (thesis section 6.1)  
* `presentation` contains the latex sources of presentation.pdf
* `thesis` contains the latex sources of mscthesis.pdf

[^1]: Follow the instructions at https://www.rust-lang.org/tools/install for your platform