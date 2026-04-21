# TCB

Scop: acest fisier enumera baza minima de incredere pentru claim-urile de corectitudine ale traseului canonic.

## Regula De Baza

- Tot ce nu este enumerat aici nu trebuie presupus tacit ca sigur.
- Orice claim de corectitudine inseamna `corect modulo TCB-ul si assumptions de mai jos`.
- Directia corecta este micsorarea TCB-ului, nu mascarea lui.

## Misiunea proiectului

Proiectul nu urmareste doar un limbaj mai safe in izolare.

Directia oficiala este construirea unui sistem care elimina buguri pe toata suprafata declarata:

- program
- compiler
- runtime
- toolchain
- frontiere externe
- concurenta
- timp
- distributie
- securitate

Acest fisier exista tocmai pentru ca orice astfel de claim trebuie facut riguros: suprafata exacta si baza minima de incredere trebuie enumerate, nu lasate implicite.

## Tinta Curenta Suportata

TCB-ul de mai jos se refera doar la:

- worktree-ul `thirsty-proskuriakova`
- traseul selfhost `ng_selfhost_clean.exe -> ng_native.ng`
- output PE x64 incarcat de Windows pe tinta suportata
- scripturile canonice de verificare din repo

## TCB Curent

### 1. Trust Root Promovat

- `ng_selfhost_clean.exe`
- size `175104`
- SHA-256 `D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653`

Acesta este checkpoint-ul executabil de la care porneste traseul canonic.

### 2. Sursa Compilerului Sub Test

- `ng_native.ng`

Aceasta sursa nu este presupusa infailibila; ea este obiectul verificarii. Totusi, pana la proof/validation mai puternic, corectitudinea ei este judecata prin contractele si testele canonice din repo.

### 3. Scripturile Canonice

- [run_fast_check.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/run_fast_check.ps1)
- [run_direct_stack.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/run_direct_stack.ps1)
- [run_bootstrap_vertical.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/run_bootstrap_vertical.ps1)
- [direct_toolchain_common.ps1](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/direct_toolchain_common.ps1)

Aceste scripturi sunt parte din lantul de verdict pentru selfhost, fixed-point si bootstrap.

### 4. Platforma De Executie

- Windows x64
- PowerShell
- loaderul PE al sistemului de operare
- filesystem-ul local si semantica de proces folosita de scripturi

Acestea sunt presupuneri externe ale traseului curent, nu proprietati dovedite de compiler.

### 5. Primitivele De Audit

- compare de bytes
- SHA-256 pentru artefacte
- verificarea ca binarele generate sunt PE x64 valide

Aceste primitive sunt folosite pentru verdictul fixed-point si bootstrap.

## Boundary-uri Externe

Zonele de mai jos nu fac parte din nucleul sigur si trebuie tratate ca boundary-uri explicite:

- fisiere si directoare externe compilerului
- proces creation / environment vars
- timp, scheduler si concurenta externa
- orice interop viitor sau FFI
- orice input brut din afara corpusului sau din afara contractelor parse-uite

## Ce Trebuie Scos Din TCB

Pe masura ce proiectul avanseaza, urmatoarele nu trebuie sa mai ramana baze tacite de incredere:

- metadata duplicate in CSV si store-uri canonice
- scanari text sau `csv_*` in hot path pentru date structurale
- presupuneri ascunse despre layout PE fix
- fallback-uri istorice care ocolesc backings canonice
- reguli operationale fragile dependente de stare reziduala in workspace

## Reguli De Reducere A TCB-ului

Orice schimbare noua trebuie sa respecte:

- muta datele critice in structuri canonice unice
- adauga contracte executabile la boundary-uri
- transforma presupunerile operationale in verificari automate
- separa clar ce este dovedit, ce este validat si ce este doar asumat
- nu creste TCB-ul fara motiv explicit in `STATUS.md`

## Gauri Curente De Incredere

Pe `2026-04-21`, TCB-ul curent ramane mai mare decat tinta din motive documentate:

- builderul PE depinde inca de layout fix pentru `.rdata` / `.idata`
- verdictul canonic trebuie luat din rulare seriala fiindca scripturile pot reutiliza aceleasi directoare locale

Aceste puncte trebuie eliminate inaintea oricarui claim puternic de corectitudine.

## Tinta Pe Termen Mediu

TCB-ul vizat pentru claim-uri mai tari trebuie sa ramana aproximativ:

- trust root promovat clar
- semantic core scris si inghetat
- validatorii canonici de layout / patch / bootstrap
- un lant de build reproductibil si auditabil
- boundary-uri externe mici si explicit contractate

Tot ce poate fi mutat din `trust me` in `check me` trebuie mutat.
