# Error Catalog

Scop: catalog central pentru erori, failure modes si diagnostice importante ale compilerului, interpreterului si verticalei de bootstrap.

Acest fisier nu este doar despre buguri fixate. Este despre:
- ce erori exista
- unde apar
- cum trebuie interpretate
- daca sunt asteptate, temporare sau semn de coruptie

Documente conexe:
- [SPEC_ERRORS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SPEC_ERRORS.md)
- [SPEC_MEMORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SPEC_MEMORY.md)
- [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md)
- [BUG_HISTORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_HISTORY.md)

## Format

Pentru fiecare eroare importanta, adauga:
- `Code`
- `Layer`
- `Trigger`
- `Meaning`
- `Typical Cause`
- `Action`

## Catalog

### ERR-COMP-001

- Layer: compiler `ng`
- Trigger: `Error: missing function name entry`
- Meaning: metadata pentru function table este incoerenta sau indexul este in afara intervalului
- Typical Cause: tabele CSV nealiniate sau index gresit in load path
- Action: verifica `fn_names`, `func_count`, `load_function_plan(...)`

### ERR-COMP-002

- Layer: compiler `ng`
- Trigger: `Error: missing compiled user offset entry`
- Meaning: rebuild-ul offseturilor per functie nu are intrare pentru o functie user deja compilata
- Typical Cause: mismatch intre `fn_compiled_offsets` si `user_func_count`
- Action: verifica `merge_compiled_function(...)` si `rebuild_compiled_function_offsets(...)`

### ERR-COMP-003

- Layer: compiler `ng`
- Trigger: `Error: patch offset count mismatch` sau `patch target count mismatch`
- Meaning: patch table globala este corupta sau incompleta
- Typical Cause: mismatch intre `patch_count`, `patch_offsets`, `patch_targets`
- Action: verifica `validate_patch_table(...)` si generarea patch-urilor

### ERR-COMP-004

- Layer: compiler `ng`
- Trigger: `Error: invalid local patch entry`
- Meaning: metadata locala a unei functii compilate este invalida
- Typical Cause: CSV local corupt sau parse esuat in `load_local_patch_entry_opt(...)`
- Action: verifica `CompiledFunction` local si `validate_compiled_function_patch_tables(...)`

### ERR-COMP-005

- Layer: compiler `ng`
- Trigger: `Error: invalid local rdata patch entry`
- Meaning: string-ref patch local invalid
- Typical Cause: mismatch intre `rdata_patch_offsets_local` si `rdata_patch_targets_local`
- Action: verifica emiterea de string literals si merge-ul pe `rdata`

### ERR-COMP-006

- Layer: compiler `ng`
- Trigger: `Error: invalid local import patch entry`
- Meaning: patch local pentru IAT invalid
- Typical Cause: mismatch intre `import_patch_offsets_local` si `import_patch_targets_local`
- Action: verifica emiterea import-call-urilor RIP-relative

### ERR-COMP-007

- Layer: compiler `ng`
- Trigger: `Error: unresolved call target ...` urmat de `FATAL: unresolved call target ...`
- Meaning: a fost intalnit un apel catre functie inexistenta sau nerezolvata
- Typical Cause: typo in nume de functie, functie lipsa, metadata de functie incompleta
- Action: trateaza ca failure de compilare; nu trebuie sa rezulte `output.exe`; verifica `find_func_opt(...)`, `compile_function_planned(...)` si `fast_check_missing_call.ng`

### ERR-COMP-008

- Layer: compiler `ng`
- Trigger: `Error: missing variable ...` urmat de `FATAL: missing variable ...`
- Meaning: expresia sau statement-ul foloseste o variabila care nu exista in scope-ul compilat
- Typical Cause: typo in nume, lipsa declararii, metadata locala de variabile incoerenta
- Action: trateaza ca failure de compilare; nu trebuie sa rezulte `output.exe`; verifica `find_var_opt(...)`, `compiler_fail(...)` si `fast_check_missing_var.ng`

### ERR-TYPE-001

- Layer: compiler / selfhost
- Trigger: `missing field i64.start`
- Meaning: un obiect tratat ca struct a fost tipat sau inregistrat gresit ca `i64`
- Typical Cause: metadata de variabile sau parametri corupta
- Action: verifica `register_params(...)`, `add_var_typed(...)`, lookup-ul de tip

### ERR-RUNTIME-001

- Layer: interpreter host
- Trigger: bounds invalid pe `str_byte_at`, `str_slice`, `buf_get`, `buf_set`
- Meaning: acces invalid in `str` sau `buf`
- Typical Cause: indexare OOB
- Action: trateaza ca bug real de logic sau contract; nu se mai ascunde prin sentinele

### ERR-BOOT-001

- Layer: verticala bootstrap
- Trigger: `%1 is not a valid Win32 application. (os error 193)`
- Meaning: artefactul compilat nu este executabil valid pentru target
- Typical Cause: miscompile sever, PE corupt sau selfhost rupt
- Action: opreste progresul, compara cu ultimul punct verde si cauta ultima schimbare care afecteaza codegen / build final

### ERR-BOOT-002

- Layer: toolchain / verificare
- Trigger: `failed printing to stdout: The pipe is being closed. (os error 232)`
- Meaning: procesul a murit in timp ce stdout-ul era inchis de caller sau de timeout-ul tool-ului
- Typical Cause: timeout de orchestrare sau pipe inchis din exterior, nu neaparat bug de compiler
- Action: reruleaza cu timeout mai mare inainte sa marchezi regresie reala

## Update Rule

Dupa orice eroare noua sau diagnostic relevant:
- adauga intrare noua aici daca eroarea merita retinuta
- leaga eroarea de un bug din [BUG_HISTORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_HISTORY.md) daca exista
- pastreaza `STATUS.md` scurt; detaliul traiectoriei sta aici
