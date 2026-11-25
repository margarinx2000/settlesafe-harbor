# SettleSafe — Escrowed Invoice Payments on ME Network

**Escrow for invoice payments: the buyer's money moves only when the work does.**

SettleSafe is a CosmWasm smart contract + demo UI for escrowed invoice
payments, built for the **MetaEarth HARBOR Hackathon 2026** (Track 02 —
Consumer Products & Payments) and deployed live on **ME Beta Testnet**
(chain `mechain_400-1`).

- **Contract:** `me1qwy2pa40vhvt8hetkzky3shhd7pnau4rgerk3j75fpvt8gejallqw8d7u4`
  (code ID **45**, label "SettleSafe Escrow v0.1.0", no admin)
- **Demo video (3:38, English):** https://www.youtube.com/watch?v=xhFA3tQWEbg
- **Team:** SettleSafe — Margarin (captain) & Wahyuni, Bireuen, Indonesia

## The problem

Freelancers and micro-sellers get paid late — or never — because buyers
demand delivery before payment while sellers demand payment before delivery.
SettleSafe removes the trust requirement: the buyer locks the payment in an
on-chain escrow contract, and the contract releases it only when delivery is
confirmed, with a dispute path and deadline-based safety rails for every
unhappy path.

## How it works

A seller creates an invoice naming the buyer, an independent arbiter, the
amount (native MEC, escrow denom configurable at instantiate — `umec` on the
testnet deployment), and three time windows: payment, delivery, review.

State machine:

```
Created → Funded → Delivered → Released
Created → Cancelled            (either party anytime; anyone after the pay deadline)
Funded  → Refunded             (anyone, after the deliver deadline, if never delivered)
Delivered → Released           (seller claim_after_review once the review window expires)
Funded|Delivered → Disputed → Released | Refunded   (arbiter only, binary outcome)
Funded|Delivered → Released    (buyer release)
```

Execute messages: `create_invoice`, `fund_invoice` (buyer attaches exactly
the invoice amount), `cancel_invoice`, `mark_delivered`, `release`,
`claim_after_review`, `refund_after_deadline`, `raise_dispute`,
`resolve_dispute`. Queries: `get_invoice`, `invoice_count`, `total_escrowed`.

Safety properties: role checks on every transition (only the buyer funds and
releases, only the seller delivers and claims, only the named arbiter
resolves disputes), exact-amount funding enforcement, deadline enforcement
on every window, and checks-effects-interactions ordering throughout.

## Repository layout

| Path | Contents |
|---|---|
| `contract/` | CosmWasm contract (Rust): `src/contract.rs`, `src/msg.rs`, `src/state.rs`, `src/error.rs`, `src/tests.rs` (32 unit tests), JSON schema in `schema/` |
| `demo-ui/` | Zero-dependency single-page demo (`index.html`) that reads the live contract state via the ME Hub REST endpoint and builds every execute message + equivalent `med` command |
| `DEPLOY-RUN.md` | Full deployment & lifecycle run record (tx hashes, fees, final balances) |

## Build & test the contract

```bash
cd contract
cargo test                                # 32/32 unit tests
cargo build --lib --release --target wasm32-unknown-unknown
```

Notes (from the build record):

- The deployed artifact was built with **Rust 1.80.1** (inside the ME docs
  range 1.75–1.80): `settlesafe_cw.wasm`, 260,393 bytes,
  sha256 `2fe032a5f43ba1a52cf9124586e7af620238e5fcff2a407036fa3617fe3c778d`.
- `.cargo/config.toml` adds `link-arg=--allow-undefined` for wasm32 —
  required so the VM-provided imports link.
- Wasm builds must use `--lib` (the schema binary refuses wasm32).
- `Cargo.lock` pins `base64ct 1.6.0` and `zeroize 1.7.0`; do **not** run a
  bare `cargo update` — newer releases of both require edition2024 and break
  the Rust 1.80 build.
- The 32 tests mirror the 27 cases of the original Solidity escrow suite
  1:1, plus 5 CosmWasm-specific cases (wrong amount, wrong denom,
  zero window, stranger-cancel-before-expiry, no-such-invoice).
- Regenerate the JSON schema with `cargo run --bin schema`.

## Deploy (ME Beta Testnet)

Using the `med` CLI against the Hub RPC
(`https://beta-hub-26657.explorer-testnet.me`, chain `mechain_400-1`):

```bash
med tx wasm store settlesafe_cw.wasm \
  --from <deployer> --chain-id mechain_400-1 --node <RPC> \
  --gas auto --gas-adjustment 1.3 --gas-prices 0.1umec -y        # → CODE_ID

med tx wasm instantiate <CODE_ID> '{"escrow_denom":"umec"}' \
  --label "SettleSafe Escrow v0.1.0" --no-admin \
  --from <deployer> --chain-id mechain_400-1 --node <RPC> \
  --gas auto --gas-adjustment 1.3 --gas-prices 0.1umec -y        # → contract address
```

## Run the demo UI

```bash
cd demo-ui
python3 -m http.server 8000
# open http://localhost:8000
```

The page queries the contract live (invoice #0, invoice count, total
escrowed) and falls back to the recorded `DEPLOY-RUN.md` snapshot if the
REST endpoint is unreachable. See `demo-ui/README.md` for details.

## ME Network run records — invoice #0 (full lifecycle, 25 Nov 2026)

Invoice #0: "Logo design package", 50,000,000 umec (0.5 MEC),
seller → buyer, arbiter as named in the contract. Full record:
[`DEPLOY-RUN.md`](DEPLOY-RUN.md).

| Step | Tx hash | Explorer |
|---|---|---|
| Store code (CODE_ID 45) | `53D8D15C87E49B9AAB186397D3E5145BE45092383BEC1700FED13CA54DCD6350` | https://www.explorer-testnet.me/en-US/transactions/53D8D15C87E49B9AAB186397D3E5145BE45092383BEC1700FED13CA54DCD6350 |
| Instantiate | `3E03BEDF04D50E411E15DC2FFF583C6B61B36861CE6E10B67098A18A80DEC0E7` | https://www.explorer-testnet.me/en-US/transactions/3E03BEDF04D50E411E15DC2FFF583C6B61B36861CE6E10B67098A18A80DEC0E7 |
| Create invoice #0 | `CF05909AD9279BC906327FA35412510CA22F1538B67B7009F5626365CAFB7BF5` | https://www.explorer-testnet.me/en-US/transactions/CF05909AD9279BC906327FA35412510CA22F1538B67B7009F5626365CAFB7BF5 |
| Fund invoice #0 | `67DADDDC4872650410157227F459EDB3167C50A6EBB4053F5E92BA318913F128` | https://www.explorer-testnet.me/en-US/transactions/67DADDDC4872650410157227F459EDB3167C50A6EBB4053F5E92BA318913F128 |
| Mark delivered | `429EE322AD669CE1FFFFBB93C1D25F4528F4050281B44714F7D004F27C75CDA3` | https://www.explorer-testnet.me/en-US/transactions/429EE322AD669CE1FFFFBB93C1D25F4528F4050281B44714F7D004F27C75CDA3 |
| Release | `0C48374A389314CD74713FDCF5CBF50D6292D48BCEEBC6E2ABB452769DD66C4E` | https://www.explorer-testnet.me/en-US/transactions/0C48374A389314CD74713FDCF5CBF50D6292D48BCEEBC6E2ABB452769DD66C4E |

Final state (queried after release): invoice #0 status **Released**,
`total_escrowed` = 0, seller balance 69,925,545 umec (20,000,000 test
funding + the full 50,000,000 escrow payout − fees) — **payout proven
on-chain**.

## License

MIT — see [`LICENSE`](LICENSE).
