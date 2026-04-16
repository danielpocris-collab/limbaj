# Sentinel Audit

Scop: identificam functiile din `ng_native.ng` care ascund semnificatie in valori precum `-1`, `0`, `""` sau default-uri tacite.

Fisier analizat:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng)

## Prioritate ridicata

### 1. `find_func(comp, name) -> i64`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:503)

Problema:
- intoarce `-1` pentru "not found"

De ce este fragil:
- semnificatia este ascunsa in valoare
- cere verificare manuala la fiecare call site
- daca verificarea lipseste, poate produce miscompile sau crash

Call site important:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:2935)

Inlocuitor recomandat:
- `find_func_opt(...) -> Option[i64]`

## Prioritate ridicata

### 2. `find_field_offset(comp, struct_name, field_name) -> i64`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:468)

Problema:
- intoarce `-1` pentru "field not found"

De ce este fragil:
- `-1` este foarte periculos intr-un backend care emite offset-uri si addressing
- daca scapam valoarea mai departe, riscul este direct de codegen gresit

Call sites relevante:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:1117)
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:1154)
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:1455)

Observatie:
- call site-urile trateaza doar cazul `foff == 0` ca optimizare de encoding
- nu exista validare explicita pentru `foff == -1`

Inlocuitor recomandat:
- `find_field_offset_opt(...) -> Option[i64]`
- plus diagnostic explicit la field missing

## Prioritate medie

### 3. `get_struct_size(comp, struct_name) -> i64`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:490)

Problema:
- intoarce `0` pentru "struct not found"

De ce este fragil:
- `0` poate insemna fie "missing", fie "size zero"
- chiar daca azi size zero poate fi imposibil, semantica este ambigua

Inlocuitor recomandat:
- `get_struct_size_opt(...) -> Option[i64]`

## Prioritate ridicata

### 4. `csv_get(csv, n) -> str`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:286)

Problema:
- intoarce `""` pentru index lipsa

De ce este fragil:
- `""` este si valoare valida de string
- ascunde eroarea de lookup
- polueaza toata semantica bazata pe `csv`

Impact:
- afecteaza lookup-uri de nume, tipuri, offsets si field lists

Inlocuitor recomandat:
- `csv_get_opt(...) -> Option[str]`

## Prioritate ridicata

### 5. `find_var_type(comp, name) -> str`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:962)

Problema:
- intoarce `"i64"` daca variabila nu exista

De ce este fragil:
- nu este doar sentinel, este inferenta falsa
- transforma "lookup failed" in "type valid"
- poate genera codegen gresit fara sa se vada imediat

Inlocuitor recomandat:
- `find_var_type_opt(...) -> Option[str]`
- diagnostic explicit daca variabila nu exista

## Prioritate ridicata

### 6. `find_var(comp, name) -> i64`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:926)

Problema:
- daca variabila nu exista, intoarce offset-ul pe care l-ar avea o variabila noua

De ce este fragil:
- ascunde lookup failure
- poate face ca o citire din variabila lipsa sa para valida
- combina "lookup" si "allocation convention" in aceeasi functie

Inlocuitor recomandat:
- `find_var_opt(...) -> Option[i64]`
- functie separata pentru calcul offset nou

## Prioritate medie

### 7. `parse_int(src, start, end) -> i64`

Loc:
- [ng_native.ng](C:/Users/pocri/OneDrive/Desktop/limbaj/.claude/worktrees/thirsty-proskuriakova/ng_native.ng:275)

Problema:
- nu semnaleaza explicit input invalid

De ce este fragil:
- presupune ca toate caracterele sunt cifre
- un input corupt devine rezultat numeric prost, nu eroare

Inlocuitor recomandat:
- `parse_int_result(...) -> Result[i64, ParseError]`

## Nu sunt prioritare ca sentinel bugs

Urmatoarele `0` sunt acceptabile sau deliberate in acest moment:
- token kind constants precum `tk_eof() -> 0`
- `csv_count("") -> 0`
- `count_params(...) -> 0` pentru lista vida

Aici `0` este semantica directa, nu sentinel ambiguu.

## Ordinea recomandata de reparare

1. `find_var_type`
2. `find_field_offset`
3. `find_func`
4. `find_var`
5. `csv_get`
6. `get_struct_size`
7. `parse_int`

## Prima schimbare concreta

Prima schimbare buna este:
- adaugam variante `_opt` pentru `find_func`, `find_field_offset`, `find_var_type`
- pastram variantele vechi temporar doar ca wrapper legacy
- introducem diagnostic explicit la call site-urile backend-ului
