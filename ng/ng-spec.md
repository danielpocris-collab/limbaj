# ng — Language Specification v0.2

> "From zero bytes to any machine. Written in itself from day one."
> "The compiler proves it. The type system prevents it. The runtime never sees it."

---

## 1. Philosophy

ng is a systems language designed to replace C, C++, and Rust for all layers:
- Bare metal / OS kernels
- Drivers and embedded
- Userspace applications
- High-performance services
- Distributed systems
- Cryptographic infrastructure

**Core principles:**
1. Zero hidden cost — every allocation, copy, and effect is visible in the type
2. Zero undefined behavior — illegal states are unrepresentable
3. Zero runtime mandatory — the standard library is optional
4. Self-hosting — the compiler is written in ng, bootstrapped from a ~300-byte hex seed
5. One language, all layers — same syntax from interrupt handler to web server
6. Provably correct — the compiler is a theorem prover, not just a type checker
7. Security by construction — secrets cannot leak, capabilities cannot be forged
8. Time-aware — expiry, causality, and ordering are first-class type properties
9. Distributed-native — values on remote machines are typed the same as local values
10. Reversible by default — pure functions have automatic inverses

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

### 4.5 Scheduler as Effect

Concurrency is not a built-in primitive — it is an algebraic effect.
This means you can swap schedulers without changing any call site.

```ng
// the Spawn effect: ability to create concurrent tasks
effect Spawn {
    fn spawn[T](task: fn() -> T) -> TaskHandle[T]
    fn yield_now()
}

// a function that uses concurrency — scheduler-agnostic
fn parallel_map[T, U](items: Vec[T], f: fn(T) -> U) -> Vec[U]
    ! {Spawn}
{
    let handles = items.map(|x| Spawn.spawn(|| f(x)))
    handles.map(|h| h.join()).collect()
}

// green thread scheduler handler
handle Spawn with GreenThreads {
    spawn(task) => GreenThreads.submit(task),
    yield_now() => GreenThreads.switch(),
}

// single-threaded event loop handler
handle Spawn with EventLoop {
    spawn(task) => EventLoop.queue(task),
    yield_now() => EventLoop.poll(),
}

// thread-pool handler
handle Spawn with ThreadPool(N) {
    spawn(task) => GLOBAL_POOL.submit(task),
    yield_now() => {},
}

// the scheduler is injected at the call site — no global state
fn main() {
    handle Spawn with GreenThreads {
        parallel_map(my_data, process)
    }
}
```

io_uring is exposed as an effect handler in `ng.io`:

```ng
// Linux io_uring: the IO effect handler uses io_uring under the hood
fn main() {
    handle IO with IOUring {           // submits all IO through io_uring
        read_all_files(paths)          // this code is unchanged — it just uses IO effect
    }
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
@stable_abi      // compiler guarantees layout never changes across versions
@reversible      // marks a pure function as having a compiler-generated inverse
@wire(json)      // first-class serialization format (json | msgpack | binary | protobuf)
@wire(binary)
@cap(File)       // requires capability to call this function
@invariant       // marks a field or struct with runtime+compile-time invariant
@proof           // attaches an SMT proof obligation to a function
@tainted(Secret) // marks return value as information-flow tainted
@causal          // tracks value provenance for audit trail
@ttl(3600s)      // temporal type: value expires after duration
@simd            // enables SIMD vectorization on function or type
@gpu             // marks function for GPU execution
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
ng.io          — File, Stdin, Stdout, Stderr, io_uring (Linux native async)
ng.net         — TCP, UDP, HTTP, TLS (via effects, async-first)
ng.sync        — Channel, Mutex, Atomic
ng.os          — process, env, signal, fs
ng.math        — arithmetic, trig, float ops, SIMD (f32x8, f64x4, i32x16)
ng.fmt         — formatting, display, debug
ng.test        — test framework (@test functions)
ng.comptime    — type introspection, code generation
ng.ffi         — C interop utilities
ng.kernel      — bare-metal / OS dev (no alloc, no runtime)
ng.crypto      — post-quantum (ML-KEM, ML-DSA), AES-GCM, ChaCha20, Blake3
ng.proof       — SMT solver integration (Z3), contract verification
ng.dist        — Remote[T], distributed channels, location transparency
ng.cap         — capability definitions and propagation
ng.flow        — information flow labels (Secret, Public, Trusted)
ng.time        — temporal types, TTL tracking, clock effects
ng.serial      — wire format codecs (@wire implementations)
ng.causal      — causal type tracking, audit trail generation
ng.gpu         — GPU kernel dispatch, buffer management
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

## 17. Dependent Types (Lite)

Not full theorem proving — just the cases that eliminate real bugs:

```ng
// array access: index must be provably < length
fn get[T, comptime N: usize](arr: [N]T, i: usize) -> T
    where i < N   // checked at compile time when i is known
                  // checked at runtime with zero overhead when i is unknown

// non-zero integer
type NonZero[T: Int] = T where self != 0

fn divide(a: i64, b: NonZero[i64]) -> i64 {
    a / b   // division by zero: impossible by construction
}

// bounded integers
type Port = u16 where self >= 1 && self <= 65535
type Probability = f64 where self >= 0.0 && self <= 1.0

// length-indexed vectors
fn zip[T, U, comptime N: usize](a: [N]T, b: [N]U) -> [N](T, U)
    // lengths are equal by type — no runtime check needed
```

The compiler uses the Z3 SMT solver to discharge proof obligations at compile time.
Unknown values fall back to runtime bounds checks (never silently skipped).

---

## 18. Capability-Based Security

Capabilities replace ambient authority. A function cannot access a resource
unless a capability token is passed in — explicitly, at the type level.

```ng
// capability definitions (in ng.cap)
cap FileCap      // ability to open files
cap NetCap       // ability to make network connections
cap SpawnCap     // ability to spawn processes
cap ClockCap     // ability to read the current time (affects determinism)

// function requires FileCap
fn read_config(path: str, @cap file: FileCap) -> Result[str, IOError]
    ! {IO, Fail[IOError]}
{
    IO.read_file(path)
}

// main receives all capabilities from the OS
fn main(@caps all: AllCaps) {
    // pass only what each subsystem needs
    let config = read_config("app.ng", all.file)?

    // a sandboxed component gets zero capabilities
    let result = sandbox(untrusted_plugin, NoCaps)
}
```

Rules:
- Capabilities cannot be forged (no `FileCap.new()` outside `main`)
- Capabilities can be attenuated (restricted), never amplified
- The type system tracks capability flow — a `@test` function runs with `NoCaps` by default

---

## 19. First-Class Serialization

Types define their wire format. No separate schema files. No runtime reflection.

```ng
@wire(json, binary, msgpack)
type Config = {
    host:    str,
    port:    Port,          // u16 where 1..=65535
    timeout: Duration,
    tls:     bool,
}

// compiler generates at compile time:
// Config.to_json()    -> str
// Config.from_json()  -> Result[Config, ParseError]
// Config.to_binary()  -> []u8
// Config.from_binary()-> Result[Config, ParseError]
// Config.to_msgpack() -> []u8

// field-level control
type User = {
    name:     str,
    email:    str,
    @wire(skip) password_hash: [32]u8,   // never serialized
    @wire(rename = "created") created_at: Timestamp,
}

// versioned formats
@wire(binary, version: 2)
@wire_migrate(1 -> 2, fn migrate_v1_to_v2)
type Packet = { ... }
```

Zero runtime reflection. All codecs are generated at compile time.
Invalid input is caught at the type boundary, not deep in application logic.

---

## 20. SIMD and GPU (First-Class)

```ng
// SIMD types — map directly to hardware registers
type f32x8  = [8]f32  @simd    // AVX 256-bit
type f32x16 = [16]f32 @simd    // AVX-512
type i32x8  = [8]i32  @simd

// SIMD operations (compiler selects best instruction)
fn dot8(a: f32x8, b: f32x8) -> f32 {
    (a * b).sum()   // compiles to: vmulps + vdpps or vhaddps chain
}

// auto-vectorization hint
@simd
fn scale(v: []f32, factor: f32) {
    for x in v { x *= factor }  // compiler vectorizes automatically
}

// GPU kernels (same language, different execution model)
@gpu
fn matrix_mul(a: GpuBuffer[f32], b: GpuBuffer[f32], out: GpuBuffer[f32],
              n: u32) @thread_id(x, y) {
    let row = x
    let col = y
    var sum = 0.0f32
    for k in 0..n { sum += a[row*n + k] * b[k*n + col] }
    out[row*n + col] = sum
}

// dispatch GPU kernel from CPU
fn main() {
    let a = GpuBuffer.upload(matrix_a)
    let b = GpuBuffer.upload(matrix_b)
    let out = GpuBuffer.alloc[f32](n * n)
    matrix_mul.dispatch(grid: (n, n), a, b, out, n)
    let result = out.download()
}
```

---

## 21. Stable ABI

```ng
// marks struct layout as immutable across all future compiler versions
@stable_abi
type Point = {
    x: f64,
    y: f64,
}

// compiler REFUSES to change layout, reorder fields, or add padding
// error if you try to change a @stable_abi type in a breaking way

@stable_abi
type ApiResult[T] = {
    status:  u32,
    payload: T,
}

// versioned stable ABI
@stable_abi(since: "1.0")
type NetworkPacket = { ... }
```

Enables: shared libraries, plugin systems, cross-language binaries — all with guaranteed layout.

---

## 22. Hot Reload

Live programming: change code while the program runs. State is preserved.

```ng
// mark a module as hot-reloadable
@hot_reload
module app.ui

// when code changes, compiler:
// 1. recompiles the module
// 2. migrates existing state using the migration function
// 3. replaces function pointers atomically

// state migration (old -> new)
@migrate(from: "1.0", to: "1.1")
fn migrate_widget_state(old: WidgetState_v1) -> WidgetState_v2 {
    WidgetState_v2 {
        x: old.x,
        y: old.y,
        visible: true,   // new field with default
    }
}
```

In kernel/embedded mode: `@hot_reload` is a compile error (no runtime patching).

---

## 23. Information Flow Types

Secrets cannot leak. The type system enforces it — not access control lists, not audits.

```ng
// flow labels (in ng.flow)
label Public   // can go anywhere
label Secret   // can never flow to Public context
label Trusted  // can flow to Secret or Public, but not from Public

// tainted values
fn get_password() -> Secret[str] { ... }
fn get_api_key()  -> Secret[str] { ... }

fn log(msg: Public[str]) ! {IO} { IO.print(msg) }

fn bad() {
    let pass = get_password()
    log(pass)           // COMPILE ERROR: Secret cannot flow to Public
    log(pass.hash())    // OK only if hash() is: Secret[str] -> Public[str]
                        //   (a declassification function, explicitly approved)
}

// declassification: explicit, auditable
@declassify(Secret -> Public)
fn safe_hash(s: Secret[str]) -> Public[str] {
    Public(blake3(s.inner()))
}

// database query results inherit taint from inputs
fn query(db: &DB, id: Secret[UserId]) -> Secret[UserRecord] { ... }
```

The compiler tracks taint transitively through all operations.
No secret ever reaches a log, network socket, or file unless explicitly declassified.

---

## 24. Temporal Types

Values have lifetimes in wall-clock time, not just program scope.

```ng
// value expires after 1 hour
fn authenticate(creds: Credentials) -> Token @ttl(3600s) ! {IO, Fail[AuthError]} {
    // ...
}

// temporal type in the type system
type Token = {
    value: str,
    issued: Timestamp,
    ttl:   Duration,
} where now() < issued + ttl   // invariant: token is not expired

fn use_token(t: Token) -> Response ! {IO} {
    // compiler inserts: if token is expired, Fail[TokenExpired]
    // no manual expiry check needed
}

// short-lived keys
fn generate_otp() -> OtpCode @ttl(30s) { ... }

let code = generate_otp()
// ... 31 seconds later ...
verify(code)   // COMPILE ERROR if code is provably expired
               // RUNTIME check if expiry is unknown at compile time
```

---

## 25. Distributed Types

Values on remote machines are typed identically to local values.

```ng
// Remote[T, "location"] — T living on a specific node
type Remote[T, comptime loc: str] = ...

// connect to a remote service
let db: Remote[Database, "db-01.cluster"] = cluster.connect("db-01")

// call remote methods — compiler generates: serialize args, network call,
//                        deserialize result, handle failures
let user: User = db.get_user(id)!   // ! = propagate network error

// location-transparent channels
let (tx, rx) = DistChannel[Message].new()
spawn_remote("worker-03", || {
    tx.send(compute())
})
let result = rx.recv()!

// affine types for distributed resources (moved, not copied, across nodes)
fn transfer(account: Remote[BankAccount, "node-a"]) -> Remote[BankAccount, "node-b"] {
    // compiler ensures account is consumed (not duplicated) during transfer
}
```

Network failures are `Fail[NetError]` effects — visible in every function signature that crosses node boundaries.

---

## 26. Proof Synthesis

The compiler calls Z3 to prove function contracts. Not optional annotations — real proofs.

```ng
// postcondition: compiler proves this holds for all inputs, or rejects
fn sqrt(x: f64) -> f64
    requires x >= 0.0
    ensures  (result * result - x).abs() < 1e-10
{
    // ... implementation ...
}

// loop invariant
fn binary_search[T: Ord](arr: &[T], target: &T) -> Option[usize]
    ensures match result {
        Some(i) => arr[i] == *target,
        None    => arr.all(|x| x != *target),
    }
{
    // compiler verifies the postcondition is provable from the implementation
}

// struct invariant (always holds)
type SortedVec[T: Ord] = {
    inner: Vec[T],
    inv always: inner.windows(2).all(|w| w[0] <= w[1])
}

// when proof fails, compiler shows the counterexample:
// error: cannot prove `result >= 0` for input x = -1.0
//        counterexample: x = -1.0 → result = NaN
```

For cases where Z3 cannot decide automatically, the programmer adds lemmas.
Proofs are cached — not recomputed on every build.

---

## 27. Reversible Functions

Pure functions have compiler-generated inverses.

```ng
// mark as reversible — compiler generates the inverse automatically
@reversible
fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

// compiler generates:
fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

// reversible codec — encode AND decode from one definition
@reversible
fn encode_varint(n: u64) -> []u8 { ... }
// compiler generates: decode_varint(bytes: []u8) -> Result[u64, DecodeError]

// reversible encryption (both directions explicit but derived from one source)
@reversible
fn xor_cipher(data: []u8, key: []u8) -> []u8 {
    data.zip(key.cycle()).map(|(b, k)| b ^ k).collect()
}
// xor_cipher.inverse == xor_cipher  (self-inverse — compiler detects this)

// not everything is reversible:
// @reversible fn hash(data: []u8) -> [32]u8 { blake3(data) }
// COMPILE ERROR: hash is not bijective — cannot generate inverse
```

---

## 28. Live Invariants

Invariants expressed in the type system, checked at compile time and monitored at runtime.

```ng
// field invariant — always holds
type BankAccount = {
    owner:   str,
    balance: i64,
    inv always: balance >= 0,   // compile time: all writes to balance are checked
                                // debug mode:   runtime assertion on every mutation
                                // release mode: zero-cost (proven statically)
}

// operation that would violate invariant: COMPILE ERROR
fn overdraft(acc: &mut BankAccount, amount: i64) {
    acc.balance -= amount   // ERROR if compiler cannot prove balance >= 0 after
}

// correct version:
fn withdraw(acc: &mut BankAccount, amount: i64) -> Result[(), InsufficientFunds] {
    if acc.balance < amount { return Err(InsufficientFunds) }
    acc.balance -= amount   // OK: compiler proves balance >= 0 (since amount <= balance)
    Ok(())
}

// global system invariant
inv system: total_supply == issued_tokens.sum()  // checked on every state transition
```

---

## 29. Causal Types

Every value knows its provenance. Automatic audit trails, zero manual logging.

```ng
// values carry their causal origin
let price: Caused[f64, from: "db.prices"] = db.get("AAPL")
let factor: Caused[f64, from: "user.input"] = parse_input()?

// derived values inherit causality
let adjusted = price * factor
// adjusted: Caused[f64, from: {"db.prices", "user.input"}]

// audit: the compiler can generate a full data lineage graph
@causal
fn compute_tax(income: Caused[f64, from: "payroll"]) -> Caused[f64, from: "payroll"] {
    income * 0.2
}

// causal types prevent silent data corruption:
fn report(value: Caused[f64, from: "db.prices"]) -> str { ... }
report(adjusted)  // ERROR: adjusted also comes from "user.input" —
                  //        cannot use user input in a price-only report
                  //        without explicit acknowledgement

// explicit acknowledgement (audited):
@acknowledge_source("user.input")
fn override_price(v: Caused[f64, from: {"db.prices", "user.input"}])
    -> Caused[f64, from: "db.prices"] { ... }
```

Causal types are zero-cost at runtime — all tracking is compile-time metadata.
The compiler generates an audit report as a build artifact.

---

*ng — 0 dependencies. 0 GC. 0 undefined behavior. 0 secret leaks. Written in itself.*
