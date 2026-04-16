# Bug History

Scop: istoric explicit al bugurilor reale, regresiilor si incidentelor observate in acest worktree.

Nota:
- referintele la `cargo`, `target\\release\\limbaj.exe` sau la host-ul Rust descriu istoria dinaintea eliminarii host-ului pe `2026-04-16`
- nu reprezinta traseul operational curent

Acest fisier nu este pentru viziune sau backlog. Este pentru:
- buguri confirmate
- regresii introduse si retrase
- cauza radacina cunoscuta
- fixul aplicat
- verificarea concreta dupa fix

Documente conexe:
- [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md)
- [SENTINEL_AUDIT.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SENTINEL_AUDIT.md)
- [ANTI_BUG_MATRIX.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ANTI_BUG_MATRIX.md)
- [BOOTSTRAP_REPRO_AUDIT.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BOOTSTRAP_REPRO_AUDIT.md)

## Format

Pentru fiecare incident nou, adauga:
- `ID`
- `Status`: `fixed`, `open`, `reverted`, `mitigated`
- `Zona`
- `Simptom`
- `Cauza`
- `Fix / Decizie`
- `Verificare`

## Incidents

### BUG-001

- Status: `fixed`
- Zona: `bind` / `vary` codegen
- Simptom: dupa declarare, valoarea era scrisa la `next_var_offset(...)` in loc de offset-ul real al variabilei; producea crash-uri si valori corupte la runtime
- Cauza: offset-ul de write era recalculat, nu folosea slotul real al variabilei deja inregistrate
- Fix / Decizie: write-ul dupa declarare a fost legat de offset-ul real al variabilei
- Verificare: corpusul minim si apoi corpusul extins au trecut

### BUG-002

- Status: `fixed`
- Zona: `register_params(...)`
- Simptom: selfhost-ul mare lovea erori de forma `missing field i64.start`
- Cauza: parametrii erau inregistrati cu `add_var(...)`, deci primeau implicit tipul `i64`
- Fix / Decizie: parametrii sunt inregistrati cu `add_var_typed(...)`
- Verificare: selfhost-ul mare a trecut de blocajele pe `tok.start`, `tok.end`, `struct_fields`

### BUG-003

- Status: `fixed`
- Zona: interpreter host, operatori logici
- Simptom: `and` / `or` evaluau eager, nu short-circuit
- Cauza: evaluatorul nu oprea evaluarea pe primul operand
- Fix / Decizie: short-circuit explicit in interpreter
- Verificare: `cargo test` a ramas verde; bounds checks stricte pe `str` / `buf` nu au mai expus citiri false

### BUG-004

- Status: `fixed`
- Zona: memory builtins in interpreter host
- Simptom: `str_byte_at`, `str_slice`, `buf_get`, `buf_set` ascundeau OOB prin sentinele
- Cauza: contract vechi bazat pe valori implicite in loc de eroare explicita
- Fix / Decizie: failure explicit pe bounds invalid
- Verificare: teste unitare dedicate pentru toate cele 4 path-uri OOB

### BUG-005

- Status: `fixed`
- Zona: `merge_compiled_function(...)`
- Simptom: selfhost-ul raporta `missing field PatchEntry.patch_offset` / `target_offset` / `target_slot`, apoi `gen1` producea corpus rupt
- Cauza: refolosirea aceluiasi nume local `patch_entry` pe structuri diferite a confundat tipurile in selfhost
- Fix / Decizie: nume locale distincte pentru `PatchEntry`, `RdataPatchEntry`, `ImportPatchEntry`
- Verificare: `cargo test` PASS, `bootstrap-vertical` PASS, corpus pe `gen1` si `gen2` PASS

### BUG-006

- Status: `reverted`
- Zona: nested struct migration pentru patch metadata
- Simptom: verticala s-a rupt imediat pe corpusul `gen1`
- Cauza: limita actuala a selfhost-ului pe assignment / transport de structuri imbricate
- Fix / Decizie: migrarea a fost retrasa; directia corecta ramane accessors si hardening incremental, nu nesting direct
- Verificare: dupa revert, verticala a revenit pe verde

### BUG-007

- Status: `reverted`
- Zona: validari explicite pentru `struct table` / `var table`
- Simptom: selfhost-ul producea un `gen1` nevalid ca executabil Win32 (`os error 193`)
- Cauza: forma concreta a validatorilor a activat o limita a selfhost-ului; cauza exacta fina ramane nediagnosticata
- Fix / Decizie: validatorii au fost retrasi complet; vor fi reintrodusi mai incremental
- Verificare: dupa revert, `bootstrap-vertical` a revenit pe verde

### BUG-008

- Status: `reverted`
- Zona: centralizare metadata function table
- Simptom: dupa extragerea append-urilor de function metadata intr-un helper generic, `host -> gen1` producea un compilator care rupea corpusul complet pe `gen1`; cazurile cadeau cu crash-uri si iesiri de forma `-1073741819`
- Cauza: helperul generic `append_function_entry(...)` a expus o limita actuala a selfhost-ului / codegen-ului; semantic refactorul parea echivalent, dar binarul generat nu mai era corect
- Fix / Decizie: helperul si toate call site-urile au fost retrase; append-urile manuale au fost restaurate in `collect_functions(...)` si inregistrarea builtin-urilor
- Verificare: dupa revert, `cargo test` PASS si `cargo run --release -- bootstrap-vertical 2` PASS; fixed-point bootstrap curent `108032` bytes, SHA-256 `47DF95803A49ED3CCFB2DDE546E74CF9B4A9163EEEB0E3C2CECE9274AB2BA32B`

### BUG-009

- Status: `reverted`
- Zona: validari late de metadata in Blocul 1
- Simptom: o runda mai lata de validari pentru function/struct/var table producea din nou un `gen1` nevalid ca executabil Win32; corpusul pe `gen1` cadea complet cu `%1 is not a valid Win32 application. (os error 193)`
- Cauza: reintroducerea validatorilor ca functii reale in selfhost a activat din nou limita deja vazuta pe traseul `os error 193`; cauza fina ramane nediagnosticata
- Fix / Decizie: validatorii lati au fost retrasi complet; a ramas doar partea sigura din runda aceasta, adica accessorul `FunctionEntry` si call site-urile lui pentru function metadata
- Verificare: dupa revertul validatorilor, `cargo test` PASS si `cargo run --release -- bootstrap-vertical 2` PASS; fixed-point bootstrap curent `108544` bytes, SHA-256 `C41B8678B7905E1BD9BE80AF9B6FEFDE2D6EA8878F12F76C267FE4C968A73947`

### BUG-010

- Status: `fixed`
- Zona: runtime helper calls in selfhost
- Simptom: dupa introducerea helperului `trace(...)`, host selfhost-ul se oprea imediat dupa primul log (`DBG: main entered`) si nu mai producea `output.exe`
- Cauza: in starea actuala a selfhost-ului, apelurile la functii care intorc valoare nu sunt sigure ca statement-uri simple in `main`; rezultatul neconsumat al lui `trace(...)` a rupt traseul operational
- Fix / Decizie: helperii `trace(...)` au fost pastrati, dar apelurile lor din `main` consuma explicit rezultatul prin `bind`
- Verificare: dupa fix, `cargo test` PASS si `cargo run --release -- bootstrap-vertical 2` PASS; fixed-point bootstrap curent `109056` bytes, SHA-256 `0B56664B7131D0A36C2C81B920C3728B6BAC24C081D7A1ACE2EC099B34B4D562`

### BUG-011

- Status: `fixed`
- Zona: compile path failure handling
- Simptom: `Error: unresolved call target ...` era doar logat; compilarea continua si producea totusi `output.exe`
- Cauza: compile path-ul local nu avea un semnal minim de esec care sa opreasca merge-ul functiei si buildul final
- Fix / Decizie: `Compiler` local si `CompiledFunction` poarta acum un semnal minim de esec; apelul nerezolvat marcheaza functia ca esuata, `compile_function(...)` nu o mai merge-uieste, iar buildul final se opreste prin validarea offseturilor
- Verificare: `run_fast_check.ps1` include acum `fast_check_missing_call` si trece; `cargo test` PASS; `bootstrap-vertical` PASS; fixed-point bootstrap curent `110080` bytes, SHA-256 `9283633103BE41381D07D4F0EF00FC203BD704272D32AC2972C0101CF5514C8A`

### BUG-012

- Status: `fixed`
- Zona: compile path failure handling for missing locals
- Simptom: `Error: missing variable ...` era logat in mai multe locuri, dar nu oprea coerent compilarea si putea lasa buildul sa continue
- Cauza: lipsa unui mecanism unificat de escaladare din erorile locale de metadata/lookup spre esecul functiei compilate
- Fix / Decizie: a fost introdus `compiler_fail(...)`, iar failure mode-ul pentru `missing variable`, `missing variable type` si `missing field` in punctele critice foloseste acum acelasi semnal minim de esec ca si apelurile nerezolvate
- Verificare: `run_fast_check.ps1` include acum `fast_check_missing_var` si trece; `cargo test` PASS; `bootstrap-vertical` PASS; fixed-point bootstrap curent `110080` bytes, SHA-256 `ADFC23E64ADB1791745B243BC9D129501621346785AB32F78A451B3B3B117AB5`

### BUG-013

- Status: `fixed`
- Zona: compile path diagnostic hygiene
- Simptom: dupa primul failure local, compile path-ul emitea doua linii `FATAL: ...` pentru acelasi diagnostic
- Cauza: atat `compile_function(...)`, cat si `main(...)` tratau acelasi esec ca punct de iesire final
- Fix / Decizie: `compile_function(...)` doar marcheaza `comp.failed` + `failure_message`; singurul punct canonic de `fatal(...)` ramane dupa `compile_user_functions_serial(...)`, iar compile loop-ul iese imediat la primul esec
- Verificare: `cargo run --release -- selfhost ng_native.ng tests/programs/fast_check_missing_var.ng` emite acum o singura linie `FATAL: missing variable x`; `run_fast_check.ps1` verifica acum explicit unicitatea `FATAL` pe cazurile negative si PASS; `cargo test` PASS

### BUG-014

- Status: `reverted`
- Zona: serial batch collection for compiled job results
- Simptom: tentativa de a serializa batch-ul de `CompiledFunction` si apoi de a-l aplica intr-o a doua faza rupea imediat selfhost-ul pe cazuri simple, cu eroarea `str_from_i64 expects i64`
- Cauza: `code_buf` / `rdata_buf` si handle-urile asociate nu sunt transportabile sigur prin serializare simpla `str_from_i64(...)` in forma actuala a runtime-ului selfhost
- Fix / Decizie: batch-ul serial explicit a fost retras; au ramas doar frontierele sigure deja introduse (`prepare/apply`, `merge plan`, logging pe rezultat colectat)
- Verificare: dupa revert, `cargo run --release -- fast-check` PASS, `cargo test` PASS, `cargo run --release -- selfhost ng_native.ng tests/programs/exit42.ng` PASS

### BUG-015

- Status: `reverted`
- Zona: binary artifact table for compiled job payloads
- Simptom: tentativa de a pastra `CompiledFunction` intre `plan` si `apply` prin handle-uri binare + metadata separata a rupt selfhost-ul chiar in faza `compile plan`; cazurile simple cadeau cu eroare de tip `Type mismatch in binary operation: [..bytes..] Less 0`
- Cauza: in forma actuala a selfhost-ului, tratarea handle-urilor `buf` ca payload numeric binar persistat in `Compiler` a expus o limita reala de tipare/operatii; traseul a ajuns sa compare un buffer cu `0`, deci modelul de transport nu este sigur inca
- Fix / Decizie: tabela binara de artefacte a fost retrasa complet; a ramas pe verde varianta cu doua faze `plan all -> apply all`, dar cu recompilare in faza `apply`
- Verificare: dupa revert, `cargo run --release -- fast-check` PASS, `cargo test` PASS, `cargo run --release -- selfhost ng_native.ng tests/programs/exit42.ng` PASS

## Update Rule

Dupa orice bug real sau regresie:
- adauga intrare noua aici
- actualizeaza `STATUS.md` cu starea curenta
- daca bugul tine de sentinele sau invariants, actualizeaza si auditul relevant
