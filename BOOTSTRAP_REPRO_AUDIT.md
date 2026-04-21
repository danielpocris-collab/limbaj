# Direct Bootstrap Repro Audit

## Fixed-Point Result

- trusted == gen1: `True`
- gen1 == gen2: `True`

## Hashes

- trusted SHA-256: `D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653`
- gen1 SHA-256: `D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653`
- gen2 SHA-256: `D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653`

## Practical Verdict

- direct selfhost trust root is executable without Rust orchestration
- corpus and direct stack both passed on gen1 and gen2
- current bootstrap pair is byte-identical