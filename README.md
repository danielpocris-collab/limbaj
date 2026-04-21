# Next Gen

Self-hosted compiler worktree for `Next Gen`.

`Axiom` is the target direction for the project, not the public name of the language.

## Project Mission

The goal of this project is not only to make the language safer in isolation.

The goal is to build a system that eliminates bugs across the entire declared surface:

- program logic
- compiler
- runtime
- toolchain
- external boundaries
- concurrency
- time
- distribution
- security

In the canonical documents, every strong correctness claim must still be read modulo the explicit TCB and the declared supported surface.

## Canonical Path

This workspace is selfhost-only.

- Trust root: `ng_selfhost_clean.exe`
- Compiler under active work: `ng_native.ng`
- Canonical verification entrypoints:
  - `powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1`
  - `powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1`
  - `powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -MaxParallel 2`

The old Rust host path was removed from this worktree on `2026-04-16` to eliminate dual-entry confusion.
If `ng_selfhost_clean.exe` is missing locally, restore it from cache or external backup before running the canonical scripts.

## Current Checkpoint

- Fixed point: `ng_selfhost_clean.exe == output.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe`
- Size: `175104`
- SHA-256: `D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653`

## Active Documents

- [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md)
- [BUG_FREE_ACCEPTANCE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_FREE_ACCEPTANCE.md)
- [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md)
- [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md)
- [NG_TO_AXIOM_ROADMAP.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/NG_TO_AXIOM_ROADMAP.md)
- [BUG_ELIMINATION_STACK.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_ELIMINATION_STACK.md)
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

Trust root restore from local cache:

```powershell
powershell -ExecutionPolicy Bypass -File .\restore_trust_root.ps1
```

Trust root backup after any restored/promoted checkpoint:

```powershell
powershell -ExecutionPolicy Bypass -File .\backup_trust_root.ps1
```

Workspace scratch cleanup:

```powershell
powershell -ExecutionPolicy Bypass -File .\clean_workspace_scratch.ps1
```

Canonical repo artifact check:

```powershell
powershell -ExecutionPolicy Bypass -File .\verify_canonical_repo_artifacts.ps1
```

Canonical PE layout contract check:

```powershell
powershell -ExecutionPolicy Bypass -File .\verify_pe_layout_contract.ps1
```

## Current Focus

Block 1 hardening on the large `ng_native.ng -> ng_native.ng` path is closed for the current canonical surface.

The current official blocker is the fixed PE layout cliff:

- replace the fixed `.rdata` / `.idata` window with dynamically calculated RVAs
- keep the canonical scripts serial because direct/bootstrap work dirs are shared
- keep TCB and bug-free gates synchronized before any strong correctness claim

## Notes

- `README.md` is a quick operator guide, not the detailed audit log.
- `STATUS.md` is the active source of truth for current state and next steps.
- use `restore_trust_root.ps1` to restore the local trust root from `%LOCALAPPDATA%\Limbaj\trust-root-cache` when available.
- use `backup_trust_root.ps1` immediately after any restored or promoted checkpoint so cleanup does not strand the workspace again.
- use `verify_canonical_repo_artifacts.ps1` to confirm that the repo-kept trust root, corpus fixture and reference executables still match the canonical checkpoint.
- `BUG_FREE_ACCEPTANCE.md` defines when `bug-free` can be said onest and on what surface.
- `TCB.md` lists the current trust base and what still has to leave it.
- `SEMANTICS_CORE.md` fixes the semantic surface that future proofs and validators must preserve.
- `BUG_ELIMINATION_STACK.md` defines the full project-level stack on which bug elimination is expected, not just the currently green canonical slice.
- Historical documents may still mention the retired Rust host path; treat those references as archival only.
