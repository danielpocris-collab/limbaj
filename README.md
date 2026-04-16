# Limbaj

Selfhosted compiler worktree on the `ng -> Axiom` path.

## Canonical Path

This workspace is selfhost-only.

- Trust root: `ng_selfhost_clean.exe`
- Compiler under active work: `ng_native.ng`
- Canonical verification entrypoints:
  - `powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1`
  - `powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1`
  - `powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -MaxParallel 2`

The old Rust host path was removed from this worktree on `2026-04-16` to eliminate dual-entry confusion.

## Current Checkpoint

- Fixed point: `ng_selfhost_clean.exe == output.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe`
- Size: `161280`
- SHA-256: `25DCECD94D17BBBF1BBCFFE36EC009F19BC7E5B7094C061D70E56A0FA3541012`

## Active Documents

- [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md)
- [BUG_FREE_ACCEPTANCE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_FREE_ACCEPTANCE.md)
- [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md)
- [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md)
- [NG_TO_AXIOM_ROADMAP.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/NG_TO_AXIOM_ROADMAP.md)
- [BUG_HISTORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_HISTORY.md)
- [ERROR_CATALOG.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ERROR_CATALOG.md)
- [BOOTSTRAP_REPRO_AUDIT.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BOOTSTRAP_REPRO_AUDIT.md)

## What To Run

Fast sanity:

```powershell
powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1
```

Direct selfhost stack:

```powershell
powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1
```

- runs the canonical leaf suites in separate selfhost invocations
- performs the large `ng_native.ng` fixed-point compare directly in PowerShell against the compiler under test

Full bootstrap vertical:

```powershell
powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -MaxParallel 2
```

Direct compile from trusted checkpoint:

```powershell
.\ng_selfhost_clean.exe ng_native.ng
```

## Current Focus

The current official next step is Block 1 hardening on the large `ng_native.ng -> ng_native.ng` path:

- `validate_final_layout_contract(...)`
- patch / rdata / import validation
- removal of the last direct CSV lookups and scans from the hot path
- freeze the supported core, TCB and bug-free gates in the new canon docs before any strong correctness claim
- keep bootstrap/direct build dirs ephemeral and reset inside the workspace before each canonical build

## Notes

- `README.md` is a quick operator guide, not the detailed audit log.
- `STATUS.md` is the active source of truth for current state and next steps.
- `BUG_FREE_ACCEPTANCE.md` defines when `bug-free` can be said onest and on what surface.
- `TCB.md` lists the current trust base and what still has to leave it.
- `SEMANTICS_CORE.md` fixes the semantic surface that future proofs and validators must preserve.
- Historical documents may still mention the retired Rust host path; treat those references as archival only.
