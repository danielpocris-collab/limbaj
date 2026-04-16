# Status

Acest fisier ramane sursa operationala de adevar pentru worktree-ul curent.

## Current Direction

- worktree activ: `thirsty-proskuriakova`
- directie oficiala: `ng -> Axiom`
- traseu canonic: `ng_native.ng -> ng_native.ng`
- trust root canonic: `ng_selfhost_clean.exe`
- document canonic: [NG_TO_AXIOM_ROADMAP.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/NG_TO_AXIOM_ROADMAP.md)
- acceptance gates: [BUG_FREE_ACCEPTANCE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_FREE_ACCEPTANCE.md)
- trust base: [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md)
- semantic core: [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md)
- istoric buguri: [BUG_HISTORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_HISTORY.md)
- catalog erori: [ERROR_CATALOG.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ERROR_CATALOG.md)

## Current State

- worktree-ul este selfhost-only; host-ul Rust a fost scos din repo pe `2026-04-16`
- entrypoint-urile canonice raman:
  - [run_fast_check.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/run_fast_check.ps1)
  - [run_direct_stack.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/run_direct_stack.ps1)
  - [run_bootstrap_vertical.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/run_bootstrap_vertical.ps1)
- infrastructura comuna pentru rulare directa ramane in [direct_toolchain_common.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/direct_toolchain_common.ps1)
- compilatorul activ este [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng)
- fixed point-ul checkpoint-ului promovat ramas canonic este:
  - `ng_selfhost_clean.exe == output.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe`
  - size `161280`
  - SHA-256 `25DCECD94D17BBBF1BBCFFE36EC009F19BC7E5B7094C061D70E56A0FA3541012`
- candidatul selfhost curent validat, dar inca nepromovat, este:
  - `output.exe == direct_stack_compiler\output.exe == direct_fixed_point_selfhost_clean.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe == bootstrap_gen1\output.exe == bootstrap_gen2\output.exe`
  - size `171520`
  - SHA-256 `417188498A10B9219924E12B7804463A780EC5E73ACBBF1902B4898660A17CBD`

## Recent Progress

- ultimul checkpoint verde promovat inainte de hardening-ul curent a inchis Faza B Axiom pentru metadata de job queue:
  - `job_queue_param_starts` a fost eliminat complet din `Compiler`
  - dupa acel pas au trecut din nou:
    - `powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1`
    - `powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1`
    - `powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -MaxParallel 2`
- cleanup operational pe `2026-04-16`:
  - repo-ul nu mai contine host-ul Rust, `Cargo.toml`, `Cargo.lock`, `src/*.rs`, `target/` sau directoarele/artifactele `rust_*`
  - au fost eliminate si suitele / referintele `tooling_host_*` si `host_bridge_*` ca sa nu mai existe doua trasee concurente
  - documentele active au fost rescrise ca selfhost-only
- pas Axiom / Blocul 1 pe traseul mare:
  - hot path-ul mare nu mai foloseste apeluri CSV brute in punctele critice noi atinse in runda asta; au fost introduse accessori cursor pentru tabelele canonice `fn_*` si `builtin_offsets`
  - `find_func_opt(...)`, build-ul de job queue si apply-ul pentru builtin offsets trec acum prin accessori stabili, nu prin `csv_next_*` direct
  - urmele de debug injectate in validatorul final / bootstrap au fost scoase
- pas Axiom / Blocul 1 pe frontiere structurale:
  - lookup-urile pe `struct_names`, `fields`, `var_names` si `var_types` folosesc acum accessori canonici index-based, nu cursoare `csv_next_str_opt(...)` tinute manual in buclele de compilare
  - `find_field_offset_opt(...)`, `get_struct_size_opt(...)`, `find_var_opt(...)`, `add_var_typed(...)` si `find_var_type_opt(...)` citesc acum prin `load_*_opt(..., idx)` pe tabelele canonice deja existente
- cleanup operational pe stack-ul canonic direct:
  - `run_direct_stack.ps1` nu mai ruleaza o meta-suita nested intr-un singur proces `ng`
  - suitele leaf sunt rulate acum secvential, iar fixed-point-ul mare pentru `ng_native.ng` este verificat direct in PowerShell prin compare de bytes cu compilatorul testat
  - asta evita crash-ul runner-ului generic `#!ng-tool suite` pe cazul mare si pastreaza stack-ul canonic selfhost verde
- pas Axiom / Blocul 1 pe validatorul final:
  - `validate_final_layout_contract(...)`, `validate_final_patch_table(...)` si `validate_final_rip_patch_table(...)` verifica acum si contractele de aliniere / count pentru store-urile `u32`
  - `load_effective_function_offset_opt(...)` si `effective_function_offset_count(...)` trec prin `u32_store_count_opt(...)`, iar frontiera de import foloseste `import_slot_count()`
  - `start_apply_user_functions_serial(...)` nu mai citeste `fn_schedule_code_starts` inainte de guard-urile de lungime
- pas Axiom / Blocul 1 pe totalurile finale agregate:
  - `validate_final_layout_contract(...)` verifica acum si ca totalurile finale din `patch_offsets_store` / `patch_targets_store` si `rip_patch_offsets_store` / `rip_patch_targets_store` raman egale cu sumele aggregate din headerele `fn_artifact_*`
  - count-ul final pentru RIP patches este verificat explicit ca `rdata_patch_count + import_patch_count`, deci patch-urile de import raman contractate si la nivelul artefactului final, nu doar in apply
- pas Axiom / Blocul 1 pe contractul offset-urilor builtin in layout final:
  - `validate_final_layout_contract(...)` cere acum ca `builtin_offset_store` sa aiba exact `builtin_function_count() * 4` bytes inainte sa accepte layout-ul final
  - guard-ul a ramas compact, fara diagnostic literal nou, ca sa nu dubleze stringuri in `.rdata` pe buildul mare
- pas Axiom / Blocul 1 pe frontiera offset-urilor efective:
  - `validate_final_layout_contract(...)` ancora acum explicit primul offset din prefixul user (`fn_schedule_code_starts[0]`) si primul/ultimul offset din sufixul builtin (`builtin_offset_store[0]`, respectiv ultimul entry) in `final_effective_offsets`
  - daca `executor_effective_offsets` este prezent, aceleasi ancore sunt verificate si acolo, astfel compunerea efectiva ramane contractata pe frontiera dintre user code si builtins fara sa reintroduca scanari CSV
- pas Axiom / Blocul 1 pe frontiera schedule -> effective:
  - `validate_final_layout_contract(...)` cere acum si ca `fn_schedule_code_starts` sa aiba exact `compiled_function_schedule_count() * 4` bytes inainte sa citeasca prefixul user
  - validatorul ancora acum si ultimul offset user din `fn_schedule_code_starts` in `final_effective_offsets`, respectiv in `executor_effective_offsets` cand store-ul executor exista
- pas Axiom / Blocul 1 pe capatul schedule-ului final:
  - `validate_final_layout_contract(...)` leaga acum explicit `compiled_function_schedule_code_end(...)` de `entry_offset` si `compiled_function_schedule_rdata_end(...)` de `buf_len(comp.rdata)`
  - asta contracteaza ca ultimul entry user din schedule inchide exact buffer-ele finale de `code` si `rdata` inainte de validarea patch-urilor finale
- pas Axiom / Blocul 1 pe frontiera entrypoint-ului:
  - `validate_final_layout_contract(...)` cere acum ca `main_idx` sa ramana in prefixul user (`main_idx < compiled_function_schedule_count()`), nu in sufixul builtin din `effective_offsets`
  - asta leaga entrypoint-ul final de schedule-ul user fara sa adauge guard nou pe offset-uri sau sa impinga layout-ul PE peste cliff
- hardening operational pe builderul PE:
  - s-a confirmat un cliff real pe layout-ul fix al sectiunilor: daca `.rdata` trece de 4096 bytes, `.idata` ramane la RVA fix `0x102000` si loaderul Windows respinge binarul cu `ERROR_BAD_EXE_FORMAT`
  - guard-ul nou pentru totalurile finale a fost compactat fara string literals suplimentare, astfel candidatul curent ramane sub pragul critic si fixed-point-ul selfhost revine pe binar PE valid
- hardening operational pe bootstrap:
  - `direct_toolchain_common.ps1` recreeaza acum curat directoarele dedicate de build/bootstrap in interiorul workspace-ului inainte de compilari
  - asta elimina state rezidual pe `direct_*` / `bootstrap_*` fara sa atinga afara din workspace
- hardening operational pe orchestrarea nativa:
  - `direct_toolchain_common.ps1` nu mai redirectioneaza `stderr` in directorul de lucru al compilatorului; logurile sunt mutate in `_tool_logs\...`
  - asta elimina crash-ul Windows observat pe fixed-point-ul mare atunci cand `Start-Process` pornea compilatorul cu `stderr` redirectionat in `cwd`
- pas Axiom / Blocul 1 pe apply-ul de artefacte:
  - `start_apply_user_functions_serial(...)` verifica acum ca perechile `patch/rdata/import` din `fn_artifact_*_store` raman aliniate pe `u32` si au lungimi paralele
  - `step_apply_user_functions_serial(...)` opreste apply-ul daca o fereastra locala iese din store-ul pereche sau daca un patch local tinteste in afara `code_size`, `rdata_size`, `import_slot_count()` sau `executor_effective_offsets`
- pas Axiom / Blocul 1 pe offset-urile builtin:
  - `builtin_offsets` are acum backing canonic in `builtin_offset_store`, populat la registration prin `emit_u32(...)`
  - `load_builtin_offset_opt(...)` si apply-ul care construieste `effective_offsets` citesc acum builtins index-based din store-ul `u32`, nu din parse CSV pe traseul efectiv
- pas Axiom / Blocul 1 pe fallback-ul CSV pentru builtin offsets:
  - `load_builtin_offset_opt(...)` nu mai cade inapoi pe `builtin_offsets` CSV; daca `builtin_offset_store` nu este aliniat sau indexul iese din count, accessorul esueaza direct
  - helperul cursor `load_next_builtin_offset_opt(...)` a fost scos, iar traseul mare nu mai depinde de scanare CSV pentru builtin offsets
- pas Axiom / Blocul 1 pe scrierea builtin offsets:
  - `append_builtin_registration(...)` nu mai populeaza deloc `builtin_offsets` CSV; singurul backing ramas pentru builtin offsets este `builtin_offset_store`
  - asta scoate duplicarea reziduala din write path-ul mare fara sa schimbe inca layout-ul `Compiler`
- pas Axiom / Blocul 1 pe job queue-ul mare:
  - `build_compiled_function_job_queue(...)` nu mai parcurge `fn_names` / metadata de functii prin cursoare CSV directe pe traseul mare
  - numele, `param_count`, `param_start`, `body_start` si `local_count` sunt citite acum index-based prin accessori deja canonici
- pas Axiom / Blocul 1 pe contractul artefactelor locale:
  - `validate_compiled_function_contract(...)` verifica acum si perechile `patch`, `rdata_patch` si `import_patch` la nivel de CSV si store `u32`
  - count-ul din `*_targets_local`, `*_count_local` si store-urile locale `*_offsets_local_store` / `*_targets_local_store` trebuie sa ramana perfect paralel inainte ca artefactul sa intre in schedule/apply
  - `validate_compiled_function_patch_tables(...)` verifica acum explicit si count-ul logic din store-urile `u32` locale, nu doar lungimea lor bruta in bytes, inainte sa intre in buclele de validare pe intrari
- pas Axiom / Blocul 1 pe contractul agregat din schedule table:
  - `validate_compiled_function_schedule_table(...)` verifica acum si lungimile / alinierea pentru headerele `fn_artifact_*` si pentru store-urile agregate pereche `offset/target`
  - astfel, inconsistentele grosiere din tabelele de artefacte sunt respinse in faza de schedule, inainte sa intre in `start_apply_user_functions_serial(...)`
- pas Axiom / Blocul 1 pe contractul patch-urilor locale:
  - `append_csv_i64_to_u32_store(...)` promoveaza acum CSV-urile locale `patch/rdata/import` in store-uri `u32` intr-o singura trecere, fara pre-scan `csv_count(...)`
  - `compile_function_planned(...)` deriveaza acum `patch_count_local`, `rdata_patch_count_local` si `import_patch_count_local` din store-urile offset deja emise
  - `validate_compiled_function_contract(...)` verifica acum paritatea CSV atat pe listele locale de `offset`, cat si pe cele de `target`, deci mismatch-urile CSV/store raman contractate fara sa reintroduca count-ul CSV in write path-ul local
- candidat selfhost reverificat pe `2026-04-17`:
  - `.\ng_selfhost_clean.exe ng_native.ng` -> PASS, produce `output.exe` de `171520`
  - `powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1` -> PASS
  - `powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1 -Compiler ng_native.ng` -> PASS
  - `powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -Compiler ng_native.ng -MaxParallel 2` -> PASS
  - `GetBinaryType(output.exe)` / `GetBinaryType(ngc_gen1_bootstrap.exe)` / `GetBinaryType(ngc_gen2_bootstrap.exe)` -> PASS (`type = 6`, PE x64 valid)
- fundamentul de proiect pentru claim-uri tari de corectitudine este acum explicit in repo:
  - [BUG_FREE_ACCEPTANCE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_FREE_ACCEPTANCE.md) defineste acceptance gates pentru orice claim `bug-free`
  - [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md) enumera trust base-ul actual si gaurile care trebuie eliminate
  - [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md) fixeaza suprafata semantica minima care trebuie inghetata si pastrata de compiler
- checkpoint nou promovat pe `2026-04-16`:
  - `ng_selfhost_clean.exe == output.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe`
  - size `161280`
  - SHA-256 `25DCECD94D17BBBF1BBCFFE36EC009F19BC7E5B7094C061D70E56A0FA3541012`

## Known Bugs / Risks

- pasul oficial ramas este Blocul 1:
  - hardening pe traseul mare `ng_native.ng -> ng_native.ng`
  - in special `validate_final_layout_contract(...)`
  - validari de patch / rdata / import
  - reducerea ultimelor lookup-uri si scanari CSV directe
- checkpoint-ul promovat ramas canonic este inca cel de `161280`; pana la promovarea candidatului `171520`, `run_direct_stack.ps1` si `run_bootstrap_vertical.ps1` fara `-Compiler ng_native.ng` compara in continuare impotriva trust root-ului vechi
- compare-ul fixed-point mare pentru `ng_native.ng` nu mai ruleaza prin `#!ng-tool suite`; runner-ul generic ramane bun pentru suite leaf si compare-urile mici, dar cazul mare este orchestrat direct din PowerShell
- `run_direct_stack.ps1` si `run_bootstrap_vertical.ps1` reutilizeaza directoare / artefacte locale; verdictul canonic trebuie luat din rulare seriala, nu din doua scripturi pornite in paralel peste acelasi workspace
- builderul PE foloseste inca layout fix pentru sectiunile `.rdata` / `.idata`; orice schimbare care impinge `.rdata` peste 4096 bytes poate reintroduce `ERROR_BAD_EXE_FORMAT` pana cand RVA-urile de sectiune devin dinamice
- trust root-ul vechi ramane sensibil la schimbari aparent ne-semantice in zonele fierbinti; comentarii sau diagnostice noi in hardening-ul mare trebuie compactate si reverificate imediat cu `.\ng_selfhost_clean.exe ng_native.ng`
- validarea completa per-entry a compunerii `effective_offsets` ramane in afara bugetului curent al layout-ului PE fix; forma verde curenta contracteaza doar ancorele de frontiera pentru prefixul user si sufixul builtin
- worktree-ul nu este inca eligibil pentru vreun claim `bug-free`; acceptance gates din `BUG_FREE_ACCEPTANCE.md` raman rosii pana cand dispar riscurile de corectitudine de mai sus si checkpoint-ul nou este promovat
- documentele istorice pot pastra referinte la fostul host Rust; acestea sunt arhivistice, nu operationale

## Last Verified

Checkpoint-ul promovat ramas canonic:

- `ng_selfhost_clean.exe == output.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe`
- size `161280`
- SHA-256 `25DCECD94D17BBBF1BBCFFE36EC009F19BC7E5B7094C061D70E56A0FA3541012`

Candidatul curent verificat pe `2026-04-17`:

- `.\ng_selfhost_clean.exe ng_native.ng` -> PASS, produce `output.exe` de `171520`
- `powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1` -> PASS
- `powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1 -Compiler ng_native.ng` -> PASS
- `powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -Compiler ng_native.ng -MaxParallel 2` -> PASS
- `GetBinaryType(output.exe)` / `GetBinaryType(ngc_gen1_bootstrap.exe)` / `GetBinaryType(ngc_gen2_bootstrap.exe)` -> PASS (`type = 6`, PE x64 valid)
- `Get-FileHash output.exe, ngc_gen1_bootstrap.exe, ngc_gen2_bootstrap.exe -Algorithm SHA256` -> PASS, hash neschimbat `417188498A10B9219924E12B7804463A780EC5E73ACBBF1902B4898660A17CBD`

Checkpoint asociat:

- `output.exe == direct_stack_compiler\output.exe == direct_fixed_point_selfhost_clean.exe == ngc_gen1_bootstrap.exe == ngc_gen2_bootstrap.exe`
- `bootstrap_gen1\output.exe == bootstrap_gen2\output.exe == output.exe`
- size `171520`
- SHA-256 `417188498A10B9219924E12B7804463A780EC5E73ACBBF1902B4898660A17CBD`

## Next Steps

1. Reia Blocul 1 exact din checkpoint-ul verde de mai sus, numai pe `ng_native.ng`.
2. Pastreaza [BUG_FREE_ACCEPTANCE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_FREE_ACCEPTANCE.md), [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md) si [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md) sincronizate cu orice pas care atinge corectitudinea sau suprafata suportata.
3. Stabilizeaza contractul mare de layout final:
   - `validate_final_layout_contract(...)`
   - `validate_final_patch_table(...)`
   - `validate_final_rip_patch_table(...)`
4. Elimina ultimele lookup-uri / scanari CSV directe din traseul mare fara a reintroduce metadata duplicate.
5. Inchide explicit blocantele din acceptance gates:
   - validare completa per-entry pentru `effective_offsets`
   - builder PE fara cliff fix pe `.rdata` / `.idata`
   - promovarea oficiala a checkpoint-ului `171520` sau a succesorului lui
6. Dupa fiecare pas relevant, reruleaza numai suitele canonice selfhost:
   - `run_fast_check.ps1`
   - `run_direct_stack.ps1`
   - `run_bootstrap_vertical.ps1 -MaxParallel 2`

## Update Rule

Dupa orice schimbare relevanta:

- actualizeaza `Recent Progress`
- actualizeaza `Known Bugs / Risks`
- actualizeaza `Last Verified`
- pastreaza fisierul scurt si operational
