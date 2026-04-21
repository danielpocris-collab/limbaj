# Bug Elimination Stack

Scop: acest fisier defineste explicit ce inseamna, la nivel de proiect, formula
`eliminam buguri pe toata suprafata declarata`.

Nu este doar un slogan despre un limbaj mai safe.
Este modelul de lucru pentru `ng -> Axiom`.

## Formula Proiectului

Scopul oficial este construirea unui sistem care elimina buguri pe toata suprafata declarata:

- program
- compiler
- runtime
- toolchain
- frontiere externe
- concurenta
- timp
- distributie
- securitate

Forma riguroasa a oricarui claim tare ramane:

- numai pe suprafata canonica suportata explicit
- numai modulo TCB-ul enumerat explicit
- numai dupa ce validatorii, fixed-point-ul si documentatia activa sunt verzi

## Ce Inseamna "Eliminam Buguri"

In proiectul acesta, `eliminam buguri` inseamna ca o clasa de buguri trebuie mutata din:

- conventii tacite
- review informal
- debugging post-factum
- testare ad-hoc

in una sau mai multe dintre urmatoarele forme:

- imposibil de exprimat
- obligatoriu de declarat
- verificabil automat
- validat prin contracte executabile
- dovedibil sau redus la proof obligations clare

## Straturile Stack-ului

### 1. Program si Model de Domeniu

Tintim eliminarea:

- starilor invalide
- `null` / sentinele ambigue
- cazurilor lipsa
- tranzitiilor ilegale intre stari
- efectelor ascunse in API-uri aparent pure

Mijloacele tinta:

- tipuri de domeniu
- `Option` / `Result`
- exhaustivitate
- state machines
- contracte
- effects explicite

### 2. Frontend de Compiler

Tintim eliminarea:

- parser recovery care schimba tacit semantica
- resolution ambiguu
- programe invalide care trec mai departe
- forme IR imposibile introduse din frontend

Mijloacele tinta:

- `parse, do not validate`
- AST-uri si forme interne rigide
- resolution explicit
- type checking fail-closed

### 3. IR, Pase si Invariants Interne

Tintim eliminarea:

- incoerente intre simboluri, tipuri si destinatii
- branch-uri invalide
- offset-uri invalide
- tabele nealiniate sau neparalele
- fallback-uri istorice care reactiva trasee fragile

Mijloacele tinta:

- backing canonic unic pentru date critice
- invariants pe fiecare pas
- validatori locali si globali
- translation validation unde proof-ul complet lipseste

### 4. Backend, Layout si Artefact Final

Tintim eliminarea:

- miscompile
- patch-uri invalide
- offset-uri efective incoerente
- binare respinse de loader pe suprafata suportata
- dependente fragile de layout fix

Mijloacele tinta:

- contracte explicite pentru `code/rdata/idata`
- validare de patch / RIP patch / offsets
- verificari de ABI si entrypoint
- builder final cu reguli verificabile, nu praguri tacite

### 5. Runtime Minim

Tintim eliminarea:

- failure model ambiguu
- allocator behavior necontractat
- exit path-uri incoerente
- builtins cu semantica implicita

Mijloacele tinta:

- contracte precise pentru `main`, args, I/O, exit codes si fatal diagnostics
- runtime mic
- boundary-uri explicite pentru alloc si process behavior

### 6. Toolchain si Bootstrap

Tintim eliminarea:

- build-uri nereproductibile
- fixed-point fals
- dependenta de stare reziduala in workspace
- trust chain neauditabil
- checkpoint-uri active nesincronizate cu documentatia

Mijloacele tinta:

- reproducible builds
- selfhost verificat
- bootstrap vertical verificat
- compare de bytes
- hash-uri canonice
- trust root explicit si restaurabil

### 7. Frontiere Externe

Tintim eliminarea:

- efecte ascunse la I/O, procese, fisiere, retea, FFI
- propagarea datelor brute dincolo de boundary-uri
- `unsafe` amestecat liber cu miezul sigur

Mijloacele tinta:

- boundary contracts
- fail-closed la margine
- FFI quarantine
- capability discipline

### 8. Concurenta, Timp si Distributie

Tintim eliminarea:

- race-uri semantice
- ordering bugs
- expirari uitate
- protocoale distribuite invalide
- stari remote tratate informal

Mijloacele tinta:

- session/protocol types
- actor discipline
- capabilitati
- modele explicite pentru ordering, causality si expiry

Acest strat este tinta oficiala, dar nu este inca suprafata canonica activa.

### 9. Securitate

Tintim eliminarea:

- secrete scapate prin API-uri comune
- capabilitati forjate
- boundary-uri cu privilegiu implicit
- interop nesigur care contamineaza miezul

Mijloacele tinta:

- security by construction
- capability model
- secret-aware typing unde este cazul
- quarantine strict pentru zonele nesigure

## Ordinea Corecta

Ordinea oficiala de executie ramane:

1. siguranta structurala a limbajului si compilerului
2. predictibilitatea runtime-ului
3. verificarea toolchain-ului si a bootstrap-ului
4. contracte, proof obligations si validation mai tare
5. boundary discipline
6. concurenta / timp / distributie
7. securitate integrata pe tot stack-ul

Aceasta ordine exista pentru a evita cresterea puterii sistemului mai repede decat creste verificabilitatea lui.

## Regula Pentru Feature-uri Noi

Orice feature nou trebuie sa raspunda clar la:

1. ce clasa de bug elimina
2. ce invariant nou introduce
3. ce cost semantic adauga
4. ce cost de runtime sau toolchain adauga
5. reduce sau mareste TCB-ul
6. la ce strat din acest stack apartine

Daca nu poate raspunde clar, nu intra in nucleul limbajului si nu intra in suprafata canonica.

## Regula Pentru Claim-uri Tari

Acest document defineste directia completa a proiectului.

Nu autorizeaza singur claim-uri de tip:

- `bug-free peste tot`
- `zero buguri in univers`
- `tot stack-ul este verificat complet`

Astfel de claim-uri raman permise doar pe suprafata canonica verde, dupa regulile din:

- [BUG_FREE_ACCEPTANCE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/BUG_FREE_ACCEPTANCE.md)
- [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md)
- [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md)
- [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md)

## Formula Scurta

`ng -> Axiom` nu inseamna doar un limbaj mai safe.

Inseamna construirea unui stack in care bugurile sunt eliminate sistematic din toate straturile declarate, iar ceea ce nu este inca eliminat este enumerat explicit, contractat si auditat.
