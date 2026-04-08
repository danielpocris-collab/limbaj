# ng — Real Programs

Programs written to validate the language design.
Each program stresses a different part of the spec.

---

## 1. Hello World

```ng
import ng.io

fn main() ! {IO} {
    IO.println("Hello, world!")
}
```

---

## 2. Fibonacci — Recursive and Iterative

```ng
// recursive — clean, but stack-grows
fn fib_rec(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fib_rec(n - 1) + fib_rec(n - 2),
    }
}

// iterative — O(n) time, O(1) space
fn fib_iter(n: u64) -> u64 {
    if n == 0 { return 0 }
    var a: u64 = 0
    var b: u64 = 1
    for _ in 1..n {
        let t = a + b
        a = b
        b = t
    }
    b
}

fn main() ! {IO} {
    for i in 0..10 {
        IO.println(fib_iter(i))
    }
}
```

---

## 3. Generic Stack

```ng
// ng.collections.Vec exists but let's write our own
// to stress-test generics + ownership

type Stack[T] = {
    data: Vec[T],
}

impl Stack[T] {
    fn new() -> Stack[T] {
        Stack { data: Vec.new() }
    }

    fn push(self: &mut Stack[T], val: T) {
        self.data.push(val)
    }

    fn pop(self: &mut Stack[T]) -> Option[T] {
        self.data.pop()
    }

    fn peek(self: &Stack[T]) -> Option[&T] {
        let len = self.data.len
        if len == 0 { None }
        else { Some(&self.data[len - 1]) }
    }

    fn is_empty(self: &Stack[T]) -> bool {
        self.data.len == 0
    }
}

fn main() ! {IO} {
    var s = Stack[i32].new()

    s.push(10)
    s.push(20)
    s.push(30)

    while !s.is_empty() {
        match s.pop() {
            Some(v) => IO.println(v),
            None    => {},
        }
    }
    // prints: 30, 20, 10
}
```

---

## 4. Error Handling — File Parser

```ng
import ng.io
import ng.str

enum ConfigError {
    FileNotFound(str),
    MissingKey(str),
    InvalidValue { key: str, got: str },
}

type Config = {
    host: str,
    port: u16,
}

fn parse_config(text: str) -> Result[Config, ConfigError] {
    let lines = text.split('\n')

    let host = find_key(lines, "host")
        .ok_or(ConfigError.MissingKey("host"))?

    let port_str = find_key(lines, "port")
        .ok_or(ConfigError.MissingKey("port"))?

    let port = port_str.parse_u16()
        .map_err(|_| ConfigError.InvalidValue {
            key: "port",
            got: port_str,
        })?

    Ok(Config { host, port })
}

fn find_key(lines: []str, key: str) -> Option[str] {
    for line in lines {
        let parts = line.split_once('=')
        match parts {
            Some((k, v)) if k.trim() == key => return Some(v.trim()),
            _ => {},
        }
    }
    None
}

fn main() ! {IO} {
    match IO.read_file("config.ng") {
        Err(_) => IO.println("config.ng not found, using defaults"),
        Ok(text) => match parse_config(text) {
            Ok(cfg)  => IO.println("host=" + cfg.host + " port=" + cfg.port.to_str()),
            Err(ConfigError.MissingKey(k))         => IO.println("missing key: " + k),
            Err(ConfigError.InvalidValue { key, got }) =>
                IO.println("bad value for " + key + ": " + got),
            Err(ConfigError.FileNotFound(p))       => IO.println("not found: " + p),
        },
    }
}
```

---

## 5. Algebraic Effects — Custom Logger

```ng
// Define the effect
effect Log {
    fn info(msg: str)  -> unit
    fn warn(msg: str)  -> unit
    fn error(msg: str) -> unit
}

// Business logic — doesn't know where logs go
fn process_items(items: []i32) ! {Log} -> i32 {
    var sum = 0
    for item in items {
        if item < 0 {
            Log.warn("negative item: " + item.to_str())
        } else {
            sum += item
            Log.info("added " + item.to_str() + ", running sum: " + sum.to_str())
        }
    }
    sum
}

// Handler 1: print to stdout
fn run_with_stdout_log(items: []i32) ! {IO} -> i32 {
    handle process_items(items) with {
        Log.info(msg)  => { IO.println("[INFO] " + msg);  resume unit }
        Log.warn(msg)  => { IO.println("[WARN] " + msg);  resume unit }
        Log.error(msg) => { IO.println("[ERROR] " + msg); resume unit }
    }
}

// Handler 2: silent (discard logs)
fn run_silent(items: []i32) -> i32 {
    handle process_items(items) with {
        Log.info(_)  => resume unit
        Log.warn(_)  => resume unit
        Log.error(_) => resume unit
    }
}

fn main() ! {IO} {
    let result = run_with_stdout_log([1, -2, 3, 4, -5])
    IO.println("result: " + result.to_str())
}
```

---

## 6. Ownership — Buffer without Leaks

```ng
// Shows: ownership transfer, borrow, no GC needed

type Buffer = {
    data: *mut u8,
    len:  usize,
    cap:  usize,
}

impl Buffer {
    fn alloc(cap: usize) -> Buffer {
        let data = unsafe { ng.mem.alloc(cap) }
        Buffer { data, len: 0, cap }
    }

    // takes ownership of self — caller can't use it after
    fn free(self: Buffer) {
        unsafe { ng.mem.free(self.data) }
        // self is dropped here, no double-free possible
    }

    fn write(self: &mut Buffer, bytes: []u8) -> Result[unit, unit] {
        if self.len + bytes.len > self.cap { return Err(unit) }
        unsafe {
            ng.mem.copy(self.data.offset(self.len), bytes.ptr, bytes.len)
        }
        self.len += bytes.len
        Ok(unit)
    }

    fn as_slice(self: &Buffer) -> []u8 {
        unsafe { ng.mem.slice(self.data, self.len) }
    }
}

fn main() ! {IO} {
    var buf = Buffer.alloc(256)

    buf.write("Hello ".as_bytes()).unwrap()
    buf.write("world!".as_bytes()).unwrap()

    IO.println(str.from_utf8(buf.as_slice()).unwrap())

    buf.free()    // explicit free — ownership consumed, compiler verifies
    // buf is no longer accessible here — compile error if used
}
```

---

## 7. Concurrency — Producer / Consumer

```ng
import ng.sync { Channel }
import ng.io

fn producer(tx: Channel[i32].Sender) ! {Async} {
    for i in 0..100 {
        tx.send(i * i)
    }
    tx.close()
}

fn consumer(rx: Channel[i32].Receiver) ! {Async, IO} -> i64 {
    var sum: i64 = 0
    for val in rx {
        sum += val as i64
    }
    IO.println("sum of squares 0..99 = " + sum.to_str())
    sum
}

fn main() ! {Async, IO} {
    let (tx, rx) = Channel[i32].new()

    // spawn returns a handle — structured, tied to current scope
    let prod = spawn producer(tx)
    let cons = spawn consumer(rx)

    prod.await
    let result = cons.await
    IO.println("done: " + result.to_str())
}
```

---

## 8. Comptime — Type-Safe Units

```ng
// Encode units in types at compile time — zero runtime cost

type Meters  = (f64)
type Seconds = (f64)
type MPS     = (f64)   // meters per second

impl Meters {
    fn value(self: Meters) -> f64 { self.0 }
}
impl Seconds {
    fn value(self: Seconds) -> f64 { self.0 }
}

fn speed(dist: Meters, time: Seconds) -> MPS {
    MPS(dist.value() / time.value())
}

// compile-time dimension check — if you pass (Seconds, Meters) it fails
fn main() ! {IO} {
    let d = Meters(100.0)
    let t = Seconds(9.58)
    let v = speed(d, t)
    IO.println("speed = " + v.0.to_str() + " m/s")
    // compile error if you write speed(t, d) — types don't match
}
```

---

## 9. Kernel / Bare Metal — No Alloc

```ng
// @mode(static) — zero heap, zero runtime
// This could run as a kernel entry point or ISR

@mode(static)
module kernel.entry

extern "c" {
    fn outb(port: u16, val: u8)
    fn inb(port: u16) -> u8
}

// VGA text mode buffer (direct memory write)
const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16

fn vga_write(row: u8, col: u8, ch: char, color: u8) {
    let idx = (row as usize) * 80 + (col as usize)
    let entry = ((color as u16) << 8) | (ch as u16)
    unsafe { *VGA_BUFFER.offset(idx) = entry }
}

fn vga_print(row: u8, msg: str, color: u8) {
    for (i, ch) in msg.chars().enumerate() {
        vga_write(row, i as u8, ch, color)
    }
}

@export
fn kernel_main() {
    vga_print(0, "ng kernel booting...", 0x0F)
    vga_print(1, "no GC. no runtime. no undefined behavior.", 0x0A)
    loop {}
}
```

---

## 10. Trait Objects — Plugin System

```ng
// When you need runtime dispatch (rare in ng)

trait Renderer {
    fn render(self: &Self, scene: &Scene) -> Frame
    fn name(self: &Self) -> str
}

type SoftwareRenderer = { width: u32, height: u32 }
type GpuRenderer      = { device: GpuDevice }

impl Renderer for SoftwareRenderer {
    fn render(self: &SoftwareRenderer, scene: &Scene) -> Frame {
        // CPU rasterization
        software_rasterize(scene, self.width, self.height)
    }
    fn name(self: &SoftwareRenderer) -> str { "software" }
}

impl Renderer for GpuRenderer {
    fn render(self: &GpuRenderer, scene: &Scene) -> Frame {
        // GPU pipeline
        gpu_draw(self.device, scene)
    }
    fn name(self: &GpuRenderer) -> str { "gpu" }
}

// dyn Renderer — explicit boxing, visible cost
fn pick_renderer(use_gpu: bool) -> Box[dyn Renderer] {
    if use_gpu {
        Box.new(GpuRenderer { device: GpuDevice.init() })
    } else {
        Box.new(SoftwareRenderer { width: 1920, height: 1080 })
    }
}

fn main() ! {IO} {
    let renderer = pick_renderer(false)
    IO.println("using renderer: " + renderer.name())
    let frame = renderer.render(&scene)
    display(frame)
}
```

---

## Design Issues Found While Writing Programs

Writing these programs revealed 3 points that need refinement in the spec:

### Issue 1: Effect propagation syntax
When a function calls another function with effects, does it auto-propagate or must you annotate?

**Decision:** Auto-propagate (like Koka). If you call a function with `! {IO}`, your function implicitly has `! {IO}` unless you handle it.

```ng
// This works — IO propagates automatically
fn greet(name: str) {           // no annotation needed
    IO.println("Hello, " + name)  // IO propagates up
}
```

### Issue 2: `unit` vs `()`
The spec says `unit` for zero-size type. But `()` is more natural in match arms.

**Decision:** Both are the same type. `unit` is the name, `()` is the literal.

```ng
fn nothing() -> unit { () }
fn also_nothing() -> () { unit }  // same thing
```

### Issue 3: Range syntax `0..n` vs `0..=n`
Need both exclusive and inclusive ranges.

**Decision:**
- `0..n`   → exclusive end (0, 1, ..., n-1) — most common
- `0..=n`  → inclusive end (0, 1, ..., n)
- `..n`    → from beginning
- `n..`    → to end (for slices)
