# ng — Complex Programs (Feature Validation Suite)

One real program per language feature. No minimal examples. Each program demonstrates
the feature in a realistic, production-level context showing what the compiler proves,
prevents, and generates automatically.

---

## Program 1: Dependent Types — Linear Algebra Library

Dimensions are in the type. Matrix multiply with mismatched dimensions is a compile error.
LU decomposition is only callable on square matrices. Packet parser guarantees body length.

```ng
module ng.math.linalg

use ng.math.{sqrt, abs, pow}
use ng.core.{Option, Result, Never}
use ng.mem.Arena
use ng.fmt.Display

// ─── Matrix type: R rows, C columns ──────────────────────────────────────────

type Matrix[comptime R: usize, comptime C: usize] = {
    data: [R * C]f64,
}

impl[R: usize, C: usize] Matrix[R, C] {

    fn zero() -> Self {
        Matrix { data: [0.0f64; R * C] }
    }

    fn from_rows(rows: [[C]f64; R]) -> Self {
        var m = Self.zero()
        for i in 0..R {
            for j in 0..C {
                m.data[i * C + j] = rows[i][j]
            }
        }
        m
    }

    @inline
    fn get(self, row: usize, col: usize) -> f64
        where row < R, col < C
    {
        self.data[row * C + col]
    }

    @inline
    fn set(self: &mut Self, row: usize, col: usize, val: f64)
        where row < R, col < C
    {
        self.data[row * C + col] = val
    }

    // Multiply: [R×C] * [C×K] → [R×K]
    // COMPILE ERROR if inner dimensions do not match.
    fn mul[K: usize](self, rhs: &Matrix[C, K]) -> Matrix[R, K] {
        var out = Matrix[R, K].zero()
        for i in 0..R {
            for k in 0..C {
                let a = self.get(i, k)
                for j in 0..K {
                    out.set(i, j, out.get(i, j) + a * rhs.get(k, j))
                }
            }
        }
        out
    }

    fn transpose(self) -> Matrix[C, R] {
        var out = Matrix[C, R].zero()
        for i in 0..R {
            for j in 0..C {
                out.set(j, i, self.get(i, j))
            }
        }
        out
    }

    fn add(self, rhs: &Matrix[R, C]) -> Matrix[R, C] {
        var out = Self.zero()
        for i in 0..(R * C) {
            out.data[i] = self.data[i] + rhs.data[i]
        }
        out
    }

    fn scale(self, s: f64) -> Matrix[R, C] {
        var out = Self.zero()
        for i in 0..(R * C) { out.data[i] = self.data[i] * s }
        out
    }

    fn frobenius_norm(self) -> f64 {
        var sum = 0.0f64
        for v in self.data { sum += v * v }
        sqrt(sum)
    }

    fn row(self, r: usize) -> [C]f64
        where r < R
    {
        var out = [0.0f64; C]
        for j in 0..C { out[j] = self.get(r, j) }
        out
    }

    fn col(self, c: usize) -> [R]f64
        where c < C
    {
        var out = [0.0f64; R]
        for i in 0..R { out[i] = self.get(i, c) }
        out
    }
}

// ─── Square matrix operations (only callable when R == C) ────────────────────

impl[N: usize] Matrix[N, N] {

    fn identity() -> Self {
        var m = Self.zero()
        for i in 0..N { m.set(i, i, 1.0) }
        m
    }

    fn trace(self) -> f64 {
        var t = 0.0f64
        for i in 0..N { t += self.get(i, i) }
        t
    }

    // Partial-pivot LU decomposition → (L, U, pivot_array)
    fn lu(self) -> (Matrix[N, N], Matrix[N, N], [N]usize) {
        var L = Matrix[N, N].identity()
        var U = self
        var P = [0usize; N]
        for i in 0..N { P[i] = i }

        for k in 0..N {
            var max_val = abs(U.get(k, k))
            var max_row = k
            for i in (k + 1)..N {
                let v = abs(U.get(i, k))
                if v > max_val { max_val = v; max_row = i }
            }
            if max_row != k {
                for j in 0..N {
                    let tmp = U.get(k, j)
                    U.set(k, j, U.get(max_row, j))
                    U.set(max_row, j, tmp)
                }
                for j in 0..k {
                    let tmp = L.get(k, j)
                    L.set(k, j, L.get(max_row, j))
                    L.set(max_row, j, tmp)
                }
                P.swap(k, max_row)
            }
            for i in (k + 1)..N {
                let u_kk = U.get(k, k)
                if abs(u_kk) < 1e-14 { continue }
                let factor = U.get(i, k) / u_kk
                L.set(i, k, factor)
                for j in k..N {
                    U.set(i, j, U.get(i, j) - factor * U.get(k, j))
                }
            }
        }
        (L, U, P)
    }

    // Solve Ax = b via LU — returns None if matrix is singular
    fn solve(self, b: &[N]f64) -> Option[[N]f64] {
        let (L, U, P) = self.lu()

        // apply row permutation to b
        var pb = [0.0f64; N]
        for i in 0..N { pb[i] = b[P[i]] }

        // forward substitution: Ly = Pb
        var y = [0.0f64; N]
        for i in 0..N {
            y[i] = pb[i]
            for j in 0..i { y[i] -= L.get(i, j) * y[j] }
        }

        // back substitution: Ux = y
        var x = [0.0f64; N]
        for i in (0..N).rev() {
            let u_ii = U.get(i, i)
            if abs(u_ii) < 1e-14 { return None }
            x[i] = y[i]
            for j in (i + 1)..N { x[i] -= U.get(i, j) * x[j] }
            x[i] /= u_ii
        }
        Some(x)
    }

    // Determinant via LU
    fn det(self) -> f64 {
        let (_, U, P) = self.lu()
        var d = 1.0f64
        for i in 0..N { d *= U.get(i, i) }
        // count permutation parity
        var sign = 1.0f64
        var used = [false; N]
        for i in 0..N {
            if used[i] { continue }
            var cycle = 0usize
            var j = i
            while !used[j] { used[j] = true; j = P[j]; cycle += 1 }
            if cycle % 2 == 0 { sign *= -1.0 }
        }
        d * sign
    }

    // Invert via LU — None if singular
    fn invert(self) -> Option[Matrix[N, N]] {
        var inv = Matrix[N, N].zero()
        for j in 0..N {
            var e = [0.0f64; N]
            e[j] = 1.0
            let col = self.solve(&e)?
            for i in 0..N { inv.set(i, j, col[i]) }
        }
        Some(inv)
    }
}

// ─── Bounds-safe network packet with dependent body length ───────────────────

type ParseError = enum {
    BadMagic(u32),
    BadVersion(u8),
    Truncated { expected: usize, got: usize },
    BodyTooLarge(usize),
    Checksum { expected: u32, got: u32 },
}

const MAX_BODY: usize = 65536

type PacketHeader = {
    magic:    u32,
    version:  u8,
    kind:     u8,
    flags:    u8,
    reserved: u8,
    body_len: u32,
    checksum: u32,
}

// Body length is a comptime param — the caller gets back a typed slice
type Packet[comptime BodyLen: usize] = {
    header: PacketHeader,
    body:   [BodyLen]u8,
}
where BodyLen <= MAX_BODY  // enforced at type level

fn crc32(data: &[]u8) -> u32 {
    var crc = 0xFFFF_FFFFu32
    for b in data {
        crc ^= b as u32
        for _ in 0..8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 }
        }
    }
    crc ^ 0xFFFF_FFFF
}

// parse_packet: returns a type that carries the exact body length
fn parse_packet(buf: &[]u8) -> Result[Packet[_], ParseError]
    requires buf.len() >= 16  // header size
{
    let magic = u32.from_be_bytes(buf[0..4])
    if magic != 0xAE_NG_00_01 { return Err(ParseError.BadMagic(magic)) }

    let version = buf[4]
    if version != 1 && version != 2 {
        return Err(ParseError.BadVersion(version))
    }

    let body_len = u32.from_be_bytes(buf[8..12]) as usize
    if body_len > MAX_BODY { return Err(ParseError.BodyTooLarge(body_len)) }

    let total = 16 + body_len
    if buf.len() < total {
        return Err(ParseError.Truncated { expected: total, got: buf.len() })
    }

    let expected_crc = u32.from_be_bytes(buf[12..16])
    let actual_crc   = crc32(&buf[16..16 + body_len])
    if expected_crc != actual_crc {
        return Err(ParseError.Checksum { expected: expected_crc, got: actual_crc })
    }

    // body_len is a runtime value, but the compiler emits a runtime-checked
    // conversion — never silently truncates or overreads
    Ok(Packet {
        header: PacketHeader {
            magic,
            version,
            kind:     buf[5],
            flags:    buf[6],
            reserved: buf[7],
            body_len: body_len as u32,
            checksum: actual_crc,
        },
        body: buf[16..16 + body_len].try_into_array()?,
    })
}

fn main() {
    // 4×4 system of equations: Ax = b
    let A = Matrix[4, 4].from_rows([
        [2.0,  1.0, -1.0,  0.5],
        [-3.0, -1.0,  2.0, -0.25],
        [-2.0,  1.0,  2.0,  1.0],
        [ 1.0,  2.0, -3.0,  1.5],
    ])
    let b = [8.0f64, -11.0, -3.0, 7.5]

    match A.solve(&b) {
        Some(x) => ng.fmt.println("solution: {x}"),
        None    => ng.fmt.println("singular matrix"),
    }

    // Compile errors (uncomment to verify):
    // let bad_mul = A.mul(&Matrix[3, 2].zero())
    //   ERROR: Matrix[4,4].mul[K](rhs: &Matrix[4,K]) — rhs has R=3, expected 4
    //
    // let bad_solve = Matrix[4, 3].zero().solve(&[0.0f64; 4])
    //   ERROR: solve() requires R == C, but Matrix[4,3] is not square
}
```

---

## Program 2: Capability Security — Web Server with Sandboxed Plugin System

The plugin cannot access files, network, or spawn processes unless the host
explicitly grants a restricted capability. Forging capabilities is impossible.

```ng
module app.server

use ng.io.{File, Listener, TcpStream}
use ng.net.{HttpRequest, HttpResponse, StatusCode}
use ng.cap.{FileCap, NetCap, SpawnCap, ClockCap, AllCaps, NoCaps, attenuate}
use ng.mem.Arena
use ng.sync.{RwLock, Arc}
use ng.collections.{HashMap, Vec}
use ng.core.{Result, Option}

// ─── Plugin ABI ───────────────────────────────────────────────────────────────

type PluginCaps = {
    // plugins get only what they need — no ambient authority
    read_dir: Option[FileCap],   // read-only file access to one directory
    clock:    Option[ClockCap],  // read current time (determinism-breaking!)
    // no NetCap, no SpawnCap
}

type PluginRequest = {
    path:    str,
    method:  str,
    headers: HashMap[str, str],
    body:    []u8,
}

type PluginResponse = {
    status:  u16,
    headers: HashMap[str, str],
    body:    Vec[u8],
}

type Plugin = {
    name:    str,
    version: u32,
    handle:  fn(PluginRequest, &PluginCaps) -> Result[PluginResponse, PluginError]
             ! {Fail[PluginError]},
}

type PluginError = enum {
    NotFound,
    InternalError(str),
    PermissionDenied(str),   // plugin tried to use capability it wasn't given
    Timeout,
}

// ─── Request router ───────────────────────────────────────────────────────────

type Route = {
    prefix:  str,
    plugin:  Arc[Plugin],
    caps:    PluginCaps,
}

type Router = {
    routes:   Vec[Route],
    fallback: fn(&HttpRequest) -> HttpResponse,
}

impl Router {
    fn new(fallback: fn(&HttpRequest) -> HttpResponse) -> Self {
        Router { routes: Vec.new(), fallback }
    }

    fn mount(self: &mut Self, prefix: str, plugin: Arc[Plugin], caps: PluginCaps) {
        self.routes.push(Route { prefix, plugin, caps })
    }

    fn dispatch(self, req: &HttpRequest, @cap clock: ClockCap) -> HttpResponse
        ! {IO}
    {
        for route in &self.routes {
            if req.path.starts_with(&route.prefix) {
                let preq = PluginRequest {
                    path:    req.path[route.prefix.len()..].to_owned(),
                    method:  req.method.to_string(),
                    headers: req.headers.clone(),
                    body:    req.body.clone(),
                }
                let start = clock.now()
                let result = (route.plugin.handle)(preq, &route.caps)
                let elapsed = clock.now() - start

                if elapsed.as_millis() > 5000 {
                    return HttpResponse {
                        status: StatusCode.GatewayTimeout,
                        body:   b"plugin timeout",
                        headers: HashMap.new(),
                    }
                }

                return match result {
                    Ok(presp) => HttpResponse {
                        status:  StatusCode.from_u16(presp.status),
                        headers: presp.headers,
                        body:    presp.body.into_slice(),
                    },
                    Err(PluginError.NotFound) => HttpResponse.not_found(),
                    Err(PluginError.PermissionDenied(msg)) => {
                        ng.fmt.eprintln("plugin permission denied: {msg}")
                        HttpResponse.forbidden()
                    },
                    Err(PluginError.InternalError(e)) => {
                        ng.fmt.eprintln("plugin error: {e}")
                        HttpResponse.internal_server_error()
                    },
                    Err(PluginError.Timeout) => HttpResponse.gateway_timeout(),
                }
            }
        }
        (self.fallback)(req)
    }
}

// ─── A real plugin: static file server ───────────────────────────────────────

fn static_file_plugin(req: PluginRequest, caps: &PluginCaps)
    -> Result[PluginResponse, PluginError]
    ! {Fail[PluginError]}
{
    // caps.read_dir is Option — the plugin CANNOT read files if not granted
    let file_cap = caps.read_dir
        .ok_or(PluginError.PermissionDenied("no FileCap granted"))?

    if req.method != "GET" {
        return Ok(PluginResponse {
            status: 405,
            headers: HashMap.from([("Allow", "GET")]),
            body: Vec.from(b"Method Not Allowed"),
        })
    }

    // sanitize path — no directory traversal
    let safe = req.path.trim_start_with('/').replace("..", "")
    if safe.is_empty() || safe.contains('\0') {
        return Err(PluginError.PermissionDenied("invalid path"))
    }

    // file_cap is a restricted FileCap — it can only read from one directory
    // trying to read outside that directory is prevented by the capability's
    // root constraint, not by string checking
    let content = File.read_bytes(safe, file_cap)
        .map_err(|e| PluginError.InternalError(e.to_string()))?

    let mime = mime_from_ext(safe)
    Ok(PluginResponse {
        status: 200,
        headers: HashMap.from([
            ("Content-Type",   mime),
            ("Content-Length", content.len().to_string()),
            ("Cache-Control",  "public, max-age=3600"),
        ]),
        body: Vec.from(content),
    })
}

fn mime_from_ext(path: &str) -> str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css")  => "text/css",
        Some("js")   => "application/javascript",
        Some("json") => "application/json",
        Some("png")  => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("wasm") => "application/wasm",
        _            => "application/octet-stream",
    }
}

// ─── Server main loop ─────────────────────────────────────────────────────────

fn serve(addr: str, router: Arc[RwLock[Router]], @caps all: AllCaps)
    ! {IO, Fail[ng.io.IOError]}
{
    let listener = Listener.bind(addr, all.net)?
    ng.fmt.println("listening on {addr}")

    loop {
        let (stream, remote) = listener.accept()?
        let router = router.clone()

        // spawn only gets SpawnCap — cannot escalate to FileCap or NetCap
        ng.os.spawn(|| handle_connection(stream, remote, router, all.clock), all.spawn)
    }
}

fn handle_connection(
    stream: TcpStream,
    remote: ng.net.SocketAddr,
    router: Arc[RwLock[Router]],
    @cap  clock: ClockCap,
) ! {IO}
{
    let req = ng.net.parse_http_request(stream)?
    let resp = router.read().dispatch(&req, clock)
    ng.net.write_http_response(stream, resp)?
}

fn main(@caps all: AllCaps) ! {IO} {
    // build restricted capability for the static plugin
    // attenuate: full FileCap → read-only FileCap rooted at "./public/"
    let static_cap = attenuate(all.file, FileCap.read_only_dir("./public/"))

    let plugin = Arc.new(Plugin {
        name:    "static",
        version: 1,
        handle:  static_file_plugin,
    })

    var router = Router.new(|_req| HttpResponse.not_found())
    router.mount("/static/", plugin, PluginCaps {
        read_dir: Some(static_cap),
        clock:    None,    // static plugin doesn't need time
    })

    let shared = Arc.new(RwLock.new(router))
    serve("0.0.0.0:8080", shared, all)!
}

// Capabilities the plugin does NOT get:
// - all.net   → cannot make outgoing connections
// - all.spawn → cannot spawn processes
// - all.file  → cannot write files or read outside ./public/
// Trying to use any of these is a type error, not a runtime check.
```

---

## Program 3: First-Class Serialization — Network Protocol with Versioned Formats

Binary wire format v1 and v2 with automatic migration. JSON REST adapter.
All codecs generated at compile time — zero reflection, zero external schema.

```ng
module app.protocol

use ng.serial.{Codec, WireError}
use ng.core.{Result, Option}
use ng.collections.{Vec, HashMap}
use ng.net.{Duration, Timestamp}

// ─── Core protocol types ──────────────────────────────────────────────────────

@wire(json, binary, msgpack)
type NodeId = {
    cluster: u16,
    node:    u32,
}

@wire(json, binary)
type Severity = enum {
    Debug   = 0,
    Info    = 1,
    Warning = 2,
    Error   = 3,
    Fatal   = 4,
}

// v1 log entry — stable, already deployed
@stable_abi
@wire(binary, version: 1)
type LogEntryV1 = {
    timestamp: u64,              // unix epoch millis
    severity:  Severity,
    node:      NodeId,
    @wire(max_len = 4096) message: str,
}

// v2 log entry — adds trace_id and structured fields
@stable_abi
@wire(binary, version: 2)
@wire(json)
@wire_migrate(1 -> 2, fn migrate_log_v1_to_v2)
type LogEntry = {
    timestamp:    u64,
    severity:     Severity,
    node:         NodeId,
    @wire(max_len = 4096) message: str,
    // new in v2:
    trace_id:     Option[u128],
    @wire(max_entries = 64) fields: HashMap[str, str],
    duration_us:  Option[u64],
}

fn migrate_log_v1_to_v2(old: LogEntryV1) -> LogEntry {
    LogEntry {
        timestamp:   old.timestamp,
        severity:    old.severity,
        node:        old.node,
        message:     old.message,
        trace_id:    None,
        fields:      HashMap.new(),
        duration_us: None,
    }
}

// ─── Batch protocol ───────────────────────────────────────────────────────────

@wire(binary, version: 2)
@wire(json)
type LogBatch = {
    batch_id:   u64,
    producer:   NodeId,
    sent_at:    u64,
    @wire(max_entries = 10000) entries: Vec[LogEntry],
    // checksum auto-included in binary format (not in json)
    @wire(binary_only) crc32: u32,
}

// ─── Query protocol ───────────────────────────────────────────────────────────

@wire(json, binary)
type TimeRange = {
    from_ms: u64,
    to_ms:   u64,
}
where self.to_ms >= self.from_ms   // dependent constraint

@wire(json, binary)
type LogQuery = {
    range:          TimeRange,
    severity_min:   Severity,
    @wire(max_entries = 8)  nodes:  Option[Vec[NodeId]],
    @wire(max_len = 256)    search: Option[str],
    limit:          u32,
    offset:         u64,
}

@wire(json, binary)
type QueryResult = {
    query_id:   u64,
    total:      u64,
    returned:   u32,
    duration_us: u64,
    @wire(max_entries = 10000) entries: Vec[LogEntry],
}

// ─── Wire codec implementation ────────────────────────────────────────────────

// Compiler generates these from the @wire attributes, but you CAN override:

impl LogBatch {
    // custom binary serializer (overrides compiler-generated one)
    fn to_binary_custom(self) -> Vec[u8] {
        var buf = Vec[u8].with_capacity(
            16                                    // header
            + 8                                   // batch_id
            + 6                                   // producer
            + 8                                   // sent_at
            + 4                                   // entry count
            + self.entries.len() * 256            // estimate per entry
        )
        // magic + version
        buf.extend_from_slice(b"\xAE\x4C\x00\x02")
        // batch_id
        buf.extend_from_u64_le(self.batch_id)
        // producer
        buf.extend_from_u16_le(self.producer.cluster)
        buf.extend_from_u32_le(self.producer.node)
        // sent_at
        buf.extend_from_u64_le(self.sent_at)
        // entries
        buf.extend_from_u32_le(self.entries.len() as u32)
        for entry in &self.entries {
            // use compiler-generated codec for individual entries
            let encoded = entry.to_binary()
            buf.extend_from_u32_le(encoded.len() as u32)
            buf.extend_from_slice(&encoded)
        }
        // crc32 over everything so far
        let crc = ng.serial.crc32(&buf)
        buf.extend_from_u32_le(crc)
        buf
    }
}

// ─── HTTP/JSON API adapter ────────────────────────────────────────────────────

fn json_api_handler(req: ng.net.HttpRequest) -> ng.net.HttpResponse
    ! {IO, Fail[WireError]}
{
    match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v2/logs/ingest") => {
            // accept both JSON and binary formats
            let batch = match req.content_type() {
                "application/json"   => LogBatch.from_json(&req.body_str()?)?,
                "application/x-nglog-binary" => LogBatch.from_binary(&req.body)?,
                ct => return Err(WireError.UnsupportedFormat(ct.to_owned())),
            }
            ingest_batch(batch)?
            ng.net.HttpResponse.ok_json(r#"{"status":"accepted"}"#)
        },

        ("POST", "/v2/logs/query") => {
            let query = LogQuery.from_json(&req.body_str()?)?
            let result = execute_query(query)?
            // serialize as JSON for API consumers
            ng.net.HttpResponse.ok_json(result.to_json())
        },

        ("GET", path) if path.starts_with("/v2/logs/batch/") => {
            let batch_id = path["/v2/logs/batch/".len()..]
                .parse::<u64>()
                .map_err(|_| WireError.InvalidField("batch_id"))?
            let batch = fetch_batch(batch_id)?
            // return binary or json depending on Accept header
            if req.accepts("application/x-nglog-binary") {
                ng.net.HttpResponse.ok_binary(
                    "application/x-nglog-binary",
                    batch.to_binary()
                )
            } else {
                ng.net.HttpResponse.ok_json(batch.to_json())
            }
        },

        _ => ng.net.HttpResponse.not_found(),
    }
}

// ─── Forward-compatible reader ────────────────────────────────────────────────

// Reads either v1 or v2 binary — always returns v2 LogEntry
fn read_log_from_stream(buf: &[]u8) -> Result[Vec[LogEntry], WireError] {
    // peek at version byte (offset 3)
    if buf.len() < 4 { return Err(WireError.Truncated) }
    match buf[3] {
        1 => {
            let v1 = LogBatch.from_binary_v1(buf)?
            Ok(v1.entries.into_iter().map(migrate_log_v1_to_v2).collect())
        },
        2 => {
            let v2 = LogBatch.from_binary(buf)?
            Ok(v2.entries)
        },
        v => Err(WireError.UnsupportedVersion(v)),
    }
}

fn ingest_batch(batch: LogBatch) -> Result[(), WireError] ! {IO} { Ok(()) }
fn execute_query(q: LogQuery)    -> Result[QueryResult, WireError] ! {IO} { Ok(QueryResult { query_id: 0, total: 0, returned: 0, duration_us: 0, entries: Vec.new() }) }
fn fetch_batch(id: u64)          -> Result[LogBatch, WireError] ! {IO} { Err(WireError.NotFound) }
```

---

## Program 4: SIMD and GPU — Image Processing Pipeline

Gaussian blur with AVX2 SIMD, Sobel edge detection vectorized, GPU batch processing.
Compiler selects best instruction sequence; `@simd` blocks are verified to vectorize.

```ng
module ng.img

use ng.math.{sqrt, exp, PI}
use ng.mem.{Box, Arena}
use ng.core.{Result}

// ─── Image type ───────────────────────────────────────────────────────────────

type Pixel = { r: u8, g: u8, b: u8, a: u8 }

type Image = {
    width:  u32,
    height: u32,
    pixels: Box[[]u8],   // RGBA packed, row-major
}

impl Image {
    fn new(w: u32, h: u32) -> Self {
        Image {
            width:  w,
            height: h,
            pixels: Box.alloc_zeroed(w as usize * h as usize * 4),
        }
    }

    fn get_pixel(self, x: u32, y: u32) -> Pixel {
        let i = (y as usize * self.width as usize + x as usize) * 4
        Pixel { r: self.pixels[i], g: self.pixels[i+1],
                b: self.pixels[i+2], a: self.pixels[i+3] }
    }

    fn set_pixel(self: &mut Self, x: u32, y: u32, p: Pixel) {
        let i = (y as usize * self.width as usize + x as usize) * 4
        self.pixels[i]   = p.r
        self.pixels[i+1] = p.g
        self.pixels[i+2] = p.b
        self.pixels[i+3] = p.a
    }

    fn as_f32_plane(self, channel: u8) -> Vec[f32] {
        let n = self.width as usize * self.height as usize
        let mut out = Vec.with_capacity(n)
        let mut i = channel as usize
        while i < self.pixels.len() {
            out.push(self.pixels[i] as f32 / 255.0)
            i += 4
        }
        out
    }
}

// ─── SIMD grayscale conversion ────────────────────────────────────────────────

// Process 8 pixels at once using f32x8 SIMD
@simd
fn rgba_to_gray_plane(rgba: &[]u8, out: &mut []f32)
    requires rgba.len() % 32 == 0      // 8 pixels × 4 channels
    requires out.len() == rgba.len() / 4
{
    var i = 0usize
    while i < rgba.len() {
        // load 8 pixels of each channel
        let r8 = f32x8.load_u8_scaled(&rgba[i..],     0, 4, 1.0/255.0)
        let g8 = f32x8.load_u8_scaled(&rgba[i..],     1, 4, 1.0/255.0)
        let b8 = f32x8.load_u8_scaled(&rgba[i..],     2, 4, 1.0/255.0)

        // ITU-R BT.709 luma: Y = 0.2126R + 0.7152G + 0.0722B
        let gray8 = r8 * f32x8.splat(0.2126)
                  + g8 * f32x8.splat(0.7152)
                  + b8 * f32x8.splat(0.0722)

        gray8.store(&mut out[i / 4..])
        i += 32
    }
}

// ─── SIMD Gaussian blur (horizontal pass) ─────────────────────────────────────

fn build_gaussian_kernel(radius: u32, sigma: f32) -> Vec[f32] {
    let size = 2 * radius as usize + 1
    var kernel = Vec.with_capacity(size)
    var sum = 0.0f32
    for i in 0..size {
        let x = i as f32 - radius as f32
        let v = exp(-x * x / (2.0 * sigma * sigma))
        kernel.push(v)
        sum += v
    }
    // normalize
    for k in &mut kernel { *k /= sum }
    kernel
}

@simd
fn gaussian_blur_horizontal(
    src:    &[]f32,
    dst:    &mut []f32,
    width:  u32,
    height: u32,
    kernel: &[]f32,
    radius: u32,
)
    requires src.len() == dst.len()
    requires src.len() == width as usize * height as usize
    requires kernel.len() == 2 * radius as usize + 1
{
    let w = width as usize
    let r = radius as usize
    let klen = kernel.len()

    for y in 0..height as usize {
        let row_start = y * w

        // vectorized inner portion (full kernel always fits)
        var x = r
        while x + 8 <= w - r {
            var acc = f32x8.zero()
            for k in 0..klen {
                let col_start = row_start + x - r + k
                let pixels = f32x8.load(&src[col_start..])
                acc = acc + pixels * f32x8.splat(kernel[k])
            }
            acc.store(&mut dst[row_start + x..])
            x += 8
        }

        // scalar border fallback
        while x < w {
            var acc = 0.0f32
            for k in 0..klen {
                let col = (x + k).saturating_sub(r).min(w - 1)
                acc += src[row_start + col] * kernel[k]
            }
            dst[row_start + x] = acc
            x += 1
        }
    }
}

// ─── SIMD Sobel edge detection ────────────────────────────────────────────────

@simd
fn sobel_magnitude(
    src:    &[]f32,
    dst:    &mut []f32,
    width:  u32,
    height: u32,
)
    requires src.len() == dst.len()
    requires src.len() == width as usize * height as usize
{
    let w = width as usize
    let h = height as usize

    for y in 1..h - 1 {
        var x = 1usize
        while x + 8 <= w - 1 {
            let i = y * w + x

            // load 3×3 neighborhood for 8 pixels at once
            let tl = f32x8.load(&src[i - w - 1..])
            let tc = f32x8.load(&src[i - w..])
            let tr = f32x8.load(&src[i - w + 1..])
            let ml = f32x8.load(&src[i - 1..])
            let mr = f32x8.load(&src[i + 1..])
            let bl = f32x8.load(&src[i + w - 1..])
            let bc = f32x8.load(&src[i + w..])
            let br = f32x8.load(&src[i + w + 1..])

            // Sobel kernels
            let gx = tr + mr * f32x8.splat(2.0) + br
                   - tl - ml * f32x8.splat(2.0) - bl

            let gy = bl + bc * f32x8.splat(2.0) + br
                   - tl - tc * f32x8.splat(2.0) - tr

            let mag = (gx * gx + gy * gy).sqrt()
            mag.store(&mut dst[i..])
            x += 8
        }
        // scalar tail
        while x < w - 1 {
            let i = y * w + x
            let gx = (src[i-w+1] + 2.0*src[i+1] + src[i+w+1])
                   - (src[i-w-1] + 2.0*src[i-1] + src[i+w-1])
            let gy = (src[i+w-1] + 2.0*src[i+w] + src[i+w+1])
                   - (src[i-w-1] + 2.0*src[i-w] + src[i-w+1])
            dst[i] = sqrt(gx*gx + gy*gy)
            x += 1
        }
    }
}

// ─── GPU batch processing kernel ──────────────────────────────────────────────

// Runs on GPU — processes entire image batch in parallel
@gpu
fn gpu_threshold_batch(
    planes:    GpuBuffer[f32],   // [batch × w × h] flattened
    thresholds: GpuBuffer[f32],  // [batch] one threshold per image
    output:    GpuBuffer[u8],    // [batch × w × h] binary output
    image_size: u32,             // w * h pixels per image
) @thread_id(img, pixel) {
    let base  = img * image_size
    let idx   = base + pixel
    let thresh = thresholds[img]
    output[idx] = if planes[idx] >= thresh { 255u8 } else { 0u8 }
}

// ─── Full pipeline ────────────────────────────────────────────────────────────

fn process_batch(images: Vec[Image]) -> Vec[Image] ! {IO} {
    let n = images.len()
    if n == 0 { return Ok(Vec.new()) }

    let w = images[0].width  as usize
    let h = images[0].height as usize
    let sz = w * h

    // --- CPU phase: grayscale + gaussian blur (SIMD) ---
    let kernel = build_gaussian_kernel(3, 1.2)
    var gray_planes = Vec.with_capacity(n * sz)
    var blurred     = vec![0.0f32; n * sz]

    for (idx, img) in images.iter().enumerate() {
        let gray = img.as_f32_plane(0)
        rgba_to_gray_plane(&img.pixels, &mut gray_planes[idx*sz..])
        gaussian_blur_horizontal(
            &gray_planes[idx*sz..],
            &mut blurred[idx*sz..],
            img.width, img.height,
            &kernel, 3,
        )
    }

    // --- Edge detection (SIMD, per image) ---
    var edges = vec![0.0f32; n * sz]
    for i in 0..n {
        sobel_magnitude(
            &blurred[i*sz..],
            &mut edges[i*sz..],
            images[i].width,
            images[i].height,
        )
    }

    // --- GPU phase: threshold batch ---
    let gpu_edges  = GpuBuffer.upload(&edges)
    let thresholds = GpuBuffer.upload(&vec![0.15f32; n])
    let gpu_output = GpuBuffer.alloc::<u8>(n * sz)

    gpu_threshold_batch.dispatch(
        grid:  (n as u32, sz as u32),
        gpu_edges, thresholds, gpu_output, sz as u32,
    )

    let result_bytes = gpu_output.download()

    // rebuild images
    var out = Vec.with_capacity(n)
    for i in 0..n {
        var img = Image.new(images[i].width, images[i].height)
        for j in 0..sz {
            let v = result_bytes[i * sz + j]
            img.pixels[j*4]   = v
            img.pixels[j*4+1] = v
            img.pixels[j*4+2] = v
            img.pixels[j*4+3] = 255
        }
        out.push(img)
    }
    Ok(out)
}
```

---

## Program 5: Stable ABI — Plugin Host with Versioned API

Layout is guaranteed forever. Old plugins load in new host without recompilation.

```ng
module app.plugin_host

use ng.cap.{FileCap, NoCaps}
use ng.mem.{Arc, Box}
use ng.collections.{Vec, HashMap}
use ng.os.{DynLib}
use ng.core.{Result, Option}

// ─── Stable ABI types — layout NEVER changes ─────────────────────────────────

@stable_abi(since: "1.0")
type PluginApiVersion = {
    major: u16,
    minor: u16,
    patch: u32,
}

@stable_abi(since: "1.0")
type SliceRef = {
    ptr: *const u8,
    len: usize,
}
// safe wrapper — never expose the raw fields outside of FFI layer

@stable_abi(since: "1.0")
type PluginInfo = {
    api_version: PluginApiVersion,
    plugin_name: SliceRef,    // points into plugin's read-only data segment
    plugin_ver:  PluginApiVersion,
    flags:       u32,
}

@stable_abi(since: "1.0")
type RequestHandle = { id: u64, _pad: [56]u8 }  // 64 bytes, opaque

@stable_abi(since: "1.0")
type ResponseHandle = { id: u64, status: u32, _pad: [52]u8 }

// v1.1 extends without breaking v1.0 plugins: old plugins never read new fields
@stable_abi(since: "1.1")
type PluginInfoV1_1 = {
    base:       PluginInfo,   // first field = v1.0 PluginInfo, same offset
    trace_flags: u32,
    reserved:   [28]u8,
}

// ─── Plugin vtable — function pointers with stable calling convention ─────────

@stable_abi(since: "1.0")
@repr(c)
type PluginVTable = {
    version:    u32,   // vtable version — plugins check this before calling new entries
    init:       fn(*mut u8, *const HostVTable) -> i32,          // 0 = ok
    shutdown:   fn(*mut u8) -> (),
    on_request: fn(*mut u8, RequestHandle) -> ResponseHandle,
    on_tick:    fn(*mut u8, u64) -> (),                         // millis since start
    // v1.1 extension — only called if vtable.version >= 2
    on_reload:  Option[fn(*mut u8, *const u8, usize) -> i32],  // hot config reload
    _reserved:  [6]*const u8,                                   // space for future methods
}

@stable_abi(since: "1.0")
@repr(c)
type HostVTable = {
    version:     u32,
    alloc:       fn(usize) -> *mut u8,
    free:        fn(*mut u8, usize) -> (),
    log:         fn(u8, *const u8, usize) -> (),  // level, msg ptr, len
    read_config: fn(*const u8, usize, *mut u8, usize) -> i32,  // key, out_buf, returns len
    _reserved:   [8]*const u8,
}

// ─── Host-side implementation ─────────────────────────────────────────────────

type LoadedPlugin = {
    lib:    DynLib,
    vtable: &'static PluginVTable,
    state:  *mut u8,
    info:   PluginInfo,
}

impl LoadedPlugin {
    fn load(path: str, @cap file: FileCap) -> Result[Self, PluginLoadError] {
        let lib = DynLib.open(path, file)?

        // every ng plugin exports ng_plugin_vtable
        let vtable_ptr = lib.symbol::<PluginVTable>("ng_plugin_vtable")?
        let vtable = unsafe { &*vtable_ptr }

        if vtable.version < 1 || vtable.version > 2 {
            return Err(PluginLoadError.IncompatibleVersion(vtable.version))
        }

        // build host vtable — stable layout guaranteed
        let host_vtable: &'static HostVTable = &HOST_VTABLE

        // call plugin init — plugin allocates its own state
        let state = unsafe {
            let mut state_ptr: *mut u8 = core::ptr::null_mut()
            let rc = (vtable.init)(&mut state_ptr, host_vtable)
            if rc != 0 { return Err(PluginLoadError.InitFailed(rc)) }
            state_ptr
        }

        // read plugin info
        let info_ptr = lib.symbol::<PluginInfo>("ng_plugin_info")?
        let info = unsafe { (*info_ptr).clone() }

        Ok(LoadedPlugin { lib, vtable, state, info })
    }

    fn dispatch_request(self, req: RequestHandle) -> ResponseHandle {
        unsafe { (self.vtable.on_request)(self.state, req) }
    }

    fn tick(self, millis: u64) {
        unsafe { (self.vtable.on_tick)(self.state, millis) }
    }

    fn hot_reload_config(self, config: &[]u8) -> Result[(), PluginLoadError] {
        match self.vtable.on_reload {
            None => Ok(()),  // plugin doesn't support hot reload
            Some(reload_fn) => {
                let rc = unsafe { reload_fn(self.state, config.as_ptr(), config.len()) }
                if rc != 0 { Err(PluginLoadError.ReloadFailed(rc)) } else { Ok(()) }
            }
        }
    }
}

type PluginLoadError = enum {
    NotFound(str),
    IncompatibleVersion(u32),
    MissingSymbol(str),
    InitFailed(i32),
    ReloadFailed(i32),
}

// The global host vtable — stable across all host versions
static HOST_VTABLE: HostVTable = HostVTable {
    version:     2,
    alloc:       |size| ng.mem.raw_alloc(size),
    free:        |ptr, size| ng.mem.raw_free(ptr, size),
    log:         |level, ptr, len| {
                     let s = unsafe { str.from_raw_parts(ptr, len) }
                     ng.fmt.eprintln("[plugin:{level}] {s}")
                 },
    read_config: |key_ptr, key_len, out, out_len| {
                     let key = unsafe { str.from_raw_parts(key_ptr, key_len) }
                     match GLOBAL_CONFIG.get(key) {
                         None    => -1,
                         Some(v) => {
                             let bytes = v.as_bytes()
                             let n = bytes.len().min(out_len)
                             unsafe { core.mem.copy(bytes.as_ptr(), out, n) }
                             n as i32
                         }
                     }
                 },
    _reserved:   [core::ptr::null(); 8],
}

static GLOBAL_CONFIG: HashMap[str, str] = HashMap.new()
```

---

## Program 6: Information Flow Types — Authentication Service

`Secret[T]` can never reach a log, HTTP response, or file.
Declassification is explicit, auditable, and rare.

```ng
module app.auth

use ng.flow.{Secret, Public, Trusted, declassify}
use ng.crypto.{argon2id, hmac_sha256, generate_random}
use ng.time.{Timestamp, Duration}
use ng.core.{Result, Option}
use ng.collections.HashMap
use ng.cap.{ClockCap, FileCap}

// ─── Sensitive types — tagged at definition ───────────────────────────────────

type RawPassword = Secret[str]
type ApiKey       = Secret[str]
type SessionToken = Secret[str]
type PasswordHash = Secret[[32]u8]

type UserId   = u64
type Username = Public[str]  // usernames are NOT secret
type Email    = Trusted[str] // email: can flow to verified outputs, not arbitrary ones

// ─── User record ──────────────────────────────────────────────────────────────

type User = {
    id:            UserId,
    username:      Username,
    email:         Email,
    password_hash: PasswordHash,
    created_at:    Timestamp,
    last_login:    Option[Timestamp],
    failed_logins: u32,
    locked:        bool,
}

// ─── Hash and verify — the only declassification points ──────────────────────

// This is the ONLY place a RawPassword is consumed.
// The result is a PasswordHash — still secret, but a one-way transformation.
fn hash_password(raw: RawPassword) -> PasswordHash {
    // argon2id parameters: memory=64MB, iterations=3, parallelism=4
    let bytes = argon2id(
        raw.inner_unsafe_bytes(),   // explicit: compiler requires acknowledgement
        generate_random::<[16]u8>(),
        64 * 1024, 3, 4, 32
    )
    Secret(bytes)
}

// Verify password — declassify ONLY the boolean result (not the hash)
// This is explicitly marked @declassify so it appears in the audit report.
@declassify(Secret -> Public, reason: "timing-safe comparison, boolean only")
fn verify_password(raw: RawPassword, stored: &PasswordHash) -> Public[bool] {
    let candidate = argon2id(
        raw.inner_unsafe_bytes(),
        stored.inner_unsafe_bytes()[0..16],  // salt embedded in hash
        64 * 1024, 3, 4, 32
    )
    // constant-time comparison — no timing side channel
    Public(ng.crypto.constant_eq(&candidate, stored.inner_unsafe_bytes()))
}

// ─── Session management ───────────────────────────────────────────────────────

type Session = {
    token:      SessionToken,
    user_id:    UserId,
    expires_at: Timestamp,
    ip:         Public[str],
    user_agent: Public[str],
}

fn generate_session_token() -> SessionToken {
    let bytes = generate_random::<[32]u8>()
    // base64url encode — result is still Secret
    Secret(ng.serial.base64url_encode(&bytes))
}

// SessionToken → public session ID (safe to put in cookie header)
// The actual token bytes are never in any public output — only a derived ID.
@declassify(Secret -> Public, reason: "HMAC binding: token cannot be reconstructed from id")
fn token_to_session_id(token: &SessionToken, signing_key: &Secret[[32]u8]) -> Public[str] {
    let mac = hmac_sha256(signing_key.inner_unsafe_bytes(), token.inner_unsafe_bytes())
    Public(ng.serial.hex_encode(&mac[..16]))  // 128-bit session ID
}

// ─── Login flow ───────────────────────────────────────────────────────────────

type LoginRequest = {
    username: str,           // not yet validated
    password: RawPassword,   // comes in as secret
}

type LoginError = enum {
    UserNotFound,
    WrongPassword,
    AccountLocked,
    RateLimited,
}

// The return value contains only Public data — no secrets escape
type LoginSuccess = {
    session_id: Public[str],   // safe to put in Set-Cookie
    user_id:    UserId,
    username:   Username,
    // NOT included: token, password_hash, email (trusted, not for arbitrary consumers)
}

fn login(
    req:          LoginRequest,
    db:           &mut UserDb,
    sessions:     &mut SessionStore,
    @cap clock:   ClockCap,
) -> Result[LoginSuccess, LoginError]
    ! {IO, Fail[LoginError]}
{
    // constant-time lookup — don't reveal whether user exists via timing
    let user = db.find_by_username_const_time(&req.username)

    if user.locked {
        return Err(LoginError.AccountLocked)
    }

    // even if user not found, we still call verify_password (with dummy hash)
    // so timing doesn't reveal username existence
    let dummy_hash: PasswordHash = Secret(DUMMY_HASH)
    let hash = match &user.password_hash {
        Some(h) => h,
        None    => &dummy_hash,
    }

    let ok: Public[bool] = verify_password(req.password, hash)

    // update failure counter before checking result (timing safety)
    if !*ok {
        db.record_failed_login(user.id, clock.now())
        // UserNotFound and WrongPassword return in same time window
        return Err(if user.exists { LoginError.WrongPassword }
                   else           { LoginError.UserNotFound })
    }

    let token      = generate_session_token()
    let signing_key: Secret[[32]u8> = db.get_signing_key()
    let session_id = token_to_session_id(&token, &signing_key)

    let session = Session {
        token,
        user_id:    user.id,
        expires_at: clock.now() + Duration.from_hours(24),
        ip:         Public(req.ip.clone()),
        user_agent: Public(req.user_agent.clone()),
    }

    sessions.insert(session)
    db.record_successful_login(user.id, clock.now())

    Ok(LoginSuccess {
        session_id,
        user_id:  user.id,
        username: user.username.clone(),
    })
}

// These would be COMPILE ERRORS — showing what the type system prevents:
//
// fn bad_log(user: &User) {
//     ng.fmt.println("user hash: {}", user.password_hash)
//     //                                 ^^^^^^^^^^^^^^^^
//     // ERROR: Secret[_] cannot flow to Public context (println is Public)
// }
//
// fn bad_response(token: &SessionToken) -> HttpResponse {
//     HttpResponse.ok_json(format!(r#"{{"token":"{}"}}"#, token))
//     //                                                   ^^^^^
//     // ERROR: Secret[str] cannot flow to Public[str]
//     //        (HttpResponse body is Public — anyone on the network reads it)
// }
//
// fn bad_forward(token: SessionToken) -> Secret[str] {
//     token  // OK — staying within Secret context
// }

static DUMMY_HASH: [32]u8 = [0u8; 32]

type UserDb = {}
type SessionStore = {}
impl UserDb {
    fn find_by_username_const_time(self, _: &str) -> User { todo() }
    fn record_failed_login(self: &mut Self, _: UserId, _: Timestamp) {}
    fn record_successful_login(self: &mut Self, _: UserId, _: Timestamp) {}
    fn get_signing_key(self) -> Secret[[32]u8] { Secret([0u8; 32]) }
}
impl SessionStore { fn insert(self: &mut Self, _: Session) {} }
```

---

## Program 7: Temporal Types — Multi-Level Session Management

Every token type carries its TTL in the type. Expired tokens cannot be used.
Refresh chains, OTP codes, and API keys all have distinct expiry semantics.

```ng
module app.sessions

use ng.time.{Timestamp, Duration, Expired, @ttl}
use ng.flow.{Secret, Public}
use ng.crypto.{hmac_sha256, generate_random}
use ng.core.{Result, Option}
use ng.cap.ClockCap

// ─── Token types with TTL in the type ────────────────────────────────────────

// Short-lived access token: 15 minutes
type AccessToken = Secret[str] @ttl(900s)

// Long-lived refresh token: 30 days
type RefreshToken = Secret[str] @ttl(2592000s)

// One-time password: 30 seconds (TOTP window)
type OtpCode = str @ttl(30s)

// API key: 1 year, but explicitly revocable
type ApiKey = Secret[str] @ttl(365d)

// Email verification link: 24 hours
type VerifyLink = str @ttl(86400s)

// ─── Token store — stores expiry alongside value ──────────────────────────────

type StoredToken[T, comptime Ttl: Duration] = {
    value:     T @ttl(Ttl),
    issued_at: Timestamp,
    user_id:   u64,
    metadata:  TokenMeta,
}
where Timestamp.now() < issued_at + Ttl   // compile: type invariant
                                           // runtime: checked on every access

type TokenMeta = {
    ip:           str,
    user_agent:   str,
    scope:        TokenScope,
}

type TokenScope = enum {
    Full,
    ReadOnly,
    Scoped(Vec[str]),  // list of allowed resource prefixes
}

// ─── Token generation ─────────────────────────────────────────────────────────

fn generate_access_token(
    user_id: u64,
    meta:    TokenMeta,
    key:     &Secret[[32]u8],
    @cap clock: ClockCap,
) -> StoredToken[AccessToken, 900s] {
    let raw = generate_random::<[32]u8>()
    let payload = ng.serial.base64url_encode(&raw)
    let mac = hmac_sha256(key.inner_unsafe_bytes(), payload.as_bytes())
    let token = Secret(format!("{payload}.{}", ng.serial.hex_encode(&mac[..8])))

    StoredToken {
        value:     token,
        issued_at: clock.now(),
        user_id,
        metadata:  meta,
    }
}

fn generate_otp(seed: &Secret[[20]u8], @cap clock: ClockCap) -> OtpCode {
    // TOTP RFC 6238
    let counter = (clock.now().as_unix_secs() / 30) as u64
    let mac = hmac_sha256(seed.inner_unsafe_bytes(), &counter.to_be_bytes())
    let offset = (mac[19] & 0x0f) as usize
    let code = u32.from_be_bytes(mac[offset..offset+4]) & 0x7fff_ffff
    format!("{:06}", code % 1_000_000)
}

// ─── Token verification with automatic expiry ─────────────────────────────────

type VerifyError = enum {
    NotFound,
    Expired,
    SignatureMismatch,
    Revoked,
}

// The returned token is proven non-expired at time of verification.
// The type carries a "verified_at" timestamp so callers know how fresh it is.
type VerifiedToken[T, comptime Ttl: Duration] = {
    inner:       StoredToken[T, Ttl],
    verified_at: Timestamp,
}

fn verify_access_token(
    raw:       &str,
    store:     &TokenStore,
    key:       &Secret[[32]u8],
    @cap clock: ClockCap,
) -> Result[VerifiedToken[AccessToken, 900s], VerifyError] {
    let stored = store.find_access_token(raw)
        .ok_or(VerifyError.NotFound)?

    if store.is_revoked(&stored.value) {
        return Err(VerifyError.Revoked)
    }

    // The compiler inserts this check automatically because of @ttl
    // but we can also check manually and return a typed error
    let age = clock.now() - stored.issued_at
    if age > Duration.from_secs(900) {
        return Err(VerifyError.Expired)
    }

    // verify HMAC
    let parts: [str; 2] = raw.splitn('.', 2).try_into()?
    let mac_expected = hmac_sha256(key.inner_unsafe_bytes(), parts[0].as_bytes())
    if !ng.crypto.constant_eq(parts[1].as_bytes(), &ng.serial.hex_encode(&mac_expected[..8]).as_bytes()) {
        return Err(VerifyError.SignatureMismatch)
    }

    Ok(VerifiedToken { inner: stored, verified_at: clock.now() })
}

// ─── Refresh flow ─────────────────────────────────────────────────────────────

fn refresh_tokens(
    refresh_raw: &str,
    store:       &mut TokenStore,
    key:         &Secret[[32]u8],
    meta:        TokenMeta,
    @cap clock:  ClockCap,
) -> Result[(StoredToken[AccessToken, 900s], StoredToken[RefreshToken, 2592000s]), VerifyError]
{
    let stored = store.find_refresh_token(refresh_raw)
        .ok_or(VerifyError.NotFound)?

    let age = clock.now() - stored.issued_at
    if age > Duration.from_secs(2592000) {
        return Err(VerifyError.Expired)
    }

    if store.is_revoked(&stored.value) {
        return Err(VerifyError.Revoked)
    }

    // Refresh token rotation: old token is revoked, new ones issued
    store.revoke(&stored.value)

    let new_access  = generate_access_token(stored.user_id, meta.clone(), key, clock)
    let new_refresh = generate_refresh_token(stored.user_id, meta, key, clock)

    Ok((new_access, new_refresh))
}

fn generate_refresh_token(
    user_id: u64,
    meta:    TokenMeta,
    key:     &Secret[[32]u8],
    @cap clock: ClockCap,
) -> StoredToken[RefreshToken, 2592000s] {
    let raw = generate_random::<[48]u8>()
    let token = Secret(ng.serial.base64url_encode(&raw))
    StoredToken {
        value:     token,
        issued_at: clock.now(),
        user_id,
        metadata:  meta,
    }
}

// The compiler would reject this:
//
// fn bad_use_expired(token: StoredToken[AccessToken, 900s>, @cap clock: ClockCap) {
//     ng.fmt.println("user: {}", token.user_id)
//     // ERROR if token.issued_at + 900s < clock.now()
//     //         and the compiler can prove this statically
// }

type TokenStore = {}
impl TokenStore {
    fn find_access_token(self, _: &str)  -> Option[StoredToken[AccessToken, 900s]]   { None }
    fn find_refresh_token(self, _: &str) -> Option[StoredToken[RefreshToken, 2592000s>] { None }
    fn is_revoked(self, _: &Secret[str]) -> bool { false }
    fn revoke(self: &mut Self, _: &Secret[str]) {}
}
```

---

## Program 8: Distributed Types — Replicated Key-Value Store

Remote values are typed identically to local. Network failures are effects.
Replication protocol with leader election. No raw sockets exposed to application code.

```ng
module app.kvstore

use ng.dist.{Remote, DistChannel, NodeAddr, cluster}
use ng.sync.{RwLock, Mutex, Condvar}
use ng.collections.{HashMap, BTreeMap, Vec}
use ng.time.{Timestamp, Duration}
use ng.core.{Result, Option}
use ng.cap.{NetCap, ClockCap}

// ─── Raft consensus log entry ─────────────────────────────────────────────────

type Term  = u64
type Index = u64
type NodeId = u32

type LogEntry = enum {
    Put { key: str, value: Vec[u8], client_id: u64, seq: u64 },
    Del { key: str, client_id: u64, seq: u64 },
    NoOp,
}

type RaftLog = {
    entries: Vec[(Term, LogEntry)>,
    commit:  Index,
    applied: Index,
}

// ─── Remote state machine — the value lives on the leader ────────────────────

type KvState = {
    data:        BTreeMap[str, Vec[u8]>,
    client_seqs: HashMap[u64, u64],  // deduplication
}

// Remote[KvState, "leader"] = KvState living on the leader node
// Accessing it transparently generates network RPCs with Fail[NetError] effect
type LeaderState = Remote[KvState, "leader"]

// ─── Raft RPC types ───────────────────────────────────────────────────────────

@wire(binary)
type RequestVoteReq = {
    term:          Term,
    candidate_id:  NodeId,
    last_log_idx:  Index,
    last_log_term: Term,
}

@wire(binary)
type RequestVoteResp = {
    term:         Term,
    vote_granted: bool,
}

@wire(binary)
type AppendEntriesReq = {
    term:          Term,
    leader_id:     NodeId,
    prev_log_idx:  Index,
    prev_log_term: Term,
    entries:       Vec[(Term, LogEntry)],
    leader_commit: Index,
}

@wire(binary)
type AppendEntriesResp = {
    term:    Term,
    success: bool,
    match_idx: Index,
}

// ─── Raft node ────────────────────────────────────────────────────────────────

type RaftRole = enum { Follower, Candidate, Leader }

type RaftNode = {
    id:           NodeId,
    peers:        Vec[NodeAddr],
    current_term: Term,
    voted_for:    Option[NodeId],
    log:          RaftLog,
    role:         RaftRole,
    leader_id:    Option[NodeId],
    commit_index: Index,
    next_index:   HashMap[NodeId, Index],
    match_index:  HashMap[NodeId, Index],
    state:        KvState,
    // election timer
    election_deadline: Timestamp,
}

impl RaftNode {
    fn new(id: NodeId, peers: Vec[NodeAddr], @cap clock: ClockCap) -> Self {
        RaftNode {
            id,
            peers,
            current_term: 0,
            voted_for:    None,
            log:          RaftLog { entries: Vec.new(), commit: 0, applied: 0 },
            role:         RaftRole.Follower,
            leader_id:    None,
            commit_index: 0,
            next_index:   HashMap.new(),
            match_index:  HashMap.new(),
            state:        KvState { data: BTreeMap.new(), client_seqs: HashMap.new() },
            election_deadline: Self.random_election_timeout(clock),
        }
    }

    fn random_election_timeout(@cap clock: ClockCap) -> Timestamp {
        let jitter = ng.crypto.random_range(150, 300) as u64
        clock.now() + Duration.from_millis(jitter)
    }

    // Handle incoming RequestVote RPC
    fn handle_request_vote(
        self: &mut Self,
        req: RequestVoteReq,
        @cap clock: ClockCap,
    ) -> RequestVoteResp {
        if req.term < self.current_term {
            return RequestVoteResp { term: self.current_term, vote_granted: false }
        }
        if req.term > self.current_term {
            self.become_follower(req.term)
        }
        let last_idx  = self.log.entries.len() as u64
        let last_term = self.log.entries.last().map(|(t, _)| *t).unwrap_or(0)

        let log_ok = req.last_log_term > last_term
                  || (req.last_log_term == last_term && req.last_log_idx >= last_idx)

        let grant = log_ok && match self.voted_for {
            None     => true,
            Some(id) => id == req.candidate_id,
        }

        if grant {
            self.voted_for = Some(req.candidate_id)
            self.election_deadline = Self.random_election_timeout(clock)
        }

        RequestVoteResp { term: self.current_term, vote_granted: grant }
    }

    // Handle AppendEntries (heartbeat or log replication)
    fn handle_append_entries(
        self: &mut Self,
        req: AppendEntriesReq,
        @cap clock: ClockCap,
    ) -> AppendEntriesResp {
        if req.term < self.current_term {
            return AppendEntriesResp { term: self.current_term, success: false, match_idx: 0 }
        }
        self.become_follower(req.term)
        self.leader_id = Some(req.leader_id)
        self.election_deadline = Self.random_election_timeout(clock)

        // consistency check
        if req.prev_log_idx > 0 {
            let idx = req.prev_log_idx as usize - 1
            if idx >= self.log.entries.len() {
                return AppendEntriesResp { term: self.current_term, success: false, match_idx: 0 }
            }
            let (stored_term, _) = self.log.entries[idx]
            if stored_term != req.prev_log_term {
                // conflict — truncate
                self.log.entries.truncate(idx)
                return AppendEntriesResp { term: self.current_term, success: false, match_idx: 0 }
            }
        }

        // append new entries
        let base = req.prev_log_idx as usize
        for (i, entry) in req.entries.into_iter().enumerate() {
            let idx = base + i
            if idx < self.log.entries.len() {
                if self.log.entries[idx].0 != entry.0 {
                    self.log.entries.truncate(idx)
                    self.log.entries.push(entry)
                }
            } else {
                self.log.entries.push(entry)
            }
        }

        if req.leader_commit > self.commit_index {
            self.commit_index = req.leader_commit.min(self.log.entries.len() as u64)
            self.apply_committed()
        }

        AppendEntriesResp {
            term:      self.current_term,
            success:   true,
            match_idx: self.log.entries.len() as u64,
        }
    }

    fn apply_committed(self: &mut Self) {
        while self.log.applied < self.commit_index {
            let idx = self.log.applied as usize
            match &self.log.entries[idx].1 {
                LogEntry.Put { key, value, client_id, seq } => {
                    if self.state.client_seqs.get(client_id).copied().unwrap_or(0) < *seq {
                        self.state.data.insert(key.clone(), value.clone())
                        self.state.client_seqs.insert(*client_id, *seq)
                    }
                },
                LogEntry.Del { key, client_id, seq } => {
                    if self.state.client_seqs.get(client_id).copied().unwrap_or(0) < *seq {
                        self.state.data.remove(key)
                        self.state.client_seqs.insert(*client_id, *seq)
                    }
                },
                LogEntry.NoOp => {},
            }
            self.log.applied += 1
        }
    }

    fn become_follower(self: &mut Self, term: Term) {
        self.current_term = term
        self.role         = RaftRole.Follower
        self.voted_for    = None
    }
}

// ─── Client API — location-transparent ───────────────────────────────────────

type KvClient = {
    cluster: Vec[NodeAddr],
    leader:  Option[NodeAddr],
    id:      u64,
    seq:     u64,
}

type KvError = enum {
    NotLeader(Option[NodeAddr]),
    Timeout,
    NetworkError(ng.net.NetError),
    KeyNotFound,
}

impl KvClient {
    fn put(
        self: &mut Self,
        key:   str,
        value: Vec[u8],
        @cap net:   NetCap,
        @cap clock: ClockCap,
    ) -> Result[(), KvError] ! {IO, Fail[KvError]}
    {
        self.seq += 1
        self.rpc_with_retry(
            |addr| self.send_put(addr, key.clone(), value.clone(), self.seq, net),
            clock,
        )
    }

    fn get(
        self: &mut Self,
        key:  &str,
        @cap net:   NetCap,
        @cap clock: ClockCap,
    ) -> Result[Option[Vec[u8]], KvError] ! {IO, Fail[KvError]}
    {
        // reads can go to any node (linearizable reads use leader)
        let addr = self.leader.unwrap_or_else(|| self.cluster[0])
        self.send_get(addr, key, net)
    }

    fn rpc_with_retry[T](
        self: &mut Self,
        f:     fn(NodeAddr) -> Result[T, KvError],
        @cap clock: ClockCap,
    ) -> Result[T, KvError] ! {IO, Fail[KvError]}
    {
        let deadline = clock.now() + Duration.from_secs(5)
        var last_err = KvError.Timeout

        while clock.now() < deadline {
            let addr = self.leader.unwrap_or_else(|| self.cluster[0])
            match f(addr) {
                Ok(v) => return Ok(v),
                Err(KvError.NotLeader(Some(new_leader))) => {
                    self.leader = Some(new_leader)
                },
                Err(KvError.NotLeader(None)) => {
                    // try each node
                    for node in &self.cluster {
                        self.leader = Some(*node)
                        break
                    }
                },
                Err(e) => { last_err = e },
            }
            ng.time.sleep(Duration.from_millis(50), clock)
        }
        Err(last_err)
    }

    fn send_put(self, addr: NodeAddr, key: str, value: Vec[u8], seq: u64, @cap net: NetCap)
        -> Result[(), KvError] ! {IO} { Ok(()) }
    fn send_get(self, addr: NodeAddr, key: &str, @cap net: NetCap)
        -> Result[Option[Vec[u8]], KvError] ! {IO} { Ok(None) }
}
```

---

## Program 9: Proof Synthesis — Verified Sort and Cryptographic Contracts

Z3 proves postconditions at compile time. Counterexamples are shown as build errors.

```ng
module app.verified

use ng.proof.{requires, ensures, inv, assume, assert_proof}
use ng.core.{Result, Option}
use ng.collections.Vec

// ─── Verified merge sort ──────────────────────────────────────────────────────

// Postcondition: result is sorted AND is a permutation of input
// Z3 proves this for all possible inputs, or build fails with counterexample.
fn merge_sort(input: Vec[i64]) -> Vec[i64>
    ensures result.len() == input.len()
    ensures result.is_sorted()
    ensures result.is_permutation_of(&input)
{
    if input.len() <= 1 { return input }

    let mid = input.len() / 2
    let left  = merge_sort(input[..mid].to_vec())
    let right = merge_sort(input[mid..].to_vec())

    // Z3 uses the recursive postconditions as lemmas here:
    // left.is_sorted() and right.is_sorted() are proven, so merge is easier to verify
    merge(left, right)
}

fn merge(mut a: Vec[i64], mut b: Vec[i64]) -> Vec[i64>
    requires a.is_sorted()
    requires b.is_sorted()
    ensures  result.is_sorted()
    ensures  result.len() == a.len() + b.len()
    ensures  result.is_permutation_of_concat(&a, &b)
{
    var out = Vec.with_capacity(a.len() + b.len())
    var i = 0usize
    var j = 0usize

    while i < a.len() && j < b.len()
        inv out.len() == i + j
        inv out.is_sorted()
        inv out.all(|x| x <= a[i])   // elements not yet taken from a are >= last out
        inv out.all(|x| x <= b[j])   // elements not yet taken from b are >= last out
    {
        if a[i] <= b[j] {
            out.push(a[i])
            i += 1
        } else {
            out.push(b[j])
            j += 1
        }
    }
    out.extend_from_slice(&a[i..])
    out.extend_from_slice(&b[j..])
    out
}

// ─── Verified binary search ───────────────────────────────────────────────────

fn binary_search(arr: &Vec[i64], target: i64) -> Option[usize>
    requires arr.is_sorted()
    ensures match result {
        Some(i) => i < arr.len() && arr[i] == target,
        None    => arr.all(|x| x != target),
    }
{
    var lo = 0usize
    var hi = arr.len()

    while lo < hi
        inv lo <= hi
        inv hi <= arr.len()
        // if target is in arr, it's in arr[lo..hi]
        inv arr[..lo].all(|x| x < target)
        inv arr[hi..].all(|x| x > target)
    {
        let mid = lo + (hi - lo) / 2
        if arr[mid] == target {
            return Some(mid)
        } else if arr[mid] < target {
            lo = mid + 1
        } else {
            hi = mid
        }
    }
    None
}

// ─── Verified integer square root ─────────────────────────────────────────────

fn isqrt(n: u64) -> u64
    ensures result * result <= n
    ensures (result + 1) * (result + 1) > n
{
    if n == 0 { return 0 }
    var x = n
    var y = (x + 1) / 2

    while y < x
        inv y * y <= n || x <= y
    {
        x = y
        y = (x + n / x) / 2
    }
    x
}

// ─── Verified RSA key check ───────────────────────────────────────────────────

// Postcondition: returned (e, d) are valid RSA key pair for modulus n
fn compute_rsa_keypair(p: u64, q: u64) -> Result[(u64, u64, u64), RsaError>
    requires p > 1 && is_prime(p)
    requires q > 1 && is_prime(q)
    requires p != q
    ensures match result {
        Ok((n, e, d)) => {
            n == p * q &&
            gcd(e, (p-1) * (q-1)) == 1 &&
            (e * d) % ((p-1) * (q-1)) == 1
        },
        Err(_) => true,
    }
{
    let n  = p * q
    let phi = (p - 1) * (q - 1)
    let e   = find_coprime_e(phi)?

    // extended Euclidean: compute d such that e*d ≡ 1 (mod phi)
    let d = mod_inverse(e, phi).ok_or(RsaError.NoInverse)?

    // Z3 verifies: (e * d) % phi == 1 holds for all valid e, phi
    assert_proof((e * d) % phi == 1)

    Ok((n, e, d))
}

fn find_coprime_e(phi: u64) -> Result[u64, RsaError>
    requires phi > 2
    ensures match result {
        Ok(e) => e > 1 && e < phi && gcd(e, phi) == 1,
        Err(_) => true,
    }
{
    // try standard choices first
    for candidate in [65537u64, 257, 17, 5, 3] {
        if candidate < phi && gcd(candidate, phi) == 1 {
            return Ok(candidate)
        }
    }
    // fallback: search
    var e = phi - 1
    while e > 1 {
        if gcd(e, phi) == 1 { return Ok(e) }
        e -= 1
    }
    Err(RsaError.NoCoprime)
}

fn gcd(a: u64, b: u64) -> u64
    ensures result > 0
    ensures a % result == 0
    ensures b % result == 0
    ensures result == a || result == b || (a % result == 0 && b % result == 0)
{
    if b == 0 { a } else { gcd(b, a % b) }
}

fn mod_inverse(a: u64, m: u64) -> Option[u64>
    requires m > 1
    ensures match result {
        Some(d) => (a * d) % m == 1,
        None    => gcd(a, m) != 1,
    }
{
    // extended Euclidean algorithm
    var old_r = a as i128;  var r = m as i128
    var old_s = 1i128;      var s = 0i128

    while r != 0 {
        let q = old_r / r
        (old_r, r) = (r, old_r - q * r)
        (old_s, s) = (s, old_s - q * s)
    }
    if old_r != 1 { return None }
    Some(((old_s % m as i128 + m as i128) % m as i128) as u64)
}

fn is_prime(n: u64) -> bool { if n < 2 { return false } for i in 2..=isqrt(n) { if n % i == 0 { return false } } true }
type RsaError = enum { NoInverse, NoCoprime }
```

---

## Program 10: Reversible Functions — Binary Codec Library

`@reversible` generates decoders from encoder definitions.
Self-inverse functions are detected automatically.

```ng
module ng.codec

use ng.core.{Result}
use ng.collections.Vec

// ─── Primitive codecs ─────────────────────────────────────────────────────────

// Compiler generates: decode_u32_le(buf: &[]u8) -> Result[(u32, &[]u8), CodecError>
@reversible
fn encode_u32_le(n: u32, buf: &mut Vec[u8]) {
    buf.push((n & 0xFF) as u8)
    buf.push(((n >> 8)  & 0xFF) as u8)
    buf.push(((n >> 16) & 0xFF) as u8)
    buf.push(((n >> 24) & 0xFF) as u8)
}

// Compiler generates: decode_u64_le
@reversible
fn encode_u64_le(n: u64, buf: &mut Vec[u8]) {
    encode_u32_le((n & 0xFFFFFFFF) as u32, buf)
    encode_u32_le((n >> 32) as u32, buf)
}

// Variable-length integer (LEB128 encoding)
// Compiler generates: decode_varint(buf: &[]u8) -> Result[(u64, &[]u8), CodecError>
@reversible
fn encode_varint(mut n: u64, buf: &mut Vec[u8]) {
    loop {
        let byte = (n & 0x7F) as u8
        n >>= 7
        if n == 0 {
            buf.push(byte)
            break
        } else {
            buf.push(byte | 0x80)
        }
    }
}

// UTF-8 length-prefixed string
@reversible
fn encode_string(s: &str, buf: &mut Vec[u8]) {
    encode_varint(s.len() as u64, buf)
    buf.extend_from_slice(s.as_bytes())
}

// XOR cipher — self-inverse: the compiler detects and reports this
@reversible
fn xor_cipher(data: &mut Vec[u8], key: &[]u8)
    // Compiler note: this function is its own inverse (f(f(x)) == x)
    // generates: xor_cipher.inverse == xor_cipher
{
    for (b, k) in data.iter_mut().zip(key.iter().cycle()) {
        *b ^= k
    }
}

// ─── Structured codec for a real protocol ────────────────────────────────────

type MsgType = enum {
    Ping     = 0x01,
    Pong     = 0x02,
    Data     = 0x10,
    DataAck  = 0x11,
    Error    = 0xFF,
}

type Message = enum {
    Ping  { seq: u32 },
    Pong  { seq: u32, latency_us: u64 },
    Data  { stream_id: u32, offset: u64, payload: Vec[u8> },
    DataAck { stream_id: u32, offset: u64 },
    Error { code: u32, message: str },
}

// Compiler generates decode_message from this encoder definition
@reversible
fn encode_message(msg: &Message, buf: &mut Vec[u8]) {
    match msg {
        Message.Ping { seq } => {
            buf.push(MsgType.Ping as u8)
            encode_u32_le(*seq, buf)
        },
        Message.Pong { seq, latency_us } => {
            buf.push(MsgType.Pong as u8)
            encode_u32_le(*seq, buf)
            encode_u64_le(*latency_us, buf)
        },
        Message.Data { stream_id, offset, payload } => {
            buf.push(MsgType.Data as u8)
            encode_u32_le(*stream_id, buf)
            encode_u64_le(*offset, buf)
            encode_varint(payload.len() as u64, buf)
            buf.extend_from_slice(payload)
        },
        Message.DataAck { stream_id, offset } => {
            buf.push(MsgType.DataAck as u8)
            encode_u32_le(*stream_id, buf)
            encode_u64_le(*offset, buf)
        },
        Message.Error { code, message } => {
            buf.push(MsgType.Error as u8)
            encode_u32_le(*code, buf)
            encode_string(message, buf)
        },
    }
}

// ─── Streaming codec with framing ─────────────────────────────────────────────

type FrameHeader = {
    magic:   u32,   // 0xAE_NG_00_01
    len:     u32,   // payload length
    crc32:   u32,   // payload checksum
    version: u8,
    flags:   u8,
    _pad:    [2]u8,
}

@reversible
fn encode_frame(payload: &[]u8, flags: u8, buf: &mut Vec[u8]) {
    let crc = ng.serial.crc32(payload)
    encode_u32_le(0xAE_NG_00_01, buf)   // magic
    encode_u32_le(payload.len() as u32, buf)
    encode_u32_le(crc, buf)
    buf.push(1u8)                        // version
    buf.push(flags)
    buf.push(0u8); buf.push(0u8)         // padding
    buf.extend_from_slice(payload)
}

// Full encode/decode round-trip (compiler-generated decode_frame is called here)
fn roundtrip_test(msg: Message) -> Result[Message, CodecError> {
    var buf = Vec[u8].new()
    encode_message(&msg, &mut buf)

    var frame_buf = Vec[u8].new()
    encode_frame(&buf, 0, &mut frame_buf)

    // decode_frame: compiler-generated from encode_frame
    let (payload, _rest) = decode_frame(&frame_buf)?
    // decode_message: compiler-generated from encode_message
    let (decoded, _rest2) = decode_message(&payload)?

    Ok(decoded)
}

// ─── Codec composition ────────────────────────────────────────────────────────

// A reversible transformation chain
// Each step's inverse is auto-generated; composition is also reversible.
@reversible
fn compress_and_encode_message(msg: &Message, buf: &mut Vec[u8]) {
    var inner = Vec[u8].new()
    encode_message(msg, &mut inner)
    let compressed = ng.compress.lz4_compress(&inner)
    encode_varint(compressed.len() as u64, buf)
    buf.extend_from_slice(&compressed)
}

// Compiler generates:
// fn decompress_and_decode_message(buf: &[]u8) -> Result[(Message, &[]u8), CodecError>

type CodecError = enum {
    UnknownMsgType(u8),
    Truncated,
    BadUtf8,
    ChecksumMismatch,
    BadMagic,
    BadVersion(u8),
    InvalidLength(usize),
}
```

---

## Program 11: Live Invariants — Financial Ledger with Conservation Laws

`inv always:` is proven at every mutation point. Zero runtime cost in release.
Total supply is a system invariant — any code path that violates it won't compile.

```ng
module app.ledger

use ng.proof.{requires, ensures, inv}
use ng.time.{Timestamp}
use ng.cap.ClockCap
use ng.core.{Result}
use ng.collections.{HashMap, Vec}
use ng.sync.RwLock

// ─── Account ──────────────────────────────────────────────────────────────────

type AccountId = u64
type Satoshi   = i64   // 1e-8 units, like Bitcoin

type Account = {
    id:        AccountId,
    owner:     str,
    balance:   Satoshi,
    frozen:    bool,
    created:   Timestamp,
    inv always: balance >= 0      // no overdraft — ever
    inv always: !owner.is_empty() // owner always set
}

// COMPILE ERROR — the compiler proves this violates the invariant:
//
// fn cheat(acc: &mut Account, amount: Satoshi) {
//     acc.balance -= amount   // ERROR: cannot prove balance >= 0 after this
// }

impl Account {
    fn new(id: AccountId, owner: str, initial: Satoshi) -> Self
        requires initial >= 0
        requires !owner.is_empty()
    {
        Account { id, owner, balance: initial, frozen: false, created: Timestamp.epoch() }
    }

    // deposit: always increases balance, invariant trivially holds
    fn deposit(self: &mut Self, amount: Satoshi)
        requires amount > 0
        ensures  self.balance == old(self.balance) + amount
    {
        self.balance += amount
    }

    // withdraw: compiler checks the guard before mutation
    fn withdraw(self: &mut Self, amount: Satoshi) -> Result[(), LedgerError]
        requires amount > 0
        ensures  match result {
            Ok(())  => self.balance == old(self.balance) - amount,
            Err(_)  => self.balance == old(self.balance),
        }
    {
        if self.frozen {
            return Err(LedgerError.AccountFrozen(self.id))
        }
        if self.balance < amount {
            return Err(LedgerError.InsufficientFunds { have: self.balance, need: amount })
        }
        // Z3 proves: self.balance - amount >= 0 because of the guard above
        self.balance -= amount
        Ok(())
    }
}

// ─── Ledger — system-level invariant ─────────────────────────────────────────

type TransactionKind = enum {
    Transfer { from: AccountId, to: AccountId, amount: Satoshi },
    Mint     { to: AccountId, amount: Satoshi },
    Burn     { from: AccountId, amount: Satoshi },
}

type Transaction = {
    id:        u64,
    kind:      TransactionKind,
    timestamp: Timestamp,
    memo:      str,
}

type Ledger = {
    accounts:      HashMap[AccountId, Account],
    tx_log:        Vec[Transaction],
    total_supply:  Satoshi,
    next_tx_id:    u64,
    inv always: total_supply >= 0
    inv always: total_supply == accounts.values().map(|a| a.balance).sum()
    inv always: accounts.values().all(|a| a.balance >= 0)
}
// The system invariant: total_supply == sum of all balances
// The compiler checks this at EVERY mutation of accounts or total_supply.
// If any code path can violate it, it won't compile.

impl Ledger {
    fn new(initial_supply: Satoshi, treasury_owner: str) -> Self
        requires initial_supply >= 0
    {
        let treasury_id = 1u64
        let treasury = Account.new(treasury_id, treasury_owner, initial_supply)
        var accounts = HashMap.new()
        accounts.insert(treasury_id, treasury)
        Ledger {
            accounts,
            tx_log:       Vec.new(),
            total_supply: initial_supply,
            next_tx_id:   1,
        }
    }

    // Transfer: total supply unchanged, just moved between accounts
    fn transfer(
        self: &mut Self,
        from: AccountId,
        to:   AccountId,
        amount: Satoshi,
        memo:   str,
        @cap clock: ClockCap,
    ) -> Result[u64, LedgerError]
        requires from != to
        requires amount > 0
        // postcondition: total supply conserved
        ensures match result {
            Ok(_)  => self.total_supply == old(self.total_supply),
            Err(_) => self.total_supply == old(self.total_supply)
                   && self.accounts == old(self.accounts),
        }
    {
        if !self.accounts.contains_key(&from) { return Err(LedgerError.NoSuchAccount(from)) }
        if !self.accounts.contains_key(&to)   { return Err(LedgerError.NoSuchAccount(to)) }

        // snapshot balances for rollback (Z3 uses these for proof)
        let from_old = self.accounts[&from].balance
        let to_old   = self.accounts[&to].balance

        self.accounts.get_mut(&from).unwrap().withdraw(amount)?
        self.accounts.get_mut(&to).unwrap().deposit(amount)

        // Z3 verifies: from.balance + to.balance unchanged
        // i.e. (from_old - amount) + (to_old + amount) == from_old + to_old ✓

        let tx_id = self.next_tx_id
        self.next_tx_id += 1
        self.tx_log.push(Transaction {
            id:        tx_id,
            kind:      TransactionKind.Transfer { from, to, amount },
            timestamp: clock.now(),
            memo,
        })
        Ok(tx_id)
    }

    // Mint: creates supply — total_supply increases
    fn mint(
        self: &mut Self,
        to:     AccountId,
        amount: Satoshi,
        memo:   str,
        @cap clock: ClockCap,
    ) -> Result[u64, LedgerError]
        requires amount > 0
        ensures match result {
            Ok(_)  => self.total_supply == old(self.total_supply) + amount,
            Err(_) => self.total_supply == old(self.total_supply),
        }
    {
        if !self.accounts.contains_key(&to) { return Err(LedgerError.NoSuchAccount(to)) }

        self.accounts.get_mut(&to).unwrap().deposit(amount)
        self.total_supply += amount   // Z3: invariant still holds after this pair

        let tx_id = self.next_tx_id
        self.next_tx_id += 1
        self.tx_log.push(Transaction {
            id:        tx_id,
            kind:      TransactionKind.Mint { to, amount },
            timestamp: clock.now(),
            memo,
        })
        Ok(tx_id)
    }

    // Burn: destroys supply
    fn burn(
        self: &mut Self,
        from:   AccountId,
        amount: Satoshi,
        memo:   str,
        @cap clock: ClockCap,
    ) -> Result[u64, LedgerError]
        requires amount > 0
        ensures match result {
            Ok(_)  => self.total_supply == old(self.total_supply) - amount,
            Err(_) => self.total_supply == old(self.total_supply),
        }
    {
        if !self.accounts.contains_key(&from) { return Err(LedgerError.NoSuchAccount(from)) }

        self.accounts.get_mut(&from).unwrap().withdraw(amount)?
        self.total_supply -= amount   // Z3: total_supply - amount >= 0 (proven from account invariant)

        let tx_id = self.next_tx_id
        self.next_tx_id += 1
        self.tx_log.push(Transaction {
            id:        tx_id,
            kind:      TransactionKind.Burn { from, amount },
            timestamp: clock.now(),
            memo,
        })
        Ok(tx_id)
    }

    fn create_account(
        self: &mut Self,
        id:    AccountId,
        owner: str,
        @cap clock: ClockCap,
    ) -> Result[(), LedgerError] {
        if self.accounts.contains_key(&id) {
            return Err(LedgerError.DuplicateAccount(id))
        }
        // new account starts with 0 balance — total supply unchanged
        self.accounts.insert(id, Account.new(id, owner, 0))
        Ok(())
    }

    fn audit_report(self) -> AuditReport {
        let computed_supply: Satoshi = self.accounts.values()
            .map(|a| a.balance)
            .sum()
        // this assert is always true at compile time (invariant) —
        // it's here as documentation
        assert_proof(computed_supply == self.total_supply)

        AuditReport {
            total_supply:    self.total_supply,
            account_count:   self.accounts.len() as u64,
            transaction_count: self.tx_log.len() as u64,
            top_balances:    self.top_n_accounts(10),
        }
    }

    fn top_n_accounts(self, n: usize) -> Vec[(AccountId, Satoshi)> {
        var sorted: Vec[(&AccountId, &Account)> = self.accounts.iter().collect()
        sorted.sort_by(|(_, a), (_, b)| b.balance.cmp(&a.balance))
        sorted[..n.min(sorted.len())]
            .iter()
            .map(|(id, acc)| (**id, acc.balance))
            .collect()
    }
}

type LedgerError = enum {
    NoSuchAccount(AccountId),
    DuplicateAccount(AccountId),
    InsufficientFunds { have: Satoshi, need: Satoshi },
    AccountFrozen(AccountId),
}

type AuditReport = {
    total_supply:      Satoshi,
    account_count:     u64,
    transaction_count: u64,
    top_balances:      Vec[(AccountId, Satoshi)>,
}
```

---

## Program 12: Causal Types — Medical Records with Automatic Audit Trail

Every value knows where it came from. Mixing data sources requires explicit acknowledgement.
The compiler generates a lineage graph as a build artifact.

```ng
module app.emr   // Electronic Medical Record

use ng.causal.{Caused, @causal, @acknowledge_source, lineage}
use ng.flow.{Secret, Trusted, Public, @declassify}
use ng.time.Timestamp
use ng.cap.{FileCap, ClockCap}
use ng.core.{Result, Option}
use ng.collections.{Vec, HashMap}

// ─── Data sources (causal labels) ────────────────────────────────────────────

// Every data item carries its provenance:
// "lab"         = lab test machine result
// "physician"   = doctor manually entered
// "nurse"       = nurse measurement
// "device"      = connected monitoring device
// "admin"       = administrative data (name, DOB, insurance)
// "patient"     = patient-reported data

type PatientId = u64

// Lab result: source is "lab" — only lab systems can produce this type
type LabValue = Caused[f64, from: "lab"]

// Vital sign: from a device or nurse
type VitalSign = Caused[f64, from: {"device", "nurse"}>

// Physician order: only physicians can create
type PhysicianOrder = Caused[str, from: "physician">

// ─── Patient record ───────────────────────────────────────────────────────────

type BloodPressure = {
    systolic:  VitalSign,
    diastolic: VitalSign,
    timestamp: Timestamp,
    recorded_by: str,
}

type LabResult = {
    test_name:   str,
    value:       LabValue,
    unit:        str,
    ref_lo:      f64,
    ref_hi:      f64,
    timestamp:   Timestamp,
    lab_id:      str,
    verified_by: str,
}

type Medication = {
    name:        str,
    dose_mg:     f64,
    frequency:   str,
    prescribed:  PhysicianOrder,   // cannot be entered by anyone other than physician
    started_at:  Timestamp,
    ended_at:    Option[Timestamp],
}

type Diagnosis = {
    icd10:       Caused[str, from: "physician">,
    description: str,
    confidence:  Caused[f64, from: "physician">,
    timestamp:   Timestamp,
}

type PatientRecord = {
    patient_id:   PatientId,
    // admin data (Protected Health Information)
    name:         Secret[Caused[str, from: "admin">],
    dob:          Secret[Caused[Timestamp, from: "admin">],
    // clinical data
    vitals:       Vec[BloodPressure],
    lab_results:  Vec[LabResult],
    medications:  Vec[Medication],
    diagnoses:    Vec[Diagnosis],
    // patient-reported symptoms
    symptoms:     Vec[Caused[str, from: "patient">],
    // last modified
    updated_at:   Timestamp,
}

// ─── Clinical decision support ────────────────────────────────────────────────

type RiskLevel = enum { Low, Moderate, High, Critical }

// Risk score: can only be derived from lab + device data, NOT from patient-reported
@causal
fn compute_cvd_risk(
    cholesterol:   LabValue,
    blood_pressure: &BloodPressure,
    age:            Caused[f64, from: "admin">,
) -> Caused[f64, from: {"lab", "device", "nurse", "admin">]
    // return type inherits all input sources
{
    let tc  = *cholesterol
    let sbp = *blood_pressure.systolic
    let a   = *age

    let score = 1.0 / (1.0 + ng.math.exp(-(
        -5.9 + 0.072 * a + 0.017 * tc + 0.009 * sbp
    )))
    // result is Caused[f64, from: {"lab","device","nurse","admin"}]
    // automatically — the compiler tracks this
    Caused(score)
}

// This is a COMPILE ERROR — you cannot use patient-reported data in a
// clinical risk score without explicit acknowledgement:
//
// @causal
// fn bad_risk(symptom: Caused[str, from: "patient">, lab: LabValue)
//     -> Caused[f64, from: "lab">   // ERROR: "patient" source not in return type
// {
//     let parsed_severity = parse_severity(&symptom)
//     Caused(*lab * parsed_severity)
//     // ERROR: result depends on "patient" source but return type only lists "lab"
// }

// If you genuinely need patient-reported data in a clinical score, you must
// explicitly acknowledge it — this appears in the audit report:
@causal
@acknowledge_source("patient", reason: "validated symptom scale, attending physician reviewed")
fn enhanced_risk(
    lab_score:  Caused[f64, from: {"lab", "device", "nurse", "admin">],
    symptom_severity: Caused[f64, from: "patient">,
) -> Caused[f64, from: {"lab", "device", "nurse", "admin", "patient">]
{
    Caused(*lab_score * 0.9 + *symptom_severity * 0.1)
}

// ─── Drug interaction checker ─────────────────────────────────────────────────

type InteractionSeverity = enum { None, Mild, Moderate, Severe, Contraindicated }

type DrugInteraction = {
    drug_a:   str,
    drug_b:   str,
    severity: InteractionSeverity,
    mechanism: str,
    source:   Caused[str, from: "lab">,  // evidence from clinical literature
}

@causal
fn check_interactions(
    medications: &Vec[Medication],
    db:          &InteractionDatabase,
) -> Vec[Caused[DrugInteraction, from: {"physician", "lab">]]
{
    var found = Vec.new()
    for i in 0..medications.len() {
        for j in (i+1)..medications.len() {
            let a = &medications[i]
            let b = &medications[j]
            if let Some(interaction) = db.lookup(&a.name, &b.name) {
                // the interaction record carries "lab" provenance (clinical DB)
                // the medication names carry "physician" provenance
                // result: Caused[..., from: {"physician", "lab"}]
                found.push(Caused(interaction))
            }
        }
    }
    found
}

// ─── Export (audit-safe) ──────────────────────────────────────────────────────

// Clinical summary: excludes PII, includes only verified clinical data
type ClinicalSummary = {
    patient_id:    PatientId,               // not secret (de-identified)
    latest_vitals: Option[BloodPressure],
    active_meds:   Vec[str],                // drug names only, not full Medication
    diagnoses:     Vec[str],                // ICD10 codes only
    risk_score:    Option[Caused[f64, from: {"lab", "device", "nurse", "admin">]>,
    // explicitly: NO patient name, NO DOB, NO symptoms (patient-reported)
}

// When you generate a ClinicalSummary, the compiler can output the full
// data lineage graph: which sources contributed to each field, when each
// data point was recorded, and who entered it.
// This is a build-time artifact: clinical_lineage_report_<timestamp>.json

@declassify(Secret -> Trusted, reason: "de-identified for research export, IRB approved")
fn export_for_research(
    record: &PatientRecord,
    @cap clock: ClockCap,
) -> ClinicalSummary {
    ClinicalSummary {
        patient_id:    record.patient_id,
        latest_vitals: record.vitals.last().cloned(),
        active_meds:   record.medications
                          .iter()
                          .filter(|m| m.ended_at.is_none())
                          .map(|m| m.name.clone())
                          .collect(),
        diagnoses:     record.diagnoses
                          .iter()
                          .map(|d| d.icd10.inner().clone())
                          .collect(),
        risk_score:    None,   // computed separately if needed
    }
}

type InteractionDatabase = {}
impl InteractionDatabase {
    fn lookup(self, _a: &str, _b: &str) -> Option[DrugInteraction] { None }
}
```

---

*13 programs. Each one shows what the compiler proves, prevents, and generates.*
*No feature can be faked — they all have structural consequences in the type system.*
