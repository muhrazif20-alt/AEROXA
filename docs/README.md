AEROXA

Strict Universal Programming Language

CODE BEYOND LIMITS

AEROXA adalah bahasa pemrograman yang sedang dikembangkan menggunakan Rust, dengan fokus pada strict typing, compile-time validation, ownership-based safety, dan native compilation melalui LLVM.

AEROXA dirancang untuk menjadi fondasi bahasa pemrograman universal yang mengutamakan kontrol, keamanan, dan performa.

«Project Status: Experimental / Under Development»

---

1. Project Overview

AEROXA memiliki compiler sendiri yang dibangun menggunakan Rust dan menggunakan LLVM sebagai backend untuk menghasilkan executable native.

Tujuan pengembangan AEROXA:

- Strict and static type system.
- Compile-time error detection.
- Ownership-based memory safety.
- Native executable generation.
- Modular programming architecture.
- Future cross-platform development.

AEROXA masih berada dalam tahap pengembangan aktif. Fitur dan sintaks dapat berubah seiring perkembangan compiler.

2. Current Features

Berikut adalah kemampuan yang sudah memiliki implementasi dalam compiler saat ini.

Compiler Architecture

- [x] Custom Lexer
- [x] Custom Parser
- [x] Abstract Syntax Tree (AST)
- [x] Semantic Analysis
- [x] Symbol Table
- [x] Ownership Analyzer
- [x] Intermediate Representation (IR)
- [x] AST to IR Lowering
- [x] LLVM IR Generation
- [x] Native Compilation using Clang
- [x] Incremental Build Cache (prototype)
- [x] Cinematic Compiler Progress Display

Language Features

- [x] Variable Declaration
- [x] Mutable Variables
- [x] Immutable Variables by Default
- [x] Typed Variable Declaration
- [x] Integer Arithmetic
- [x] Unary Negative
- [x] Comparison Operators
- [x] Logical Operators
- [x] Conditional Statements
- [x] Else-if Branching
- [x] Else Branching
- [x] Loop Statements
- [x] Integer Output
- [x] String Output
- [x] Module Linking Prototype

Celeront

AEROXA also has an accompanying command-line package manager named Celeront.

Its current command interface includes:

- "init"
- "install"
- "remove"
- "update"
- "search"
- "list"
- "run"

Package and environment management remain under development.

---

3. Syntax Examples

Variable Declaration

let score: Int32 = 85;
let mut counter: Int32 = 0;

"let" declares an immutable variable, while "let mut" declares a mutable variable.

Conditional Statements

AEROXA uses its own conditional keywords.

let score: Int32 = 85;

check score >= 90 {
    print("Excellent");
} orcheck score >= 75 {
    print("Good");
} #else$ {
    print("Try Again");
}

Keywords:

AEROXA| Meaning
"check"| if
"orcheck"| else if
"#else$"| else

Loop

let mut counter: Int32 = 0;

loopcheck counter < 5 {
    print(counter);
    counter = counter + 1;
}

Module Linking

AEROXA currently has a basic module-loading prototype.

Example:

link::math;

let value: Int32 = 10;
print(value);

Example module file:

"modules/math.ex"

let number: Int32 = 100;
print(number);

The compiler can discover linked module files, parse their contents, detect circular dependencies, and integrate their AST statements into the main program.

Note: Namespace-based function calls such as "math::calculate()" are not implemented yet. Current AST integration is a preliminary implementation.

---

4. Compilation Pipeline

The current compiler pipeline:

AEROXA SOURCE (.ex)
        |
        v
   LEXICAL ANALYSIS
        |
        v
    SYNTAX ANALYSIS
        |
        v
   MODULE RESOLUTION
        |
        v
   SEMANTIC ANALYSIS
        |
        v
  OWNERSHIP ANALYSIS
        |
        v
    AST TO IR
        |
        v
  LLVM IR GENERATION
        |
        v
 NATIVE COMPILATION
        |
        v
   EXECUTABLE (.exe)

The compiler uses Clang to compile generated LLVM IR into a native executable.

5. Benchmark

AEROXA has an experimental cross-language benchmark system comparing compilation times against Rust and C++.

Initial median compilation results:

Benchmark| AEROXA| Rust| C++
Algorithms| 536.14 ms| 448.64 ms| 513.62 ms
Arithmetic| 532.37 ms| 428.84 ms| 535.38 ms
Branching| 530.49 ms| 439.99 ms| 522.62 ms

These measurements represent the benchmark pipeline, including native compilation. They are not runtime performance measurements.

Results are preliminary and depend on the machine, compiler versions, optimization settings, and benchmark implementation.

6. Technology Stack

Technology| Purpose
Rust| Compiler implementation
LLVM IR| Intermediate backend representation
Clang| Native compilation
PowerShell| Benchmark automation
C++| Benchmark comparison
TOML| Cargo project configuration
JSON| Structured data and configuration

7. Project Structure

AeroxA/
├── compiler/
│   ├── src/
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── ast.rs
│   │   ├── semantic.rs
│   │   ├── symbol_table.rs
│   │   ├── ir.rs
│   │   ├── ir_lowering.rs
│   │   ├── backend/
│   │   ├── ownership/
│   │   ├── module_resolver.rs
│   │   └── main.rs
│   └── Cargo.toml
├── celeront/
├── examples/
├── modules/
├── benchmark/
└── trash/

8. Current Development Roadmap

Planned development directions:

- Improve compiler performance.
- Expand the type system.
- Develop ownership and borrowing rules.
- Implement function declarations and calls.
- Develop namespace isolation for modules.
- Improve module and dependency management.
- Develop package and environment configuration.
- Explore cross-platform compilation.
- Explore future AEROXA Studio and integrated development tools.

These are development goals, not currently completed features.

9. Build and Run

Requirements:

- Rust toolchain
- Cargo
- Clang with LLVM IR support

Build the compiler:

cd compiler
cargo build --release

Run an AEROXA source file:

D:\cargo-target\release\Aeroxa.exe .\examples\arithmetic.ex

Compile without executing:

D:\cargo-target\release\Aeroxa.exe .\examples\arithmetic.ex --compile-only

Build Celeront separately:

cd celeront
cargo build --release

10. Project Statistics

Metric| Current Record
Total source and documentation lines| 5,470
Main implementation language| Rust
Native backend| LLVM IR + Clang
Package manager| Celeront

The line count is an approximate project snapshot and may change as development continues.

11. Philosophy

AEROXA is built around the idea that programming languages should provide strong control over how programs are written, checked, and compiled.

Strictness is not intended merely to make programming difficult. It is intended to encourage explicit code, predictable behavior, and stronger compile-time guarantees.

AEROXA is an ongoing experiment in language design and compiler engineering.

---

AEROXA — STRICT UNIVERSAL PROGRAMMING LANGUAGE

CODE BEYOND LIMITS.
3.1 Language Syntax Reference

This section demonstrates the currently implemented AEROXA syntax.

1. Variable Declaration

Immutable variable:

let score: Int32 = 85;

Mutable variable:

let mut counter: Int32 = 0;

2. Arithmetic Operations

let a: Int32 = 20;
let b: Int32 = 5;

let addition: Int32 = a + b;
let subtraction: Int32 = a - b;
let multiplication: Int32 = a * b;
let division: Int32 = a / b;
let remainder: Int32 = a % b;

print(addition);
print(subtraction);
print(multiplication);
print(division);
print(remainder);

3. Comparison Operators

let a: Int32 = 20;
let b: Int32 = 10;

check a == b {
    print("Equal");
} orcheck a != b {
    print("Not Equal");
} #else$ {
    print("Unknown");
}

Supported comparison operators:

Operator| Description
"=="| Equal
"!="| Not equal
">"| Greater than
"<"| Less than
">="| Greater than or equal
"<="| Less than or equal

4. Logical Operators

let score: Int32 = 85;
let attendance: Int32 = 90;

check score >= 75 && attendance >= 80 {
    print("Passed");
} #else$ {
    print("Failed");
}

Supported logical operators:

Operator| Description
"&&"| Logical AND
`| 

5. Conditional Statements

let score: Int32 = 85;

check score >= 90 {
    print("Excellent");
} orcheck score >= 75 {
    print("Good");
} #else$ {
    print("Try Again");
}

AEROXA conditional keywords:

- "check"
- "orcheck"
- "#else$"

6. Loop Statements

let mut counter: Int32 = 0;

loopcheck counter < 5 {
    print(counter);
    counter = counter + 1;
}

"loopcheck" repeatedly executes its block while the condition is true.

7. Unary Operations

Negative integer:

let temperature: Int32 = -10;
print(temperature);

8. String Output

print("Hello, AEROXA!");
print("CODE BEYOND LIMITS");

9. Module Linking

Main source:

link::math;

let value: Int32 = 10;
print(value);

Module file: "modules/math.ex"

let number: Int32 = 100;
print(number);

The current implementation loads and integrates module AST statements. Namespace-based function calls are not supported yet.

10. Complete Example

let mut counter: Int32 = 0;
let limit: Int32 = 5;

loopcheck counter < limit {
    check counter < 3 {
        print("AEROXA: Processing");
    } #else$ {
        print("AEROXA: Final Stage");
    }

    counter = counter + 1;
}

This example combines mutable variables, immutable variables, loops, comparisons, conditional statements, and string output.

---

Syntax Compatibility Notice

AEROXA is still under active development. This reference documents the syntax currently implemented in the compiler. Features such as functions, arrays, classes, namespace calls, and advanced memory management are not yet represented as supported language features.