# NG Strict Plan

Scop: adaptam `ng`-ul actual astfel incat sa reduca simultan:
- buguri de memorie
- buguri de logica
- buguri de design
- buguri de integrare
- surprize de performanta
- probleme din date reale neasteptate
- nevoia de debugging clasic

Nu rescriem limbajul. Intarim semantic, compilerul, runtime-ul si toolchain-ul.

## Principii

1. Erorile predictibile trebuie mutate din runtime in compile-time.
2. Efectele si alocarile trebuie sa fie vizibile in cod si in ABI.
3. Output-ul compilerului trebuie sa fie determinist.
4. Reprezentarile interne trebuie sa aiba invariante verificabile, nu conventii fragile.
5. Datele externe trebuie validate la margine, nu propagate brut prin sistem.

## Faza 1: Strict Core

Obiectiv: pastram sintaxa actuala, dar facem comportamentul mai rigid.

Reguli:
- `bind` ramane implicit si este forma recomandata.
- `vary` este permis doar pentru valori mutate efectiv.
- Nu introducem `null`.
- Nu folosim valori sentinel ca semnificatie ascunsa.
- Indexarea este checked by default.
- Conversiile numerice devin explicite.
- Overflow-ul pentru operatii semnate este definit si verificat in mod debug/strict.
- Functiile care pot esua nu mai intorc `-1`, `0`, `""` sau alte conventii ambigue.

Livrabile:
- `Option[T]`
- `Result[T, E]`
- verificari de bounds pentru string/buffer indexing
- reguli clare pentru conversii `i64 <-> byte <-> bool`

## Faza 2: Memorie si Ownership

Obiectiv: eliminam aliasing-ul neclar si transferurile implicite de ownership.

Model propus:
- tipurile de valoare mici raman copy
- `str`, `buf`, structurile care contin heap devin ownership-aware
- avem 3 moduri de folosire:
  - owner
  - borrow read-only
  - borrow mutable exclusiv

Reguli:
- nu exista doua imprumuturi mutabile simultane pe acelasi obiect
- mutatia prin alias implicit este interzisa
- functiile trebuie sa spuna daca preiau ownership sau doar imprumuta
- alocarile care pot esua intorc `Result`

Prima tinta practica:
- definirea semantica pentru `str`
- definirea semantica pentru `buf`
- eliminarea conventiilor fragile de tip `csv` ca structura interna principala

## Faza 3: Semantica datelor

Obiectiv: datele sa aiba shape verificabil.

Adaugari:
- `enum` tagged
- pattern matching exhaustiv
- destructuring simplu

Reguli:
- orice `match` pe variante trebuie sa fie exhaustiv
- parsarea datelor externe trebuie sa produca tipuri validate
- API-urile interne nu trebuie sa foloseasca `str` brut unde exista un tip mai precis

Prima tinta practica:
- tokeni, AST, IR, diagnostice, path-uri, simboluri

## Faza 4: Efecte explicite

Obiectiv: reducem bugurile de integrare si design ascuns.

Separare:
- functii pure
- functii cu alloc
- functii cu I/O
- functii care ating procesul sau sistemul de fisiere

Reguli:
- efectele devin vizibile in semnatura sau in metadata IR
- compilerul poate refuza apeluri periculoase in contexte pure
- teste si optimizari se bazeaza pe aceasta distinctie

## Faza 5: Compiler intern mai rigid

Obiectiv: eliminam clase intregi de buguri din implementare.

Schimbari:
- introducem IR intern tipat
- renuntam treptat la reprezentari bazate pe stringuri concatenate pentru metadate
- validam IR dupa fiecare pas major

Invariante minime:
- fiecare simbol rezolvat are tip stabil
- fiecare branch are destinatie valida
- fiecare acces la stack are offset valid
- fiecare apel respecta ABI si aritatea
- fiecare string literal are ownership/locatie clara

Prima tinta practica:
- `collect_functions` produce structuri reale, nu tabele `csv`
- `compile_function` lucreaza pe IR/AST tipat, nu pe conventii text

## Faza 6: Runtime minim si auditat

Obiectiv: runtime mic, previzibil, usor de inspectat.

Reguli:
- allocatorul este clar si centralizat
- codurile de iesire sunt standardizate
- erorile fatale au format unic
- tracing-ul se poate activa fara debugger
- runtime-ul nu contine magie ascunsa

Prima tinta practica:
- modul comun pentru alloc/free/error/trace
- contract clar pentru `main()`, args, file input si failure modes

## Faza 7: Toolchain verificabil

Obiectiv: scadem nevoia de debugging clasic prin verificari automate tari.

Verificari obligatorii:
- selfhost
- fixed-point
- build determinist
- corpus tests pe fisiere reale
- property tests pentru parser si semantic checking
- differential tests intre stage-uri
- reducer pentru inputuri care produc crash sau miscompile

CI minima:
- compileaza corpus mic si corpus mare
- compara artefacte intre 2 rulari
- ruleaza programe de smoke test
- ruleaza cel putin un test selfhost complet

## Faza 8: Performance fara surprize

Obiectiv: performanta predictibila, nu doar rapida pe exemple mici.

Reguli:
- evitam alocari ascunse in primitive frecvente
- evitam copii implicite de string/buffer
- introducem metrici simple:
  - alloc count
  - bytes alocati
  - dimensiune PE
  - timp pe faza
- regresiile de performanta devin detectabile automat

## Ordinea de implementare

1. Specificam `Option` si `Result`.
2. Definim ownership pentru `str` si `buf`.
3. Introducem bounds checks si conversii explicite.
4. Inlocuim structurile interne fragile din compiler cu tipuri reale.
5. Introducem IR validat.
6. Standardizam runtime-ul si tracing-ul.
7. Automatizam selfhost, fixed-point si build determinist.
8. Abia dupa aceea extindem limbajul cu enum/pattern matching complet.

## Regula strategica

Orice feature nou trebuie sa raspunda la 3 intrebari:
- ce invariante noi introduce
- ce buguri elimina
- ce cost semantic sau de runtime adauga

Daca nu reduce clar riscul sistemic, nu intra in nucleul limbajului.
