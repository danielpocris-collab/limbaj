# ng — Language Specification v0.1

> "From zero bytes to any machine. Written in itself from day one."

---

## 1. Philosophy

ng is a systems language designed to replace C, C++, and Rust for all layers:
- Bare metal / OS kernels
- Drivers and embedded
- Userspace applications
- High-performance services

**Core principles:**
1. Zero hidden cost — every allocation, copy, and effect is visible in the type
2. Zero undefined behavior — illegal states are unrepresentable
3. Zero runtime mandatory — the standard library is optional
4. Self-hosting — the compiler is written in ng, bootstrapped from a ~300-byte hex seed
5. One language, all layers — same syntax from interrupt handler to web server

---

## 2. Type System

### 2.1 Primitives

```ng
// integers (signed)
i8  i16  i32  i64  i128  isize

// integers (unsigned)
u8  u16  u32  u64  u128  usize

// floats
f32  f64

// boolean
bool  (true | false)

// character (Unicode scalar, 4 bytes)
char

// unit (zero-size, like Rust's ())
unit

// never (uninhabited, for diverging functions)
never
```

### 2.2 Compound Types

```ng
// tuple
(i32, f64, bool)

// fixed array
[N]T          // e.g. [4]f32

// slice (fat pointer: ptr + len)
[]T

// struct
type Point = {
    x: f32,
    y: f32,
}

// tuple struct
type Meters = (f32)

// enum (algebraic data type)
enum Shape {
    Circle(f32),
    Rect(f32, f32),
    Triangle { base: f32, height: f32 },
}

// union (unsafe, for C interop)
union U {
    as_int: u32,
    as_bytes: [4]u8,
}
```

### 2.3 Pointer Types

```ng
// raw pointer (unsafe only)
*T        // raw pointer to T
*mut T    // raw mutable pointer to T

// reference (safe, lifetime-tracked)
&T        // shared borrow
&mut T    // exclusive borrow

// owned heap pointer
Box[T]    // unique ownership, heap allocated

// generational reference (safe, no lifetime annotations needed)
Gen[T]    // safe alternative to *T for arena/pool allocation
```

### 2.4 Function Types

```ng
// function pointer
fn(i32, i32) -> i32

// closure (captures environment)
|i32, i32| -> i32

// function with effects
fn(str) -> str ! {IO, Error[ParseError]}
```

### 2.5 Generic Types

```ng
// generic struct
type Vec[T] = { ... }

// generic function
fn map[T, U](v: Vec[T], f: fn(T) -> U) -> Vec[U] { ... }

// constrained generics (traits)
fn sort[T: Ord](v: &mut Vec[T]) { ... }

// comptime generics (like Zig — duck typing at compile time)
comptime fn zero[T]() -> T {
    // T must have a zero() method — checked at instantiation
    T.zero()
}
```

### 2.6 Optional and Result

```ng
// no null — use Option
Option[T] = Some(T) | None

// no exceptions — use Result
Result[T, E] = Ok(T) | Err(E)

// ? operator for propagation
fn parse_int(s: str) -> Result[i32, ParseError] {
    let n = scan(s)?   // propagates Err automatically
    Ok(n)
}
```

### 2.7 Traits (Typeclasses)

```ng
trait Ord {
    fn cmp(self: &Self, other: &Self) -> Ordering
}

trait Clone {
    fn clone(self: &Self) -> Self
}

// impl for a type
impl Ord for i32 {
    fn cmp(self: &i32, other: &i32) -> Ordering {
        if *self < *other { Less }
        else if *self > *other { Greater }
        else { Equal }
    }
}

// default methods allowed
trait Display {
    fn fmt(self: &Self) -> str
    fn print(self: &Self) {    // default impl
        io.println(self.fmt())
    }
}
```

---

## 3. Memory Model

### 3.1 Ownership

Every value has exactly one owner. Ownership transfers on assignment (move semantics by default).

```ng
let a = Vec.new()    // a owns the Vec
let b = a            // ownership moves to b; a is no longer valid
```

### 3.2 Borrowing

```ng
fn print_len(v: &Vec[u8]) {         // shared borrow
    io.println(v.len)
}

fn push_zero(v: &mut Vec[u8]) {     // exclusive borrow
    v.push(0)
}
```

Rules (enforced by compiler):
- Any number of `&T` at the same time, OR
- Exactly one `&mut T`, never both simultaneously

### 3.3 Lifetimes (Implicit)

Lifetimes are inferred in 95% of cases. Explicit annotation only when inference is ambiguous:

```ng
// implicit — compiler infers
fn first(v: &Vec[str]) -> &str { v[0] }

// explicit — when two references have different origins
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len > y.len { x } else { y }
}
```

### 3.4 Memory Modes

Three memory modes, selected per module or per block:

```ng
// @mode(static)    — no heap, no dynamic alloc (kernel/ISR)
// @mode(region)   — region-based alloc (driver, zero fragmentation)
// @mode(heap)     — full heap with ownership (userspace apps)

@mode(static)
fn irq_handler() ! {Hardware} {
    // only stack and static memory allowed here
}
```

### 3.5 Generational References (arena-safe)

For code using arenas or pools, `Gen[T]` replaces raw pointers with safe generation-checked access:

```ng
let arena = Arena.new()
let r: Gen[Node] = arena.alloc(Node { ... })
// r.get() returns Option[&Node] — None if the node was freed
```

---

## 4. Algebraic Effects

Effects replace: exceptions, async/await, generators, IO, global state.

### 4.1 Effect Declaration

```ng
effect IO {
    fn print(s: str) -> unit
    fn read_line() -> str
    fn read_file(path: str) -> Result[str, IOError]
}

effect Async {
    fn suspend() -> unit
    fn spawn[T](f: fn() -> T ! {Async}) -> Task[T]
}

effect Fail[E] {
    fn raise(e: E) -> never
}
```

### 4.2 Effect Annotation

```ng
// function uses IO and can fail with ParseError
fn load_config(path: str) -> Config ! {IO, Fail[ParseError]} {
    let text = IO.read_file(path)?
    parse_config(text)
}

// pure function — no effects
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 4.3 Effect Handlers

```ng
fn main() {
    // handle IO effect
    handle load_config("config.ng") with {
        IO.print(s) => { libc.puts(s); resume unit }
        IO.read_line() => { resume libc.gets() }
        IO.read_file(path) => { resume fs.read(path) }
    }
}
```

### 4.4 Async via Effects (no function coloring)

```ng
// async is just the Async effect — no special syntax
fn fetch(url: str) -> str ! {Async, IO, Fail[NetError]} {
    let resp = http.get(url)!
    resp.body
}

// run with an executor
fn main() {
    let rt = Runtime.new()
    rt.block_on(fetch("https://example.com"))
}
```

---

## 5. Syntax

### 5.1 Variables

```ng
let x = 42           // immutable binding, type inferred
let y: f32 = 3.14    // explicit type
var z = 0            // mutable binding
z = z + 1            // mutation
```

### 5.2 Functions

```ng
fn add(a: i32, b: i32) -> i32 {
    a + b    // last expression is the return value
}

// early return
fn safe_div(a: i32, b: i32) -> Option[i32] {
    if b == 0 { return None }
    Some(a / b)
}

// inline / comptime
comptime fn pow2(n: u32) -> u32 {
    1u32 << n
}
```

### 5.3 Control Flow

```ng
// if expression (returns value)
let abs = if x >= 0 { x } else { -x }

// match (exhaustive)
match shape {
    Circle(r)                => pi * r * r,
    Rect(w, h)               => w * h,
    Triangle { base, height } => base * height / 2.0,
}

// match with guards
match n {
    0       => "zero",
    x if x < 0 => "negative",
    _       => "positive",
}

// while
while condition {
    // ...
}

// for (iterator)
for item in collection {
    // ...
}

// loop with break value
let result = loop {
    let val = compute()
    if val > 0 { break val }
}
```

### 5.4 Pattern Matching

```ng
// destructuring in let
let (x, y) = get_point()
let { name, age } = person

// in function parameters
fn norm({ x, y }: Point) -> f32 {
    sqrt(x*x + y*y)
}

// nested
match event {
    KeyDown { key: Key.Enter, mods: Mods { shift: true, .. } } => submit(),
    KeyDown { key, .. } => handle_key(key),
    _ => {}
}
```

### 5.5 Closures

```ng
let double = |x: i32| x * 2
let add = |x, y| x + y      // types inferred from usage

// capturing by borrow
let prefix = "hello"
let greet = |name| prefix + " " + name

// capturing by move
let data = load_data()
let process = move || analyze(data)
```

### 5.6 Structs and Methods

```ng
type Rect = {
    width:  f32,
    height: f32,
}

impl Rect {
    fn new(w: f32, h: f32) -> Rect {
        Rect { width: w, height: h }
    }

    fn area(self: &Rect) -> f32 {
        self.width * self.height
    }

    fn scale(self: &mut Rect, factor: f32) {
        self.width  *= factor
        self.height *= factor
    }
}

// usage
let r = Rect.new(3.0, 4.0)
let a = r.area()
```

### 5.7 Enums

```ng
enum List[T] {
    Nil,
    Cons(T, Box[List[T]]),
}

impl List[i32] {
    fn sum(self: &List[i32]) -> i32 {
        match self {
            Nil          => 0,
            Cons(x, xs)  => x + xs.sum(),
        }
    }
}
```

---

## 6. Module System

```ng
// file: math/vector.ng
module math.vector

export type Vec3 = { x: f32, y: f32, z: f32 }

export fn dot(a: Vec3, b: Vec3) -> f32 {
    a.x*b.x + a.y*b.y + a.z*b.z
}

fn internal_helper() { ... }    // not exported


// usage in another file:
import math.vector { Vec3, dot }
import math.vector              // access as math.vector.Vec3
import math.vector as v         // access as v.Vec3
```

No header files. No forward declarations. The compiler resolves order automatically.

---

## 7. Concurrency

```ng
// spawn a task (structured — task is tied to current scope)
let handle = spawn {
    compute_heavy()
}
let result = handle.await

// channels
let (tx, rx) = Channel[i32].new()

spawn {
    for i in 0..10 {
        tx.send(i)
    }
    tx.close()
}

for value in rx {
    io.println(value)
}

// select (like Go)
select {
    ch1.recv() => |v| handle_v(v),
    ch2.recv() => |v| handle_w(v),
    timeout(1s) => handle_timeout(),
}
```

Send/Sync are automatically derived from types — no manual annotation needed.

---

## 8. C Interop

```ng
// declare C functions
extern "c" {
    fn malloc(size: usize) -> *mut u8
    fn free(ptr: *mut u8)
    fn printf(fmt: *u8, ...) -> i32
}

// use in unsafe block
fn alloc_raw(n: usize) -> *mut u8 {
    unsafe { malloc(n) }
}

// C-compatible structs
@repr(c)
type CPoint = {
    x: f64,
    y: f64,
}

// export for C
@export
fn ng_add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## 9. Unsafe

Only in explicitly marked blocks. Minimal surface area:

```ng
unsafe {
    // raw pointer dereference
    let val = *ptr

    // pointer arithmetic
    let next = ptr.offset(1)

    // call unsafe function
    dangerous_fn()

    // transmute types
    let x: u32 = transmute(f32_val)
}
```

Safe abstractions can be built over unsafe code (like Rust's `Vec`, `Mutex`).

---

## 10. Comptime

Zig-style compile-time execution, more powerful than C++ templates:

```ng
// compile-time function
comptime fn type_name[T]() -> str {
    T.name    // intrinsic
}

// compile-time if
comptime if target.arch == .x86_64 {
    fn syscall(nr: u64) -> u64 { asm("syscall") }
} else {
    fn syscall(nr: u64) -> u64 { ... }
}

// compile-time loop (code generation)
comptime for field in fields(MyStruct) {
    // generates code for each field
}

// type-level programming
fn make_tuple[comptime N: usize]() -> ... {
    comptime tuple_type(N, i32)
}
```

---

## 11. Error Handling

Three mechanisms, choose based on context:

```ng
// 1. Result[T,E] — recoverable, local
fn parse(s: str) -> Result[i32, ParseError] { ... }
let n = parse("42")?    // propagate with ?

// 2. Algebraic effect Fail[E] — recoverable, across call stack
fn load() -> Config ! {Fail[ConfigError]} {
    Fail.raise(ConfigError.Missing("key"))
}

// 3. abort() — unrecoverable (like Rust's panic!, but no unwinding)
fn checked_div(a: i32, b: i32) -> i32 {
    if b == 0 { abort("division by zero") }
    a / b
}
```

No exceptions. No unwinding. Errors are values.

---

## 12. Attributes

```ng
@inline          // force inline
@noinline        // never inline
@cold            // cold path hint
@export          // export symbol (C ABI)
@repr(c)         // C-compatible layout
@repr(packed)    // packed struct
@align(16)       // alignment
@mode(static)    // memory mode override
@deprecated      // warn on use
@test            // marks a test function
@comptime_only   // function only callable at compile time
```

---

## 13. Bootstrap Chain

The compiler itself is written in ng, bootstrapped from scratch:

```
[seed.bin]  ~300 bytes, written by hand in hex
            Architecture-specific (x86-64, ARM64, RISC-V, PE/ELF)
            Implements: read, write, exit + 8-instruction bytecode VM

[stage1/]   Written in seed bytecode (~3KB)
            Implements: tokenizer, LL(1) parser, bytecode emitter
            Input: ng subset (no generics, no effects, no closures)
            Output: seed bytecode

[stage2/]   Written in ng subset (~20KB)
            Full type checker, ownership checker (simplified)
            Emits: x86-64 native or seed bytecode
            Input: ng with generics and traits
            Output: native binary or bytecode

[stage3/]   Full ng compiler (~100KB)
            Written in ng (all features)
            Compiles itself via stage2
            After this: stage2 is discarded

[final]     Self-hosting ng compiler
            stage3 compiles stage3 → identical output (fixed point)
```

Each stage is verifiable. The seed hex is small enough to be read by a human.

---

## 14. Standard Library Structure

```
ng.core        — primitives, Option, Result, Never
ng.mem         — allocators, Box, Arena, Pool
ng.collections — Vec, Map, Set, Queue, Ring
ng.str         — String, str, utf8
ng.io          — File, Stdin, Stdout, Stderr
ng.net         — TCP, UDP, HTTP (via effects)
ng.sync        — Channel, Mutex, Atomic
ng.os          — process, env, signal, fs
ng.math        — arithmetic, trig, float ops
ng.fmt         — formatting, display, debug
ng.test        — test framework (@test functions)
ng.comptime    — type introspection, code generation
ng.ffi         — C interop utilities
ng.kernel      — bare-metal / OS dev (no alloc, no runtime)
```

---

## 15. Target Platforms

| Platform       | Memory Mode | Runtime | Status    |
|----------------|-------------|---------|-----------|
| x86-64 Linux   | all         | none    | primary   |
| x86-64 Windows | all         | none    | primary   |
| ARM64 Linux    | all         | none    | primary   |
| RISC-V bare    | static only | none    | primary   |
| ARM Cortex-M   | static only | none    | embedded  |
| WASM           | heap        | minimal | secondary |

---

## 16. What ng Deliberately Omits

| Feature            | Why omitted                              | Alternative        |
|--------------------|------------------------------------------|--------------------|
| Garbage collector  | unpredictable latency, hidden cost       | ownership + regions|
| Null               | billion-dollar mistake                   | Option[T]          |
| Exceptions         | invisible control flow, no cost visible  | Result + effects   |
| Header files       | redundant, causes ordering bugs          | module system      |
| Macros             | opaque, hard to analyze                  | comptime           |
| Inheritance        | fragile base class problem               | traits + composition|
| Implicit copies    | hidden allocations                       | explicit .clone()  |
| Global mutable state | data races, hard to reason about       | effects + channels |
| Runtime reflection | unpredictable, breaks dead code elim.    | comptime introspect|
| Undefined behavior | root cause of most security bugs         | everything defined |

---

*ng — 0 dependencies. 0 GC. 0 undefined behavior. Written in itself.*
