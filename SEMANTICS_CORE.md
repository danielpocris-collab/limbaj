# Semantics Core

Scop: acest fisier fixeaza core-ul semantic minim pentru care putem discuta serios despre corectitudine si, mai tarziu, despre claim-uri `bug-free`.

## Regula De Interpretare

- Daca un comportament nu este definit aici sau trimis explicit aici din alte documente canonice, el nu intra in claim-ul tare de corectitudine.
- `Semantics Core` este suprafata care trebuie inghetata inainte de proof, validation puternic si acceptance final.

## Suprafata Vizata

Core-ul semantic vizat acopera:

- evaluarea programelor `ng` care intra pe traseul canonic selfhost
- reprezentarile si invariants-urile pe care compilerul le foloseste pentru a compila `ng_native.ng`
- contractele de baza dintre frontend, IR, codegen, runtime minim si builderul PE

Nu acopera inca:

- FFI general
- retea
- timp si nondeterminism extern
- straturi distribuite
- orice feature experimental care nu este promovat in traseul canonic

## Proprietati Fundamentale

### 1. Determinism

Pe core-ul suportat:

- acelasi input semantic + acelasi toolchain canonic -> acelasi output
- ordinea de evaluare trebuie sa fie explicita si stabila
- compilerul nu are voie sa depinda de ordine implicite sau de stare reziduala din workspace

### 2. Valori Si Tipuri

Pe core-ul suportat:

- valorile au shape verificabil, nu conventii ascunse
- `null` nu este parte din modelul semantic
- absenta si esecul trebuie modelate explicit, nu prin sentinele
- conversiile intre tipuri numerice trebuie sa fie explicite

### 3. Bindings Si Mutatie

Core-ul semantic trebuie sa mentina:

- `bind` ca forma implicita si stabila pentru valori care nu muta stare
- `vary` doar pentru mutatii reale
- fara transferuri implicite de ownership
- fara aliasing mutabil tacit

### 4. Ownership Si Heap Shapes

Pentru `str`, `buf` si alte forme cu heap:

- ownership-ul trebuie sa fie clar
- imprumutul read-only si imprumutul mutabil trebuie sa fie distincte
- mutatia prin alias neclar nu intra in suprafata sigura
- alocarile care pot esua trebuie sa aiba model de eroare explicit

### 5. Efecte

Pe traseul core:

- functiile pure, cu alloc, cu I/O sau cu efecte de proces trebuie separate semantic
- efectele nu trebuie mascate in helpers sau fallback-uri
- boundary-urile externe trebuie contractate explicit

### 6. Failure Model

Comportamentul la eroare trebuie sa fie fail-closed.

Asta inseamna:

- operatiile care pot esua nu comunica esecul prin conventii ambigue
- runtime fatal are format unic
- validarea boundary-urilor se face la margine, nu dupa propagarea datelor brute

### 7. Frontend Semantics

Frontend-ul trebuie sa conserve semnificatia programului fara recuperari tacite.

Minim:

- parserul construieste doar forme acceptate de gramatica suportata
- resolution-ul face mapping explicit de la nume la simbol
- type checking-ul elimina programele care nu respecta contractele core-ului

### 8. IR Semantics

IR-ul si reprezentarile interne trebuie sa aiba invariants verificabile.

Minim:

- fiecare simbol rezolvat are tip si destinatie stabila
- fiecare jump sau branch are tinta valida
- fiecare apel are aritate si ABI coerente
- fiecare acces la stack si la store are offset valid
- tabelele de patch raman paralele pe tot traseul

### 9. Backend Semantics

Codegen-ul trebuie sa respecte semantica IR-ului pe tinta suportata.

Minim:

- layout-ul final `code/rdata/idata` este contractat explicit
- relocarile si patch-urile au reguli verificabile
- intrarea in program si builtins-urile folosesc offset-uri canonice
- binarul final ramane valid pentru loaderul PE suportat

### 10. Runtime Minim

Runtime-ul minim trebuie sa aiba contracte precise pentru:

- `main`
- args
- file input
- allocator / memory failure
- exit codes
- fatal diagnostics

## Ce Inseamna Corectitudine Pe Acest Core

Pentru acest document, `corectitudine` inseamna simultan:

- programul sursa este interpretat dupa regulile semantice de mai sus
- compilerul nu pierde si nu distorsioneaza acea semantica pe traseul suportat
- runtime-ul si builderul final nu introduc deviatii cunoscute fata de contract

## Ce Nu Inseamna Inca

Acest document nu afirma inca:

- proof complet pentru tot limbajul
- absenta bugurilor in afara suprafetei definite aici
- corectitudine pe orice platforma sau orice OS
- corectitudine pentru feature-uri necontractate explicit

## Regula De Evolutie

Orice feature nou intra in `Semantics Core` doar daca raspunde clar la:

- ce invariant nou introduce
- ce clasa de bug elimina
- ce boundary nou deschide
- cum este validat sau dovedit

Daca nu poate raspunde clar, ramane in afara claim-ului tare de corectitudine.
