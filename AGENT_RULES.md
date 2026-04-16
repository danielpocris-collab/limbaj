# Agent Rules

Scop: reguli operative scurte pentru LLM-uri si alti agenti care lucreaza in acest worktree.

## 1. Directia oficiala

- `ng` nu se abandoneaza
- `Axiom` este arhitectura-tinta
- migrarea este incrementala
- nu se face rescriere completa daca exista cale de migrare

Documentul canonic de directie:
- [NG_TO_AXIOM_ROADMAP.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/NG_TO_AXIOM_ROADMAP.md)

## 2. Prioritati

Ordinea corecta este:
1. eliminarea claselor de buguri
2. invariants mai tari in compiler
3. runtime mai predictibil
4. toolchain mai verificabil
5. abia dupa aceea features noi

## 3. Ce sa faci

- prefera schimbari mici si cumulative
- prefera hardening fata de feature creep
- prefera frontiere explicite fata de conventii tacite
- prefera diagnostice explicite fata de fallback-uri ascunse
- prefera modele care reduc TCB-ul
- verifica dupa modificari cu smoke tests sau corpus minim

## 4. Ce sa nu faci

- nu propune rewrite complet fara motiv exceptional
- nu adauga magie implicita care ascunde ownership, efecte sau costuri
- nu introduce sentineli noi de tip `-1`, `0`, `\"\"` pentru semnificatii ascunse
- nu amesteca unsafe/interop liber in nucleul sigur
- nu creste complexitatea teoretica daca nu reduce buguri reale
- nu adauga sintaxa noua doar pentru ca suna puternic

## 5. Regula pentru feature-uri noi

Orice feature nou trebuie evaluat cu intrebarile:
- ce clasa de bug elimina
- ce invariant nou introduce
- ce cost semantic adauga
- ce cost runtime adauga
- ce impact are asupra TCB

Daca nu reduce clar riscul sistemic, nu intra in nucleu.

## 6. Regula pentru compiler

Orice schimbare de compiler trebuie gandita si in termenii:
- selfhost
- fixed-point
- reproducibilitate
- diagnostics
- failure modes explicite

## 7. Regula pentru interop

FFI, runtime extern, OS boundary, fisiere, retea si timp trebuie tratate ca frontiere explicite.

Nu se presupune ca lumea externa este sigura.
Se contracteaza, se valideaza sau se izoleaza.

## 8. Regula finala

Scopul nu este sa facem debugging-ul mai usor.

Scopul este sa facem familii intregi de debugging inutile.

## 9. Regula de status

Dupa orice schimbare relevanta, agentul trebuie sa actualizeze:
- [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md)

Minim:
- ce a facut
- unde a ramas
- ce buguri sau riscuri sunt cunoscute
- ce a verificat concret

Nu se lasa progresul doar in chat daca poate fi pierdut.

La nevoie, actualizeaza si:
- [BUG_HISTORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_HISTORY.md)
- [ERROR_CATALOG.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ERROR_CATALOG.md)
