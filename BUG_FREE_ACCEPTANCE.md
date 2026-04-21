# Bug-Free Acceptance

Scop: acest fisier defineste cand proiectul poate spune onest `bug-free` si pe ce suprafata exacta.

## Formula Oficiala

- Nu folosim `0 bug-uri peste tot` ca slogan liber.
- Putem spune cel mult: `bug-free pentru core-ul suportat, modulo TCB-ul si assumptions enumerate`.
- Daca un singur acceptance gate este rosu, verdictul oficial este `not bug-free yet`.

## Misiunea proiectului

Misiunea proiectului este mai larga decat un limbaj doar `memory-safe` sau doar un compilator care trece testele.

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

Asta este directia de proiect.

Formulele de `bug-free` din acest fisier raman totusi limitate la suprafata canonica suportata acum si la TCB-ul explicit din [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md).

## Suprafata Vizata

Claim-ul se poate aplica doar pentru:

- traseul canonic selfhost `ng_selfhost_clean.exe -> ng_native.ng -> output.exe`
- verificarea canonica prin:
  - `powershell -ExecutionPolicy Bypass -File .\run_fast_check.ps1`
  - `powershell -ExecutionPolicy Bypass -File .\run_direct_stack.ps1`
  - `powershell -ExecutionPolicy Bypass -File .\run_bootstrap_vertical.ps1 -MaxParallel 2`
- platforma suportata explicit in [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md)
- core-ul semantic definit in [SEMANTICS_CORE.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/SEMANTICS_CORE.md)

Orice comportament, feature sau platforma din afara acestor limite este in afara claim-ului.

## Acceptance Gates

### 1. Semantics Freeze

Trebuie sa existe un core semantic scris, versionat si stabil.

Minim:

- ordine de evaluare definita explicit
- model de eroare explicit
- conversii numerice explicite
- reguli de ownership / aliasing explicite
- contract pentru builtins si boundary-uri externe
- fara sentinele sau fallback-uri ascunse in miez

### 2. Zero Known Correctness Bugs

[STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md) nu trebuie sa mai contina riscuri deschise de corectitudine pe core-ul suportat.

Asta include:

- fara miscompile cunoscut
- fara cliff cunoscut in backend pe tinta suportata
- fara contract final incomplet pe traseul canonic
- fara workaround operational care schimba verdictul semantic

### 3. Single Source of Truth

Datele critice din compiler trebuie sa aiba backing canonic unic.

Nu acceptam:

- metadata duplicate care pot deriva independent
- structuri CSV sau text folosite drept backing primar pentru hot path
- fallback-uri vechi care pot reactiva trasee fragile

### 4. Frontend Soundness

Lexerul, parserul, resolution-ul si type checking-ul trebuie sa fie fail-closed pe core-ul suportat.

Minim:

- programele invalide sunt respinse
- programele valide nu sunt respinse arbitrar
- AST-ul si IR-ul nu admit stari imposibile fara verificari explicite
- diagnosticele nu ascund recuperari semantice tacite

### 5. IR si Pass Invariants

Fiecare pas critic din compiler trebuie fie sa preserve invariants, fie sa le revalideze explicit.

Minim:

- fiecare simbol rezolvat are tip stabil
- fiecare branch are destinatie valida
- fiecare acces la stack are offset valid
- fiecare apel respecta ABI si aritatea
- fiecare tabel `offset/target` ramane paralel si aliniat

### 6. Backend si Layout Final

Contractul final pentru codegen si PE layout trebuie sa fie complet pe tinta suportata.

Minim:

- `validate_final_layout_contract(...)` este complet pentru traseul suportat
- `validate_final_patch_table(...)` si `validate_final_rip_patch_table(...)` acopera toate intrarile relevante
- sectiunile PE nu depind de praguri fixe fragile
- binarul rezultat este acceptat de loader pe intreaga suprafata suportata

### 7. Runtime si Builtins

Runtime-ul minim si builtins-urile trebuie sa aiba contracte explicite.

Minim:

- allocatorul si error path-urile sunt definite
- args, file input, exit codes si fatal errors sunt standardizate
- nu exista stare globala ascunsa care schimba semantica

### 8. Determinism si Selfhost

Toolchain-ul trebuie sa fie determinist pe traseul canonic.

Minim:

- build repetat -> artefact identic
- stage `0 -> 1 -> 2` respecta fixed-point-ul asteptat
- verdictul canonic vine din rulare seriala curata
- checkpoint-ul promovat corespunde exact hash-ului/documentatiei active

### 9. Validation si Proof Obligations

Testing-ul singur nu este suficient.

Trebuie sa existe:

- contracte executabile pe pasele critice
- translation validation sau proof obligations acolo unde proof complet lipseste
- property, differential si fuzz testing pentru zonele inca nedovedite

### 10. Trust Chain Auditabil

TCB-ul trebuie sa fie explicit, mic si auditat.

Minim:

- [TCB.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/TCB.md) este actualizat
- trust root-ul promovat este clar
- reproducibility si bootstrap verdict-ul sunt arhivate
- orice presupunere externa este enumerata, nu implicita

### 11. Release Rule

Claim-ul `bug-free` se poate face doar pentru un checkpoint promovat oficial.

Checkpoint-ul trebuie sa aiba simultan:

- hash oficial
- status verde pe entrypoint-urile canonice
- documentatie sincronizata
- zero riscuri deschise de corectitudine in `STATUS.md`

## Evidenta Minima Ceruta

Inainte de orice claim `bug-free`, trebuie sa existe in repo:

- checkpoint promovat cu size si SHA-256
- update in `STATUS.md`
- update in `BOOTSTRAP_REPRO_AUDIT.md` daca se schimba fixed-point-ul
- referinta clara la `SEMANTICS_CORE.md` si `TCB.md`
- verdictul exact: ce este inclus si ce este exclus

## Verdict Curent

Pe `2026-04-21`, worktree-ul curent nu este eligibil pentru claim `bug-free`.

Blocantul documentat acum este:

- builderul PE are inca un cliff cunoscut pe layout fix `.rdata` / `.idata`

Checkpoint-ul promovat oficial curent este `175104` cu SHA-256 `D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653`, dar asta nu inchide claim-ul `bug-free` cat timp blocantul de mai sus ramane deschis.

Pana cand aceste puncte nu dispar din [STATUS.md](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/STATUS.md), verdictul oficial ramane `not bug-free yet`.
