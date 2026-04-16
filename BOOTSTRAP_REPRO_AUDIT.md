# Direct Bootstrap Repro Audit

## Fixed-Point Result

- trusted == gen1: `True`
- gen1 == gen2: `True`

## Hashes

- trusted SHA-256: `A452D8EC965395CA0C7BE8FC3EF82CF3943A8DD15E9D1F516AC002113DCDAD31`
- gen1 SHA-256: `A452D8EC965395CA0C7BE8FC3EF82CF3943A8DD15E9D1F516AC002113DCDAD31`
- gen2 SHA-256: `A452D8EC965395CA0C7BE8FC3EF82CF3943A8DD15E9D1F516AC002113DCDAD31`

## Practical Verdict

- direct selfhost trust root is executable without Rust orchestration
- corpus and direct stack both passed on gen1 and gen2
- current bootstrap pair is byte-identical