# Anti Bug Matrix

Scop: legam fiecare clasa mare de bug de mecanisme concrete in:
- limbaj
- compiler
- toolchain

Referinte:
- [NG_STRICT_PLAN.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/NG_STRICT_PLAN.md)
- [SPEC_ERRORS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SPEC_ERRORS.md)
- [SPEC_MEMORY.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SPEC_MEMORY.md)
- [COMPILER_BACKLOG.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/COMPILER_BACKLOG.md)

## 1. Memorie

Limbaj:
- ownership clar
- borrow read-only vs borrow mutabil exclusiv
- fara `null`
- indexing checked

Compiler:
- verificari de aliasing si lifetime unde modelul permite
- interzicere acces invalid prin reguli semantice

Toolchain:
- sanitizers
- corpus de memorie
- stress tests pe `str` si `buf`

Mai poate ramane:
- bug in `unsafe`
- bug in runtime/allocator/backend

## 2. Logica

Limbaj:
- tipuri stricte
- exhaustiveness
- imutabilitate implicita

Compiler:
- type checking dur
- validare control flow

Toolchain:
- property tests
- corpus real

Mai poate ramane:
- specificatie gresita
- algoritm gresit dar bine tipat

## 3. Design

Limbaj:
- modele de date tari
- enums si match
- efecte explicite

Compiler:
- verificari de invariants intre faze

Toolchain:
- teste de arhitectura
- teste de compatibilitate

Mai poate ramane:
- model conceptual gresit

## 4. Integrare

Limbaj:
- contracte explicite
- capabilitati
- ABI clar

Compiler:
- validare de interfete
- validare schema si aritate

Toolchain:
- integration tests
- reproducible builds

Mai poate ramane:
- dependinte externe schimbate

## 5. Concurenta

Limbaj:
- shared mutable state restrans
- ownership clar intre fire/actori

Compiler:
- verificari pe mutabilitate si sharing

Toolchain:
- stress tests pe scheduling

Mai poate ramane:
- buguri low-level de sincronizare

## 6. Timp si ordine

Limbaj:
- timp si clock tratate ca efect explicit

Compiler:
- interzicere efecte impure in contexte pure

Toolchain:
- timeout tests
- retry tests
- skew tests

Mai poate ramane:
- probleme distribuite reale

## 7. Date reale neasteptate

Limbaj:
- parsing in tipuri validate
- fara stringly typed internals

Compiler:
- verificare de shape si schema

Toolchain:
- fuzzing
- corpus mare
- reducer automat pentru crash-uri

Mai poate ramane:
- formate noi sau coruptii exotice

## 8. Numeric

Limbaj:
- conversii explicite
- politica clara de overflow
- unitati distincte unde conteaza

Compiler:
- verificari de range si const

Toolchain:
- edge-case tests
- property tests numerice

Mai poate ramane:
- specificatie matematica gresita

## 9. Semantica limbajului

Limbaj:
- reguli simple
- ordine de evaluare clara
- fara conversii surpriza

Compiler:
- diagnostice dure pentru ambiguitati

Toolchain:
- golden tests
- compat tests

Mai poate ramane:
- decizie de design proasta

## 10. Compiler

Limbaj:
- nucleu mic si regulat

Compiler:
- IR tipat
- validari intre pase
- backend checks

Toolchain:
- selfhost
- fixed-point
- differential testing

Mai poate ramane:
- miscompile subtil in backend

## 11. Runtime

Limbaj:
- runtime minim

Compiler:
- ABI si calling convention validate

Toolchain:
- sanitizers
- fault injection

Mai poate ramane:
- bug allocator
- bug syscall wrapper

## 12. Performanta

Limbaj:
- cost model simplu
- alocari vizibile
- copii vizibile

Compiler:
- metrics pe faze
- warnings pentru pattern-uri costisitoare

Toolchain:
- perf regression suite

Mai poate ramane:
- workload-uri neasteptate

## 13. Securitate

Limbaj:
- capabilities
- fara ambient authority
- API-uri sigure by default

Compiler:
- verificari de efecte/capabilitati

Toolchain:
- fuzzing adversarial
- hardening
- audit

Mai poate ramane:
- side channels
- integrare externa slaba

## 14. Observabilitate

Limbaj:
- erori structurate
- tracing activabil clar

Compiler:
- diagnostice precise cu faza si context

Toolchain:
- logs
- traces
- snapshot reproductibil

Mai poate ramane:
- semnal insuficient daca runtime-ul nu este instrumentat

## 15. Operational

Limbaj:
- config tipat
- feature flags tipate

Compiler:
- validare schema config

Toolchain:
- deployment checks
- migration tests

Mai poate ramane:
- erori de infrastructura

## 16. Testare

Limbaj:
- cod testabil prin puritate si efecte explicite

Compiler:
- warnings pentru zone neverificate

Toolchain:
- unit
- property
- corpus
- fuzz
- selfhost

Mai poate ramane:
- oracle gresit
- acoperire incompleta

## 17. Toolchain

Limbaj:
- determinism ca obiectiv

Compiler:
- ordine stabila
- fara dependenta de hash nondeterminist

Toolchain:
- reproducible builds
- byte compare

Mai poate ramane:
- tooluri externe instabile

## 18. Umane

Limbaj:
- putine footguns
- ergonomie sigura

Compiler:
- diagnostice care explica si corecteaza

Toolchain:
- formatter
- linter
- codemods

Mai poate ramane:
- decizii gresite de produs

## 19. AI si generare

Limbaj:
- API-uri greu de folosit gresit

Compiler:
- verificari stricte
- refuz pentru cod periculos sau inconsistent

Toolchain:
- spec check
- test generation
- proof obligations

Mai poate ramane:
- specificatii incomplete sau false

## 20. Epistemice

Limbaj:
- modele explicite
- fara magie

Compiler:
- invariants formale
- IR validat

Toolchain:
- differential tests
- specificatii
- dovezi unde conteaza

Mai poate ramane:
- intelegere gresita a problemei in sine

## Regula finala

Nu exista un singur mecanism anti-bug.

Reducerea reala de buguri apare doar din stratificare:
- limbajul blocheaza bugurile exprimabile
- compilerul blocheaza bugurile structurale
- toolchain-ul detecteaza bugurile emergente
- specificatiile reduc bugurile de design
- verificarea formala acopera zonele critice
- fuzzing-ul si corpusul real ataca necunoscutul
