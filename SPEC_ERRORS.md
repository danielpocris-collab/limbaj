# Spec Errors

Scop: definim regulile executabile pentru erori si absenta in `ng`-ul actual.

Documentele de design exista deja in:
- [ng/ng-spec.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng/ng-spec.md)
- [BATCH1_DESIGN.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BATCH1_DESIGN.md)

Aici fixam doar ce trebuie implementat concret in compiler.

## Reguli

1. Nu exista `null`.
2. Absenta se modeleaza doar cu `Option[T]`.
3. Eroarea recuperabila se modeleaza doar cu `Result[T, E]`.
4. Nu folosim sentinele ascunse pentru API-uri noi.
5. `match` pe `Option` si `Result` trebuie sa fie exhaustiv.

## Reguli de migrare

Acceptam temporar API-uri vechi care intorc:
- `-1`
- `0`
- `""`

Dar doar daca sunt marcate intern ca legacy si au wrapper strict nou.

Exemple:
- `find_func(...) -> i64` ramane intern temporar
- `find_func_opt(...) -> Option[i64]` devine API-ul nou

## Schimbari concrete in compiler

1. Parser:
- pastreaza suportul de sintaxa pentru `Option[T]` si `Result[T, E]`
- adauga erori clare pentru constructori folositi cu tipuri gresite

2. Type checker:
- verifica exhaustivitatea pe `Option/Result`
- interzice amestecul de constructori intre tipuri diferite
- interzice folosirea valorilor de tip `Option[T]` sau `Result[T, E]` ca si cum ar fi `T`

3. Runtime / codegen:
- stabileste layout ABI clar pentru `Option` si `Result`
- interzice “simplified int64_t” ca reprezentare finala pentru backend-urile serioase

4. Biblioteci:
- introdu wrappers stricte pentru functiile legacy cu sentinele
- noile API-uri standard expun doar `Option` sau `Result`

## Backlog imediat

1. Audit pe functii interne care intorc sentinele in `ng_native.ng`.
2. Lista de wrappere stricte peste API-urile legacy.
3. Teste de compilare care trebuie sa pice:
- match neexhaustiv pe `Option`
- match neexhaustiv pe `Result`
- atribuirea `Option[i64]` la `i64`
- atribuirea `Result[str, E]` la `str`

## Prima tinta mica

Refactor intern:
- `find_func`
- `find_field_offset`
- lookup-urile similare

Tinta nu este eleganta, ci eliminarea conventiilor fragile `-1 means missing`.
