# NG to Axiom Roadmap

Scop: acest fisier fixeaza directia oficiala a proiectului pentru oameni si pentru alte LLM-uri.

Regula de baza:
- nu aruncam `ng`
- nu rescriem de la zero
- nu construim un limbaj paralel complet separat
- evoluam `ng` in directia `Axiom`

`ng` este baza actuala.
`Axiom` este arhitectura-tinta.

## Misiunea proiectului

Scopul proiectului nu este doar sa faca limbajul mai safe in izolare.

Scopul proiectului este sa construiasca un sistem care elimina buguri pe toata suprafata declarata:

- program
- compiler
- runtime
- toolchain
- frontiere externe
- concurenta
- timp
- distributie
- securitate

Asta inseamna ca `ng -> Axiom` trebuie gandit ca un stack de corectitudine, nu doar ca o sintaxa sau un type checker mai bun.

Documentul operational care descrie explicit acest stack este:
- [BUG_ELIMINATION_STACK.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_ELIMINATION_STACK.md)

## 1. Decizia strategica

Proiectul actual continua pe worktree-ul:
- [thirsty-proskuriakova](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova)

Directia oficiala este:
- compilerul actual se pastreaza
- hardening-ul continua
- designul nou urmeaza principiile `Axiom`
- orice schimbare mare trebuie evaluata prin prisma `Axiom`
- orice schimbare mare trebuie evaluata si prin intrebarea: reduce buguri pe suprafata declarata sau doar muta complexitatea in alta parte?

## 2. Ce pastram din `ng`

Pastram:
- sintaxa mica si directa
- `flow`
- distinctia `bind` / `vary`
- compilatorul nativ actual
- traseul de selfhost actual
- backend-ul direct PE
- worktree-ul actual ca baza activa

Nu facem ruptura inutila cu ce functioneaza deja.

## 3. Ce preluam din `Axiom`

Preluam ca directie de design:
- invalid states are unrepresentable
- parse, do not validate
- effects explicite
- unsafe quarantined
- TCB minim
- runtime predictibil
- frontiere externe contractate explicit
- toolchain verificabil
- selfhosting disciplinat, nu circular trust

## 4. Ce implementam prima data

Ordinea oficiala este aceasta:

### Faza A — Axiom Core peste `ng`

Introducem:
- ownership/linearity simpla
- `Option` / `Result`
- efecte explicite
- eliminarea sentinelilor si fallback-urilor ascunse
- invariants mai stricte in compiler
- IR intern mai rigid

Aceasta faza are prioritate maxima.

### Faza B — Runtime si Toolchain disciplinat

Introducem:
- runtime mai predictibil
- corpus minim automat
- reproducible builds
- selfhost verificat
- fixed-point verificat
- tracing si failure model clare

### Faza C — Proof Layer gradual

Introducem:
- contracte
- refinements limitate
- proof obligations selectate
- validari de transformare pe pase critice

Nu incercam proof-heavy peste tot din prima.

### Faza D — Boundary Discipline

Introducem:
- FFI quarantine
- capabilitati
- boundary contracts
- fail-closed pentru zone critice

### Faza E — Distributed Layer

Introducem mai tarziu:
- actor model disciplinat
- session/protocol types
- choreography / endpoint discipline

Acest strat nu este prioritar pana cand nucleul local nu este stabil.

## 5. Ce amanam intentionat

Nu incercam imediat:
- proof kernel complet
- backend complet verificat formal
- SMT everywhere
- dependent types grele peste tot
- distributed verification completa
- certified FFI complet pe toate frontierele

Acestea sunt tinta finala, nu punctul de start.

## 6. Reguli pentru alte LLM-uri

Daca un LLM modifica proiectul, trebuie sa respecte urmatoarele:

1. Nu propune rescriere completa daca exista cale de migrare incrementala.
2. Nu adauga feature-uri care cresc puterea limbajului dar scad predictibilitatea.
3. Prefera eliminarea claselor de buguri inaintea adaugarii de sintaxa noua.
4. Prefera invariants si frontiere explicite in locul conventiilor tacite.
5. Nu introduce mecanisme magice sau implicite daca acelea ascund cost, efecte sau ownership.
6. Nu creste TCB-ul fara justificare clara.
7. Orice interop/unsafe trebuie izolat, nu amestecat liber cu miezul sigur.
8. Orice schimbare de compiler trebuie gandita si in termeni de selfhost, reproducibilitate si fixed-point.

## 7. Regula de evaluare a feature-urilor

Orice feature nou trebuie evaluat prin 5 intrebari:

1. Ce clasa de bug elimina?
2. Ce invariant nou introduce?
3. Ce cost semantic adauga?
4. Ce cost de runtime sau toolchain adauga?
5. Reduce sau mareste TCB-ul?

Daca nu reduce clar riscul sistemic, nu intra in nucleul limbajului.

## 8. Regula de implementare

Ordinea corecta este:
- intai siguranta structurala
- apoi predictibilitatea runtime-ului
- apoi verificarea toolchain-ului
- apoi proof layer
- abia dupa aceea extensii mari de putere

## 9. Formula scurta

`ng` nu este abandonat.

`ng` devine drumul de executie catre `Axiom`.

Nu construim altceva paralel.
Transformam ceea ce avem deja intr-un ecosistem:
- mai strict
- mai verificabil
- mai predictibil
- mai putin dependent de debugging clasic
