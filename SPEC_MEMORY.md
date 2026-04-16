# Spec Memory

Scop: definim modelul minim de memorie pentru `ng` fara sa rescriem limbajul.

Referinte de design:
- [ng/ng-spec.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng/ng-spec.md)
- [README.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/README.md)

## Reguli

1. Tipurile mici scalare sunt copy.
2. `str` si `buf` au semantica explicita, nu conventii implicite.
3. Mutatia necesita acces exclusiv.
4. Indexarea este checked by default.
5. Alocarile care pot esua trebuie sa aiba model explicit de eroare.

## Model minim pentru etapa curenta

### `str`
- este tratat ca view imutabil peste bytes
- nu se modifica in loc
- slicing-ul produce view nou validat
- orice indexare in `str_byte_at` este bounds-checked

### `buf`
- este owner mutabil
- `buf_push`, `buf_set`, `buf_append` muta sau modifica explicit bufferul
- operatiile care cresc capacitatea trebuie sa fie auditate pentru failure path

## Reguli de ownership pragmatice

In compilerul actual nu introducem imediat un borrow checker complet.
Introducem mai intai reguli si diagnostice simple:

1. Parametrii pentru tipuri mutabile trebuie marcati clar in IR sau metadata.
2. Functiile care muta semantic ownership nu trebuie sa lase in urma aliasuri utilizabile.
3. API-urile standard trebuie separate in:
- read-only view
- mutating owner API

## Schimbari concrete in compiler

1. Audit pe builtins:
- `str_len`
- `str_byte_at`
- `str_slice`
- `str_concat`
- `buf_new`
- `buf_push`
- `buf_get`
- `buf_set`
- `buf_append`

2. Pentru fiecare builtin:
- definim preconditii
- definim comportament pe input invalid
- definim daca poate aloca
- definim daca poate esua

3. Adaugam mod strict:
- bounds checks obligatorii
- cod de iesire sau trap standard la memorie invalida
- tracing simplu pentru failure path

## Backlog imediat

1. Document ABI pentru `str` si `buf` in backend-ul nativ.
2. Teste negative pentru index out of bounds.
3. Teste pentru input gol, string mare, buffer mare, concat repetat.
4. Refactor al helperilor interni care encodeaza structuri in `csv`.

## Prima tinta mica

Standardizam semantic urmatoarele functii:
- `str_byte_at`
- `str_slice`
- `buf_push`
- `buf_get`
- `buf_set`

Daca acestea sunt rigide, scade mult riscul de memorie si miscompile in restul compilerului.
