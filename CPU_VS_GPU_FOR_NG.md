# CPU vs GPU for ng

Scop: clarificare pragmatica despre unde merita folosit CPU multicore si unde ar putea avea sens GPU.

## 1. Verdict scurt

Pentru compilerul actual `ng`, prioritatea corecta este:

- **CPU multicore**

nu:

- GPU pentru nucleul compilerului

## 2. De ce CPU multicore este alegerea corecta acum

Nucleul compilerului are:

- parsing dependent de context
- branching frecvent
- string processing neregulat
- metadata interdependenta
- patching dependent de offseturi globale
- multe lookup-uri mici

Acestea sunt workload-uri potrivite pentru:

- cache CPU
- task-uri per functie
- thread pool
- memorie partajata cu cost mic

## 3. De ce GPU NU este pasul corect acum

GPU castiga cand ai:

- foarte multe operatii uniforme
- loturi mari
- putin branching
- acces regulat la memorie
- aritmetica masiva pe date omogene

Compilerul actual NU arata asa.

Riscurile ar fi:

- complexitate mult mai mare
- transfer CPU <-> GPU costisitor
- debugging mult mai greu
- determinism mai greu de controlat
- suprafata mai mare de bug si TCB mai mare
- castig probabil mic pe pipeline-ul actual

## 4. Ce trebuie mutat pe CPU multicore

Prioritati reale:

1. compilare per functie ca job independent
2. concatenare canonica dupa joburi
3. patch metadata locala per functie
4. patch final/global dupa rebuild de offseturi

Asta este sursa mare de parallel speedup.

## 5. Ce ar putea folosi GPU mai tarziu

Nu nucleul compilerului, ci eventual:

- fuzzing masiv
- analiza de corpus foarte mare
- token scanning pe loturi mari de fisiere
- validari sau transformari foarte regulate pe bufferi mari
- tooling auxiliar, nu core compiler

## 6. Regula de decizie

Pentru fiecare componenta intreaba:

- este data-parallel si uniforma?
- are branching mic?
- are acces regulat la memorie?
- merita transferul CPU <-> GPU?

Daca raspunsul nu este clar „da”, ramane pe CPU.

## 7. Aplicat pe starea curenta

Acum avem:

- verticala selfhost inchisa
- `gen1 == gen2` byte-identic
- corpus verde

Deci urmatorul pas util nu este GPU.

Urmatorul pas util este:

- `compile_function` cu output local per functie
- apoi joburi paralele pe CPU

## 8. Formula practica

- **acum:** CPU multicore
- **mai tarziu:** GPU doar pentru task-uri auxiliare bine alese

Aceasta este directia corecta pentru `ng`.
