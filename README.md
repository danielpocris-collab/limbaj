# Limbaj: Production-Grade Systems Programming Language

**Current Status:** BATCH 1 - Complete

Limbaj is a systems programming language designed for:
- Systems tooling, compilers, runtimes, query engines, infrastructure software
- Cognitive simplicity compared to Rust with stricter semantics than Go
- Low-level control matching Zig's capabilities
- First-class arena/region-based memory management
- Capability-based effects system (no hidden control flow)
- Structural concurrency with explicit task graphs
- Integrated build, test, format toolchain

## BATCH 1: Complete Execution Model

### What's Implemented

✅ **Lexer**: Full tokenization with position tracking
- All BATCH 1 token types (keywords, operators, literals, types)
- String escape sequences
- Comment support
- Identifier and string pools

✅ **Parser**: Recursive descent parser with error recovery
- Function definitions with parameters and return types
- All expression types (binary, unary, function calls)
- All statement types (let, return, if/else, match)
- Pattern matching for Result and Option types
- Type annotations

✅ **Type Checker**: Static type verification
- Function signature verification
- Type inference for local variables
- Exhaustive pattern matching for Result/Option
- Type mismatch detection with clear error messages
- Proper handling of all base types and composite types

✅ **Interpreter**: Stack-based execution engine
- Function call with parameter binding
- Local variable scoping
- Arithmetic operations (with proper zero-check)
- Logical operations
- Pattern matching with binding
- Result/Option value handling
- Proper type-safe value representation

✅ **Diagnostics**: Error reporting
- Position-aware error messages (line:col)
- Descriptive error text
- Suggestion framework

✅ **Toolchain**: CLI integration
- `limbaj build <file>` - Parse and type-check
- `limbaj run <file>` - Compile and execute

### Language Features (BATCH 1)

**Types:**
- `i64`: 64-bit signed integer
- `f64`: IEEE 754 double-precision floating point
- `bool`: Boolean
- `str`: Immutable string
- `void`: No value
- `Result[T, E]`: Success or failure with values
- `Option[T]`: Present or absent value

**Expressions:**
- Integer/float/bool/string literals
- Identifiers and variables
- Function calls with arguments
- Binary operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`
- Unary operators: `!`, `-`
- Result constructors: `Ok(value)`, `Err(value)`
- Option constructors: `Some(value)`, `None`
- Block expressions

**Statements:**
- Expression statements
- `let name: type = value;` - Variable declaration with optional type annotation
- `return value;` - Function return
- `if condition { ... } else { ... }` - Conditional execution
- `match expr { pattern => { ... } }` - Exhaustive pattern matching

**Functions:**
```
fn function_name(param1: Type1, param2: Type2) -> ReturnType {
  // statements and expressions
}
```

**Pattern Matching:**
```
match result {
  Ok(value) => { /* handle success */ }
  Err(err) => { /* handle error */ }
}

match option {
  Some(val) => { /* handle present */ }
  None => { /* handle absent */ }
}
```

### Example Programs

**Basic Arithmetic:**
```limbaj
fn main() -> i64 {
    let x = 10;
    let y = 20;
    return x + y;
}
```
Output: `30`

**Recursion (Factorial):**
```limbaj
fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() -> i64 {
    return factorial(5);
}
```
Output: `120`

**FizzBuzz with Functions:**
```limbaj
fn is_divisible_by_15(n: i64) -> bool {
    return n % 15 == 0;
}

fn fizzbuzz(n: i64) -> i64 {
    if is_divisible_by_15(n) {
        return 15;
    }
    return n;
}

fn main() -> i64 {
    return fizzbuzz(15);
}
```
Output: `15`

### Testing

**Unit Tests:** 15 tests covering all components
```bash
cargo test --lib
```

**Integration Tests:**
- `tests/programs/fizzbuzz.limbaj` - FizzBuzz variant
- `tests/programs/basic_arithmetic.limbaj` - Basic arithmetic
- `tests/programs/logic.limbaj` - Boolean logic
- `tests/programs/recursion.limbaj` - Recursive functions

All test programs execute correctly end-to-end.

### Building and Running

**Build only (parse + type check):**
```bash
cargo build --release
./target/release/limbaj build program.limbaj
```

**Build and run:**
```bash
./target/release/limbaj run program.limbaj
```

### Architecture

```
main.rs
├── CLI interface (build, run commands)
└── Integration of components

lib.rs
├── lexer.rs          - Tokenization
├── ast.rs            - Abstract syntax tree
├── parser.rs         - Parsing (tokens → AST)
├── type_checker.rs   - Type verification
├── interpreter.rs    - Execution engine
├── value.rs          - Runtime values
└── diagnostics.rs    - Error reporting
```

### Success Criteria (All Met ✓)

✓ Lexer produces correct tokens for all BATCH 1 syntax
✓ Parser builds valid AST for all constructs
✓ Type checker rejects all type errors with clear messages
✓ Type checker verifies exhaustive pattern matching
✓ Interpreter executes arithmetic correctly
✓ Interpreter executes function calls with proper scoping
✓ Match expressions work correctly on Result/Option
✓ Real programs compile and execute correctly
✓ Error messages include location and context
✓ `limbaj build` and `limbaj run` work end-to-end

---

## BATCH 2: Structs & Modules (Next)

Will include:
- Struct definitions with named fields
- Field access and initialization
- Module system with packages
- Multi-file compilation with explicit imports
- Real program: JSON parser/pretty-printer

## BATCH 3: Arena Memory & Ownership

Will include:
- Arena allocation syntax and semantics
- Ownership transfer rules
- Lifetime tracking in types
- Diagnostics for memory errors
- Real program: String builder with custom allocator

## BATCH 4: Structured Concurrency

Will include:
- Task definition and spawning
- Channel-based communication
- Task graph execution
- Ownership across task boundaries
- Real program: Parallel file processor

## BATCH 5: Capability System

Will include:
- Capability types for I/O, time, networking
- Capability threading through call chains
- Real program: HTTP request processor

---

## Design Philosophy

1. **Explicit over implicit** - No hidden allocations, no implicit conversions, no magic control flow
2. **Memory safety without complexity** - Arena-based allocation is simpler than borrow checking
3. **Structured concurrency** - Tasks are values, not opaque threads
4. **Capabilities for effects** - I/O, time, platform access require explicit capability parameters
5. **Zero-cost abstractions** - What you write is what you get
6. **Exceptional error handling** - Errors are values, exhaustively matched

## Contributing

BATCH 1 is complete and tested. All code is production-quality with:
- Complete error handling
- Comprehensive type safety
- Clear diagnostics
- Full integration testing
- Zero unsafe code (in Limbaj layer)

New features must follow strict standards:
- End-to-end implementation (syntax → semantics → execution)
- Real executable programs as validation
- Comprehensive error handling
- No placeholders or "TODO" implementations
