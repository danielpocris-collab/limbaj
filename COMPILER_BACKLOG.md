# Compiler Backlog

Scop: lista scurta de schimbari executabile pentru compilerul actual din worktree-ul `ng_native`.

Referinte:
- [NG_STRICT_PLAN.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/NG_STRICT_PLAN.md)
- [SPEC_ERRORS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SPEC_ERRORS.md)
- [SPEC_MEMORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SPEC_MEMORY.md)
- [BUG_HISTORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_HISTORY.md)
- [ERROR_CATALOG.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ERROR_CATALOG.md)

## A. Curatare compiler intern

1. Inlocuieste lookup-urile pe sentinel cu variante stricte:
- `find_func -> find_func_opt`
- `find_field_offset -> find_field_offset_opt`
- alte helper functions similare

2. Separa clar fazele:
- tokenizare
- colectare declaratii
- compilare builtin-uri
- compilare user functions
- build PE

3. Adauga validari dupa fiecare faza:
- functia `main` exista
- aritatea este valida
- offset-urile sunt parseabile
- patch-urile targeteaza pozitii valide

## B. Reprezentari interne

1. Reduce dependenta de `csv_*` pentru metadate de compiler.
2. Introdu structuri interne reale pentru:
- function table
- field table
- patch table
- string table

Prima migrare:
- nu scoatem tot `csv` dintr-o data
- incepem cu `fn_names`, `fn_params`, `fn_bodies`, `fn_offsets`

## C. Builtins si memorie

1. Standardizeaza contractul pentru:
- `str_byte_at`
- `str_slice`
- `str_concat`
- `buf_push`
- `buf_get`
- `buf_set`
- `buf_append`

2. Pentru fiecare builtin:
- defineste input valid
- defineste output
- defineste failure mode
- adauga test de stress

## D. Diagnostice si tracing

1. Pastreaza `DBG:` tracing dar trece la etichete de faza stabile.
2. Adauga un flag sau o constanta de build pentru trace on/off.
3. Orice failure path important trebuie sa poata spune:
- in ce faza a cazut
- ce functie compila
- ce helper builtin rula

## E. Selfhost si fixed-point

1. Script stabil pentru:
- host compile `ng_native.ng -> gen1`
- gen1 compile `ng_native.ng -> gen2`
- comparatie `gen1` vs `gen2`

2. Testeaza separat:
- compilatorul host
- binarul gen1
- binarul gen2

3. Daca gen1 cade:
- ruleaza corpus minim inainte de selfhost complet

## F. Corpus minim obligatoriu

1. `exit42.ng`
2. `fileio_native.ng`
3. `compiler_ops.ng`
4. `many_funcs.ng`
5. `huge_main.ng`
6. `csv_stress.ng`
7. `ng_native.ng`

Ordinea este importanta: de la simplu la selfhost complet.

## G. Urmatoarele 5 task-uri concrete

1. Audit intern pentru toate functiile care intorc sentinel.
2. Stabilizare trace phases in `main(source :: str)`.
3. Test harness serial pentru corpusul minim.
4. Document ABI pentru `str` si `buf`.
5. Prima inlocuire a unui bloc `csv` cu structura interna reala.
