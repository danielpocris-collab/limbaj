# Multicore Compiler Plan

Scop: compilerul nativ trebuie sa foloseasca mai multe nuclee real, nu doar un singur thread mare.

## 1. Situatia curenta

Pipeline-ul critic din `ng_native.ng` este in mare parte serial:

1. `collect_functions(...)`
2. `compile_function(...)` pentru fiecare functie, in ordine
3. rebuild `fn_offsets`
4. entry setup
5. `patch loop`
6. `build_pe(...)`

Selfhost-ul mare este in acest moment corect, dar aproape complet single-core.

## 2. Ce NU trebuie facut

- nu paralelizam direct `patch loop` fara sa schimbam reprezentarea metadata
- nu paralelizam direct scrierea in `comp.code`
- nu introducem thread-uri in `ng_native.ng` inainte sa existe un model clar de job output independent
- nu facem locking fin pe `buf_push(...)`; asta ar serializa iarasi totul si ar complica enorm codul

## 3. Unitatea corecta de paralelizare

Unitatea naturala este functia.

Fiecare functie user trebuie compilata independent intr-un artefact local:

- `code_chunk`
- `patch_offsets_local`
- `patch_targets_local`
- `frame metadata`
- `string refs` daca apar local

Abia dupa aceea artefactele se concateneaza intr-o faza separata.

## 4. Arhitectura tinta

### Faza A: colectare seriala

Ramane seriala:

- token scan global
- `collect_functions(...)`
- metadata globala de structuri si semnaturi

Output:

- lista functii
- `fn_names`
- `fn_params`
- `fn_param_starts`
- `fn_bodies`
- `fn_local_counts`

### Faza B: compile jobs paralele

Pentru fiecare functie user:

- jobul primeste doar metadata imutabila
- produce `CompiledFunction`

Forma tinta:

```text
CompiledFunction {
    fn_idx
    code_buf
    patch_offsets_local
    patch_target_indices_local
}
```

Important:

- niciun job nu scrie direct in bufferul global `.text`
- niciun job nu patch-uieste direct relocarile globale

### Faza C: concatenare seriala

Se construieste `.text` final:

- se emit builtins
- se concateneaza `CompiledFunction.code_buf` in ordinea canonica
- se calculeaza `fn_offsets`
- patch-urile locale se re-bazeaza la offsetul global

### Faza D: patch serial sau chunked

Dupa ce `fn_offsets` exista:

- se aplica patch-urile globale
- daca patching-ul ramane scump, se poate chunk-ui pe bucati independente

### Faza E: PE build

Ramane probabil serial initial.

## 5. Pasii executabili in repo-ul actual

### Step 1

Introdu o structura explicita de rezultat per functie in design/docs, chiar daca implementarea ramane seriala.

### Step 2

Refactorizeaza `compile_function(...)` astfel incat sa poata lucra pe un `FunctionCompileContext` local, nu pe intreg `Compiler`.

### Step 3

Scoate dependenta directa pe `comp.code` din `compile_function(...)`.

Tinta:

- `compile_function_job(...) -> CompiledFunction`

### Step 4

Schimba patch metadata sa fie local-per-function inainte de concatenare.

### Step 5

Abia dupa ce pasii de mai sus sunt facuti, introduce thread pool in host-ul Rust (`rayon` sau `std::thread` + work queue).

## 6. Ce putem face imediat fara risc mare

### 6.1

Continuam sa eliminam scanari CSV repetitive:

- `patch loop`
- `fn_offsets`
- metadata lookups

### 6.2

Introducem timing explicit pe faze:

- collect
- compile loop
- patch loop
- build PE

### 6.3

Pregatim `compile_function(...)` pentru output local.

Asta este urmatorul pas real spre multicore.

## 7. Criteriul de succes

Nu consideram multicore reusit pana cand:

1. selfhost mare foloseste mai multe nuclee vizibil
2. `gen1 == gen2` ramane byte-identic
3. corpusul extins ramane verde
4. diagnosticele nu se degradeaza

## 8. Verdict

Da, proiectul trebuie sa iti foloseasca procesorul la adevarata lui putere.

Dar pentru asta trebuie:

- sa mutam compilerul de la buffer global serial
- la joburi per functie + concatenare canonica

Asta este schimbarea corecta de arhitectura.
