# nbvm — ng Bootstrap Virtual Machine
# Specification v0.1

# nbvm este mașina virtuală minimală executată de seed.bin (~300 bytes).
# Scopul ei este să permită scrierea stage1 (tokenizer + parser minimal)
# fără a depinde de niciun compilator existent.
#
# Design principles:
# - Stack-based (simplu de implementat în ~200 bytes de x86-64)
# - Word size = 64 bits pe tot (fără conversii)
# - Memorie plată (flat) — VM-ul vede un spațiu continuu de bytes
# - Syscalls directe — nu există "runtime", totul merge prin OS
# - Fișierul bytecode = header + code section + data section

# ─────────────────────────────────────────────────────────────
# 1. MEMORY LAYOUT
# ─────────────────────────────────────────────────────────────
#
#  Address space (64-bit flat):
#
#  0x0000_0000  ┌─────────────────────────┐
#               │  bytecode image         │  loaded by seed at startup
#               │  (code + data)          │
#  image_end    ├─────────────────────────┤
#               │  heap (bump allocator)  │  grows up
#               │  ...                    │
#  hp →         ├─────────────────────────┤
#               │  (free)                 │
#               ├─────────────────────────┤
#  sp →         │  VM stack               │  grows down
#               │  (1 MB default)         │
#  stack_base   └─────────────────────────┘
#
# Registers (conceptual — mapped to x86-64 regs in seed):
#   ip  — instruction pointer  (r14)
#   sp  — stack pointer        (r15)
#   fp  — frame pointer        (r13)  for local variable frames
#   hp  — heap pointer         (r12)  bump allocator head

# ─────────────────────────────────────────────────────────────
# 2. FILE FORMAT
# ─────────────────────────────────────────────────────────────
#
# bytecode file layout:
#
# offset  size  field
# ──────  ────  ─────────────────────────────────
# 0       4     magic: 0x6E 0x67 0x62 0x63  ("ngbc")
# 4       2     version: 0x00 0x01
# 6       2     flags: bit0=has_data, bit1=debug_info
# 8       8     entry_offset: offset of first instruction (from byte 0)
# 16      8     code_size: bytes of code section
# 24      8     data_offset: offset of data section
# 32      8     data_size: bytes of data section
# 40      8     stack_size: requested VM stack size (default 1MB)
# 48      8     heap_size: requested heap size (default 4MB)
# 56      8     reserved: 0
# ── 64 bytes total header ──
#
# code section: raw instruction bytes, starts at entry_offset
# data section: read-only bytes (string literals, constants)

# ─────────────────────────────────────────────────────────────
# 3. INSTRUCTION SET (32 opcodes)
# ─────────────────────────────────────────────────────────────
#
# Encoding: 1 byte opcode + optional immediate
# All immediates are little-endian.
# Stack values are 64-bit unsigned (interpreted as signed by arithmetic ops).
#
# Notation: [a, b, ...] = stack before (top first)
#           → [c, ...]   = stack after

# ── 3.1 STACK MANIPULATION ──────────────────────────────────

# 0x00  NOP
#   no operation
NOP

# 0x01  PUSH_I64  <i64: 8 bytes>
#   [] → [v]
#   Push 64-bit signed immediate.
PUSH_I64

# 0x02  PUSH_I8   <i8: 1 byte>
#   [] → [sign_extended(v)]
#   Push sign-extended 8-bit immediate. (common, saves space)
PUSH_I8

# 0x03  POP
#   [a] → []
POP

# 0x04  DUP
#   [a] → [a, a]
DUP

# 0x05  SWAP
#   [a, b] → [b, a]
SWAP

# 0x06  OVER
#   [a, b] → [b, a, b]   (copy second-from-top)
OVER

# ── 3.2 ARITHMETIC ─────────────────────────────────────────

# 0x07  ADD    [a, b] → [a+b]
ADD

# 0x08  SUB    [a, b] → [a-b]   (a = second from top, b = top)
SUB

# 0x09  MUL    [a, b] → [a*b]
MUL

# 0x0A  DIV    [a, b] → [a/b]   signed division; div-by-zero → halt
DIV

# 0x0B  MOD    [a, b] → [a%b]   signed remainder
MOD

# 0x0C  AND    [a, b] → [a&b]   bitwise AND
AND

# 0x0D  OR     [a, b] → [a|b]   bitwise OR
OR

# 0x0E  XOR    [a, b] → [a^b]
XOR

# 0x0F  NOT    [a] → [~a]       bitwise NOT
NOT

# 0x10  SHL    [a, b] → [a<<b]  shift left by b (mod 64)
SHL

# 0x11  SHR    [a, b] → [a>>b]  logical shift right
SHR

# 0x12  SAR    [a, b] → [a>>b]  arithmetic shift right (sign-extending)
SAR

# ── 3.3 COMPARISON ─────────────────────────────────────────
# All comparison ops push 1 (true) or 0 (false)

# 0x13  EQ     [a, b] → [a==b ? 1 : 0]
EQ

# 0x14  LT     [a, b] → [a<b  ? 1 : 0]   signed
LT

# 0x15  GT     [a, b] → [a>b  ? 1 : 0]   signed
GT

# 0x16  LTU    [a, b] → [a<b  ? 1 : 0]   unsigned
LTU

# ── 3.4 CONTROL FLOW ───────────────────────────────────────

# 0x17  JUMP   <i32: 4 bytes>
#   ip += offset (relative to byte AFTER the JUMP instruction)
#   Unconditional.
JUMP

# 0x18  JUMP_Z   <i32: 4 bytes>
#   [a] → []
#   if a == 0: ip += offset
JUMP_Z

# 0x19  JUMP_NZ  <i32: 4 bytes>
#   [a] → []
#   if a != 0: ip += offset
JUMP_NZ

# 0x1A  CALL   <i32: 4 bytes>
#   [] → []
#   Push ip (after CALL) onto call stack, then ip += offset.
#   Call stack is separate from data stack (inside seed impl).
CALL

# 0x1B  RET
#   [] → []
#   Pop address from call stack, jump there.
RET

# 0x1C  CALL_IND
#   [addr] → []
#   Like CALL but target address comes from stack. For function pointers.
CALL_IND

# ── 3.5 MEMORY ─────────────────────────────────────────────

# 0x1D  LOAD8    [addr] → [zero_extend(mem[addr])]
LOAD8

# 0x1E  LOAD64   [addr] → [mem64[addr]]
LOAD64

# 0x1F  STORE8   [addr, val] → []    mem[addr] = val & 0xFF
STORE8

# 0x20  STORE64  [addr, val] → []    mem64[addr] = val
STORE64

# 0x21  ALLOC    [size] → [addr]
#   Bump-allocate `size` bytes. Returns address.
#   Alignment: always 8-byte aligned.
ALLOC

# ── 3.6 LOCAL VARIABLES (frame-based) ─────────────────────

# 0x22  ENTER  <u8: local_count>
#   Push frame: allocates local_count*8 bytes on data stack.
#   fp = sp (before allocation).
#   Locals addressed as fp - 8*index.
ENTER

# 0x23  LEAVE
#   Pop frame: restores sp to before ENTER, pops fp.
LEAVE

# 0x24  LOAD_LOCAL  <u8: index>
#   [] → [locals[index]]
LOAD_LOCAL

# 0x25  STORE_LOCAL  <u8: index>
#   [val] → []
#   locals[index] = val
STORE_LOCAL

# ── 3.7 SYSTEM ─────────────────────────────────────────────

# 0x26  SYSCALL  <u8: syscall_id>
#   Arguments on stack (top = first arg).
#   Return value pushed onto stack.
#   Standard syscall IDs:
#     0 = EXIT   (code)              → never returns
#     1 = READ   (fd, buf, len)      → bytes_read
#     2 = WRITE  (fd, buf, len)      → bytes_written
#     3 = OPEN   (path, flags)       → fd or -1
#     4 = CLOSE  (fd)                → 0 or -1
#     5 = MMAP   (size)              → addr (anonymous, RW)
#     6 = TIME   ()                  → unix timestamp (seconds)
SYSCALL

# 0x27  HALT
#   Exit the VM immediately with code 0.
HALT

# ─────────────────────────────────────────────────────────────
# 4. CALLING CONVENTION
# ─────────────────────────────────────────────────────────────
#
# Before CALL:
#   Push arguments left-to-right onto stack.
#   (First arg deepest, last arg on top before CALL)
#
# After CALL (inside callee):
#   ENTER N  — allocates N locals, saves fp
#   Arguments are accessible as locals:
#     local[0] = first arg
#     local[1] = second arg
#     ...
#
# Return value:
#   Push onto stack before RET.
#   Caller receives it on top of stack after CALL returns.
#
# Example — function: add(a, b) -> a+b
#
#   ; callee
#   ENTER 2         ; 2 locals: a, b
#   STORE_LOCAL 0   ; a = pop
#   STORE_LOCAL 1   ; b = pop
#   LOAD_LOCAL 0
#   LOAD_LOCAL 1
#   ADD
#   LEAVE
#   RET
#
#   ; caller
#   PUSH_I8 3       ; a = 3
#   PUSH_I8 4       ; b = 4
#   CALL <offset_to_add>
#   ; top of stack = 7

# ─────────────────────────────────────────────────────────────
# 5. STANDARD LIBRARY (built-in calls via SYSCALL)
# ─────────────────────────────────────────────────────────────
#
# Syscall 1 — READ(fd, buf_addr, len) -> bytes_read
#   fd=0 for stdin.
#
# Syscall 2 — WRITE(fd, buf_addr, len) -> bytes_written
#   fd=1 stdout, fd=2 stderr.
#
# Syscall 5 — MMAP(size) -> addr
#   Allocates anonymous page-aligned memory.
#   Used by ALLOC for large allocations.
#
# The seed maps these to:
#   Linux:   read(2), write(2), mmap(2), exit(2)
#   Windows: ReadFile, WriteFile, VirtualAlloc, ExitProcess

# ─────────────────────────────────────────────────────────────
# 6. EXAMPLE BYTECODE PROGRAM
# ─────────────────────────────────────────────────────────────
#
# Program: print "hello\n" to stdout and exit.
# (Assembly notation — actual file would be raw bytes)
#
# .data
#   hello_str:  68 65 6C 6C 6F 0A   ; "hello\n"
#
# .code
# main:
#   PUSH_I64 <addr_of_hello_str>  ; buf
#   PUSH_I8  6                    ; len = 6
#   PUSH_I8  1                    ; fd = 1 (stdout)
#   SYSCALL  2                    ; write(fd, buf, len)
#   POP                           ; discard return value
#   PUSH_I8  0                    ; exit code 0
#   SYSCALL  0                    ; exit(0)
#
# Raw bytes (after resolving addr_of_hello_str = 0x40 + 64 = 0x58):
#   Header:  6E 67 62 63  00 01  00 00
#            48 00 00 00 00 00 00 00  ; entry = 0x48 (72)
#            1C 00 00 00 00 00 00 00  ; code_size = 28 bytes
#            64 00 00 00 00 00 00 00  ; data_offset = 100
#            06 00 00 00 00 00 00 00  ; data_size = 6
#            00 00 10 00 00 00 00 00  ; stack_size = 1MB
#            00 00 40 00 00 00 00 00  ; heap_size  = 4MB
#            00 00 00 00 00 00 00 00  ; reserved
#
#   Code:    01 64 00 00 00 00 00 00 00  ; PUSH_I64 0x64 (data addr)
#            02 06                       ; PUSH_I8  6
#            02 01                       ; PUSH_I8  1
#            26 02                       ; SYSCALL  2
#            03                          ; POP
#            02 00                       ; PUSH_I8  0
#            26 00                       ; SYSCALL  0
#
#   Data:    68 65 6C 6C 6F 0A           ; "hello\n"

# ─────────────────────────────────────────────────────────────
# 7. SEED IMPLEMENTATION SKETCH (x86-64 Linux)
# ─────────────────────────────────────────────────────────────
#
# The seed is a ~300-byte ELF binary that:
# 1. Reads the bytecode file from stdin (or argv[1])
# 2. mmaps memory for stack + heap
# 3. Dispatches the instruction loop
#
# Register allocation in seed:
#   r14 = ip  (instruction pointer into bytecode buffer)
#   r15 = sp  (VM stack pointer, grows down)
#   r13 = fp  (frame pointer)
#   r12 = hp  (heap pointer, grows up)
#   rbx = base address of bytecode image
#
# Dispatch loop (pseudocode → becomes raw bytes):
#
#   loop:
#     al = *r14++           ; fetch opcode
#     jmp [dispatch + rax*8] ; jump to handler
#
# Handler table (64 entries × 8 bytes = 512 bytes —
# but we pack handlers inline to stay under 512 bytes total):
#
#   NOP:    jmp loop
#   PUSH_I64: r15 -= 8; *(u64*)r15 = *(i64*)r14; r14 += 8; jmp loop
#   PUSH_I8:  r15 -= 8; *(u64*)r15 = sign_extend(*r14); r14++; jmp loop
#   POP:    r15 += 8; jmp loop
#   DUP:    r15 -= 8; *(u64*)r15 = *(u64*)(r15+8); jmp loop
#   SWAP:   t=*(r15); *(r15)=*(r15+8); *(r15+8)=t; jmp loop
#   ADD:    a=pop; b=pop; push(a+b); jmp loop
#   ...
#   SYSCALL: id = *r14++
#     id==0: mov rdi,[r15]; mov rax,60; syscall  ; exit
#     id==1: mov rdx,[r15]; mov rsi,[r15+8]; mov rdi,[r15+16]; mov rax,0; syscall; r15+=24; push(rax)
#     id==2: mov rdx,[r15]; mov rsi,[r15+8]; mov rdi,[r15+16]; mov rax,1; syscall; r15+=24; push(rax)
#     id==5: ... mmap ...
#   HALT: mov rdi,0; mov rax,60; syscall

# ─────────────────────────────────────────────────────────────
# 8. OPCODE TABLE (summary)
# ─────────────────────────────────────────────────────────────
#
# Hex   Mnemonic     Immediates   Stack effect
# ────  ───────────  ──────────   ────────────────────
# 0x00  NOP          —            —
# 0x01  PUSH_I64     i64(8)       → [v]
# 0x02  PUSH_I8      i8(1)        → [v]
# 0x03  POP          —            [a] →
# 0x04  DUP          —            [a] → [a,a]
# 0x05  SWAP         —            [a,b] → [b,a]
# 0x06  OVER         —            [a,b] → [b,a,b]
# 0x07  ADD          —            [a,b] → [a+b]
# 0x08  SUB          —            [a,b] → [a-b]
# 0x09  MUL          —            [a,b] → [a*b]
# 0x0A  DIV          —            [a,b] → [a/b]
# 0x0B  MOD          —            [a,b] → [a%b]
# 0x0C  AND          —            [a,b] → [a&b]
# 0x0D  OR           —            [a,b] → [a|b]
# 0x0E  XOR          —            [a,b] → [a^b]
# 0x0F  NOT          —            [a] → [~a]
# 0x10  SHL          —            [a,b] → [a<<b]
# 0x11  SHR          —            [a,b] → [a>>b]
# 0x12  SAR          —            [a,b] → [a>>b signed]
# 0x13  EQ           —            [a,b] → [a==b]
# 0x14  LT           —            [a,b] → [a<b signed]
# 0x15  GT           —            [a,b] → [a>b signed]
# 0x16  LTU          —            [a,b] → [a<b unsigned]
# 0x17  JUMP         i32(4)       —
# 0x18  JUMP_Z       i32(4)       [a] →
# 0x19  JUMP_NZ      i32(4)       [a] →
# 0x1A  CALL         i32(4)       —
# 0x1B  RET          —            —
# 0x1C  CALL_IND     —            [addr] →
# 0x1D  LOAD8        —            [addr] → [byte]
# 0x1E  LOAD64       —            [addr] → [u64]
# 0x1F  STORE8       —            [addr,v] →
# 0x20  STORE64      —            [addr,v] →
# 0x21  ALLOC        —            [size] → [addr]
# 0x22  ENTER        u8(1)        —
# 0x23  LEAVE        —            —
# 0x24  LOAD_LOCAL   u8(1)        → [v]
# 0x25  STORE_LOCAL  u8(1)        [v] →
# 0x26  SYSCALL      u8(1)        (varies)
# 0x27  HALT         —            never returns
#
# Total: 40 opcodes (0x00–0x27)
# (fits in one dispatch table of 64 slots, rest are HALT)
