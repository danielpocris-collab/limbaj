# BATCH 1 Language Design Specification

Historical note:
- acest document descrie designul istoric Batch 1
- detaliile despre implementarea in Rust nu mai descriu worktree-ul operational curent
- pentru traseul activ foloseste `STATUS.md` si `README.md`

## Execution Model

### Stack-Based Interpreter

The Limbaj BATCH 1 execution model is an eager, stack-based interpreter with:

**Value Stack**
- Values are evaluated on a stack
- Each operation pushes/pops values
- Supports all BATCH 1 types

**Call Stack**
- Function calls create activation frames
- Each frame holds:
  - Parameter values (immutable)
  - Local variables (mutable within frame scope)
  - Return address (implicit via Rust call stack)
- LIFO discipline ensures proper unwinding

**Evaluation Order**
- Left-to-right for binary operations
- Arguments before function call
- Eager (not lazy) evaluation

### Type System Rules

**Base Types:**
```
i64  → 64-bit signed integer (-2^63 to 2^63-1)
f64  → IEEE 754 double (53-bit precision)
bool → true, false (1 bit logical value)
str  → immutable Unicode string (heap-allocated)
void → unit type (no value)
```

**Composite Types:**
```
Result[T, E] → Ok(T) | Err(E)
             → Explicitly handled, no implicit unwrap
             → Pattern matching is mandatory

Option[T]    → Some(T) | None
             → Explicitly handled, no implicit unwrap
             → Pattern matching is mandatory
```

**Type Checking Rules:**

1. **Function Signatures Are Explicit**
   - All parameters must have explicit types
   - Return type must be explicitly declared
   - No type inference for function signatures

2. **Local Variable Types Can Be Inferred**
   ```
   let x = 42;        // x: i64 (inferred from literal)
   let y: f64 = 3.14; // y: f64 (explicit annotation)
   ```

3. **No Implicit Conversions**
   - `i64` and `f64` are distinct
   - String literals are `str` type
   - All type mismatches are errors

4. **Exhaustive Pattern Matching**
   - All patterns for Result/Option must be covered
   - Missing patterns are compile errors
   - Prevents runtime panics

5. **Arithmetic Type Compatibility**
   - Binary operators require matching types
   - `i64 + i64 → i64`
   - `f64 + f64 → f64`
   - `i64 + f64 → Type Error`

### Memory Model (BATCH 1)

**No explicit memory management in BATCH 1**

- Values on stack for parameters and locals
- String values are heap-allocated and ref-counted by Rust
- Result/Option wrapper types are by-value (no pointers)
- Function frames allocated on call stack
- All cleanup is automatic via Rust's ownership

Future batches will add:
- Explicit arena allocation
- Ownership transfer semantics
- Lifetime parameters

### Error Handling Model

**Explicit Result Types**
```
Result[SuccessType, ErrorType]
```

Not exceptions - errors are first-class values:
- Must be explicitly created: `Ok(value)` or `Err(error)`
- Must be explicitly matched
- Pattern matching forces handling

**No implicit error propagation**
- Each function must declare what errors it returns
- Caller must handle returned errors
- No try/catch, only explicit match

### Function Call Semantics

**Parameter Passing**
- All parameters are passed by value
- Primitives (i64, f64, bool) are stack-copied
- Strings are cloned (cheap for now, optimizable later)
- Values inside Result/Option are cloned

**Variable Scope**
```
fn outer() -> i64 {
    let x = 1;          // outer scope
    {
        let x = 2;      // inner scope shadows outer
        let y = 3;      // inner scope only
    }
    // y is not accessible here
    return x;           // refers to outer x
}
```

**Return Values**
- Function must return value matching declared type
- `return expr;` exits function with value
- Implicit return via last expression coming in BATCH 2

### Pattern Matching Semantics

**Result Pattern Matching**
```
match result {
    Ok(value) => {
        // value has type T (from Result[T, E])
        // execute on success path
    }
    Err(err) => {
        // err has type E
        // execute on error path
    }
}
```

Both branches mandatory. Compiler verifies exhaustiveness.

**Option Pattern Matching**
```
match option {
    Some(value) => {
        // value has type T (from Option[T])
        // execute when present
    }
    None => {
        // execute when absent
    }
}
```

Both branches mandatory.

**Pattern Bindings**
- Patterns introduce variables in their branch scope
- Variable lifetime ends at end of branch
- Variables from patterns shadow outer variables

### Operators and Precedence

**Precedence (highest to lowest):**
```
1. Primary        ()  []  {}  literals  identifiers
2. Unary          -   !
3. Multiplicative *   /   %
4. Additive       +   -
5. Comparison     <   >   <=  >=  ==  !=
6. Logical AND    &&
7. Logical OR     ||
```

All binary operators are left-associative.

**Arithmetic:**
- `+` addition
- `-` subtraction  
- `*` multiplication
- `/` integer division (truncates toward zero)
- `%` modulo (sign follows dividend)

**Comparison:**
- `==` structural equality (all types except void)
- `!=` structural inequality
- `<` `>` `<=` `>=` numeric types only

**Logical:**
- `&&` logical AND (short-circuit left-to-right)
- `||` logical OR (short-circuit left-to-right)
- `!` logical NOT

### String Semantics

**String Literals**
```
"hello"      // ASCII
"hello\n"    // Escape sequences: \n \t \r \\ \"
```

**String Type**
- Immutable
- Heap-allocated
- Reference-counted in Rust layer
- No string interpolation in BATCH 1

**Operations**
- No concatenation operator in BATCH 1
- No indexing in BATCH 1
- Comparison: `==` and `!=` for equality

### Boolean Semantics

**Literals**
```
true
false
```

**Logical Operators**
```
true && true   → true
true && false  → false
false || false → false
true || false  → true
!true          → false
```

**Numeric Comparisons Return bool**
```
5 < 10    → true
5 == 5    → true
5 != 5    → false
```

---

## Implementation Details

### Lexer Implementation

**Token Pool Strategy**
- Identifiers and strings are interned in pools
- Tokens reference pool indices
- Reduces memory usage for common identifiers
- Retrieved by name lookup when needed

**Error Recovery**
- Lexer reports position of unexpected character
- Continues after single-character errors
- Enables better error messages from parser

### Parser Implementation

**Recursive Descent**
- Separate parsing function for each precedence level
- Clean separation of concerns
- Easy to understand control flow

**Expression Precedence**
```
parse_expression() → parse_or()
parse_or()        → parse_and() { || parse_and() }
parse_and()       → parse_equality() { && parse_equality() }
parse_equality()  → parse_comparison() { (== | !=) parse_comparison() }
parse_comparison()→ parse_addition() { (< | > | <= | >=) parse_addition() }
parse_addition()  → parse_mult() { (+ | -) parse_mult() }
parse_mult()      → parse_unary() { (* | / | %) parse_unary() }
parse_unary()     → (- | !) parse_unary() | parse_primary()
parse_primary()   → literals | identifiers | calls | parens | blocks
```

### Type Checker Implementation

**Two-Pass Algorithm**

Pass 1: Collect function signatures
- Map: function_name → (param_types, return_type)
- Enables recursive function definitions

Pass 2: Type check each function
- Initialize variable scope with parameters
- Type check all statements in function
- Verify all paths return correct type

**Symbol Table**
- Stack of scopes (one per block)
- Maps variable name → type
- Lookup searches inner-to-outer

**Exhaustiveness Verification**
- For Result: must have Ok(...) and Err(...) arms
- For Option: must have Some(...) and None arms
- Compiler error if any pattern missing

### Interpreter Implementation

**Value Representation**
```rust
enum Value {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Unit,
    Ok(Box<Value>),
    Err(Box<Value>),
    Some(Box<Value>),
    None,
}
```

**Scope Management**
- Stack of HashMaps
- New scope pushed on function call
- Popped on function return
- Local variables inserted/looked-up in current scope

**Return Values**
- Any non-Unit return value signals function exit
- Unit return allows statement to continue

---

## Error Messages

**Type Mismatch Example:**
```
let x: i32 = 42;  // Type i32 not available yet
```
Message:
```
Unknown type: i32
```

**Function Not Found:**
```
let x = unknown_func();
```
Message:
```
Undefined function: unknown_func
```

**Pattern Match Not Exhaustive:**
```
match result {
    Ok(x) => { return 1; }
}
```
Message:
```
Match on Result must have Ok and Err patterns
```

**Argument Count Mismatch:**
```
fn add(x: i64, y: i64) -> i64 { ... }
add(5)
```
Message:
```
Function add expects 2 arguments, got 1
```

---

## Performance Characteristics

**No Optimization in BATCH 1**
- Direct interpreter (not bytecode)
- No dead code elimination
- No constant folding
- No tail call optimization

Performance is adequate for:
- Scripts and tooling
- Learning/testing
- Prototyping

Optimization will be added in future batches.

---

## Testing Strategy

**Unit Tests (15 total)**
- Lexer: tokenization, operators, strings, numbers
- Parser: simple functions, parameters
- Type Checker: simple programs
- Interpreter: arithmetic, functions
- Diagnostics: error creation
- Value: equality, comparison

**Integration Tests (4 programs)**
- FizzBuzz variant: multiple functions, control flow
- Basic Arithmetic: variables, operators
- Logic: boolean operations
- Recursion: function calls, loops via recursion

All tests pass ✓

---

## Known Limitations (By Design)

**BATCH 1 Scope**
- No structs (BATCH 2)
- No modules (BATCH 2)
- No heap allocation (explicit) (BATCH 3)
- No concurrency (BATCH 4)
- No I/O capabilities (BATCH 5)
- No string operations (coming later)
- No arrays/slices (coming later)
- No generics (coming later)

**Intentional Gaps**
- No implicit type conversions (keep it simple)
- No operator overloading (explicit is better)
- No default parameters (clarity)
- No variable arguments (explicit)

---

## Next Steps (BATCH 2)

1. Add struct definitions and field access
2. Add field initialization syntax
3. Implement module system
4. Multi-file compilation
5. Real program: JSON parser

All with same quality standards: end-to-end, tested, production-ready.
