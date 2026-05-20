# tx_builder

Ergonomic Cardano transaction builder built on top of [`cardano-serialization-lib`](https://docs.rs/cardano-serialization-lib) (CSL). Focused on dApp / Plutus script interactions and full Conway-era governance.

## Scope

- **Build only.** You bring your own selected UTxOs and an external signer. `build()` returns an unsigned `csl::Transaction` with the witness set already populated for scripts / datums / redeemers — just attach `Vkeywitness` entries on the signing side.
- **No coin selection, no provider integration, no signing.** Those concerns live elsewhere by design — keeping this layer thin makes it composable.

## What's in the box

| Area | API |
|---|---|
| Inputs | `SpendingInput::Pubkey` / `Plutus` / `NativeScript` |
| Reference inputs | `add_reference_input` (or implicit via `ScriptSource::Reference`) |
| Outputs | `Output::new(addr).lovelace(..).assets(..).inline_datum(..).script_ref(..)` |
| Collateral | `add_collateral`, `collateral_return`, `total_collateral` |
| Mint / burn | `MintAction::plutus(..)` / `MintAction::native(..)` with signed quantities |
| Certificates | `Cert::*` for staking and all Conway-era flavors |
| Withdrawals | `add_withdrawal`, `add_plutus_withdrawal` |
| Governance — voting | `add_vote(Vote { voter, action, choice, anchor })` |
| Governance — proposals | `add_proposal(ProposalSpec { .. })` |
| Validity | `validity_start`, `ttl` |
| Metadata | `metadata(label, datum)`, `auxiliary_script` |

## Example

See [`examples/plutus_spend.rs`](examples/plutus_spend.rs) for a Plutus spend + mint with a reference script and collateral.

Minimal pubkey-only transfer:

```rust
use tx_builder::prelude::*;

let tx = {
    let mut b = TxBuilder::new(&params);
    b.change_address(&wallet)
     .add_pubkey_input(&utxo)
     .add_output(Output::new(&dest).lovelace(2_000_000))
     .ttl(100_000_000);
    b.build()?
};

let cbor = hex::encode(tx.to_bytes());
```

## Build pipeline

`TxBuilder::build()` walks this fixed order, mirroring what CSL needs:

1. Inputs (regular + Plutus + native-script).
2. Reference inputs (explicit + implicit from `ScriptSource::Reference`).
3. Outputs.
4. Collateral inputs / return / total.
5. Required signers.
6. Mint.
7. Certificates.
8. Withdrawals.
9. Voting & proposals.
10. Validity range.
11. Metadata + auxiliary scripts.
12. Script-data hash (only if Plutus witnesses present).
13. `add_change_if_needed(change)`.
14. `build_tx()`.

## Known sharp edges

- **Ex-units are placeholders.** Pass the `csl::ExUnits` you get from an evaluator (Ogmios `evaluateTransaction`, Blockfrost `tx/evaluate`, your own `aiken simulate`, etc.). The values you give to `SpendingInput::plutus` / `MintAction::plutus` are used verbatim — there's no on-the-fly evaluation in this crate.
- **Redeemer indices.** CSL fixes the index after sorting inputs / mints — the `0` placeholders inside the builder are intentional.
- **CSL method-name drift.** CSL is in active development; if a method like `set_collateral_return_and_total` or `add_with_plutus_witness` has been renamed between minor versions, you'll get a clear compile error and a one-line fix.
- **Cost models.** Pass *all* cost models you might need (PlutusV1/V2/V3) in `ProtocolParameters::cost_models`. CSL hashes only the ones referenced, so over-supplying is safe.

## Alternatives worth knowing

- [`cardano-multiplatform-lib`](https://github.com/dcSpark/cardano-multiplatform-lib) — dcSpark's fork of CSL with a different design.
- [`pallas-txbuilder`](https://github.com/txpipe/pallas) — Rust-native (no WASM lineage), part of the broader Pallas stack.

For pure-Rust environments (no JS interop) `pallas-txbuilder` is often cleaner; this crate is the right pick when you're already locked to CSL.
