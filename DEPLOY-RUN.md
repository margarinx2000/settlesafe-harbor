# SettleSafe — ME Beta Testnet Deploy Run Record

Run executed 25 Nov 2026, ~23:48–23:52 WIB (16:48–16:52 UTC), by Syepi under
Mantri's standing orders ("Gas harbor" / "Gas terus" / "jalankan tugasmu yang lain").
Chain: **mechain_400-1** (ME Beta Testnet), node via local relay
`tcp://127.0.0.1:26657` → Hub RPC. `med` version 1e9eff8.
Block height at start: 5,512,778 · at end: 5,512,826.

## Funding

- Faucet claim (submitted 25 Nov 22:44 WIB) **landed before its 2-hour window**:
  deployer balance found = **200,000,000 umec**.
  - Note: the faucet page promises "2 test MEC per claim"; 200,000,000 umec
    received implies the denom behaves as 8-decimal (1 MEC = 10^8 umec), or the
    faucet sent more than the page figure. All amounts below are raw umec.
- Deployer: `settlesafe-deployer` = me1l9vj4tjknyzd3462uru3g8srep4zpczaahe6tm

## Keys created this run (keyring-backend test, add-outputs sealed mode 600 in keys/)

| Role | Key name | Address |
|---|---|---|
| Seller | settlesafe-seller | me1hu3s8gzqa0cmn09sha3svq9t8r5s598npar50z |
| Buyer | settlesafe-buyer | me13ceqzt2ly743q5p7jyrwl09cq2wnkwnkllzf2w |
| Arbiter | settlesafe-arbiter | me1w0jt2c45kkgp2u5eys0szt979lkcahkeayvw6y |

Mantri's and Wahyuni's ME ID wallets were NOT touched.

## Fee calibration (before the store broadcast)

- `med tx … --dry-run` is broken in this build: it fails with
  `Error: .info: key not found` for both key-name and address `--from` forms.
  Estimation was therefore done via broadcast-mode `--gas auto` simulation,
  which works.
- Calibration bank send (deployer → buyer, 70,000,000 umec):
  gas estimate 291,198 · fee paid **29,120 umec** at `--gas-prices 0.1umec`.
  → The guide's 100 MEC example fee was pure over-provisioning; real fees are
  ~0.03–1.2 MEC-equivalent in raw umec terms (29,120–1,166,934 umec per tx).
- Funding sends:
  - → buyer 70,000,000 umec · tx `2BF63257C2819252A5D95EAA7141F01AAB9E347E2564D98FE354348E9AE3DD13` · fee 29,120 umec
  - → seller 20,000,000 umec · tx `EEE35DBE2F8DADCEE79E90DBA0B9FBD2B2A5B4553753AF0821EF23D14706A73F`

## Store + instantiate

- **Store** artifact `build/artifacts/settlesafe_cw.wasm` (Rust 1.80.1, 260,393 B)
  - tx `53D8D15C87E49B9AAB186397D3E5145BE45092383BEC1700FED13CA54DCD6350`
  - height 5,512,803 · gas_used 8,903,900 · fee paid 1,166,934 umec
  - **CODE_ID = 45**
- **Instantiate** `{"escrow_denom":"umec"}`, label "SettleSafe Escrow v0.1.0", --no-admin
  - tx `3E03BEDF04D50E411E15DC2FFF583C6B61B36861CE6E10B67098A18A80DEC0E7`
  - height 5,512,808 · gas_used 218,319 · fee paid 37,838 umec
  - **CONTRACT = me1qwy2pa40vhvt8hetkzky3shhd7pnau4rgerk3j75fpvt8gejallqw8d7u4**

## Lifecycle run — invoice #0 (full happy path)

Invoice: seller → buyer, arbiter as above, amount **50,000,000 umec**,
description "Logo design package", pay_window 86400, deliver_window 172800,
review_window 259200. (DEPLOY-READY's 250,000,000 example amount exceeded the
faucet-funded balance, so the run amount was scaled to 50,000,000 umec.)

| Step | From | Tx hash | Verified state after |
|---|---|---|---|
| create_invoice | seller | `CF05909AD9279BC906327FA35412510CA22F1538B67B7009F5626365CAFB7BF5` | status Created, invoice_count = 1 |
| fund_invoice (+50,000,000 umec) | buyer | `67DADDDC4872650410157227F459EDB3167C50A6EBB4053F5E92BA318913F128` | status Funded, total_escrowed = 50,000,000 |
| mark_delivered | seller | `429EE322AD669CE1FFFFBB93C1D25F4528F4050281B44714F7D004F27C75CDA3` | status Delivered, review_deadline set |
| release | buyer | `0C48374A389314CD74713FDCF5CBF50D6292D48BCEEBC6E2ABB452769DD66C4E` | status **Released**, total_escrowed = 0 |

## Final verification (queries after release)

- `get_invoice {"id":0}` → status **Released**, amount "50000000",
  seller/buyer/arbiter as above, all deadlines recorded on-chain.
- `total_escrowed {}` → 0 umec.
- Seller balance: **69,925,545 umec** = 20,000,000 funded + 50,000,000 escrow
  payout − 74,455 in create/deliver fees → **payout proven on-chain**.
- Buyer balance: 19,924,252 umec (70,000,000 − 50,000,000 escrow − 75,748 fees).
- Deployer balance after all steps: 108,798,376 umec.

## Remaining for the HARBOR submission package (deadline 21 Dec 22:59 WIB)

- Demo UI hookup to contract me1qwy2pa40vhvt8hetkzky3shhd7pnau4rgerk3j75fpvt8gejallqw8d7u4 (code 45)
- 3–5 min English demo video
- Optional: a second invoice exercising the dispute/refund path for richer run records — **DONE 26 Nov 2026, see next section**

## Invoice #1 — dispute path (refund outcome)

Run executed 26 Nov 2026, ~06:20–06:35 WIB (25 Nov 23:20–23:35 UTC), authorized
by Mantri 06:18 WIB ("Poin 1 dan 2 gak bisa kamu kerjakan?" — do points 1 and 2
yourself). Same contract, same demo keys as invoice #0; Mantri's and Wahyuni's
ME ID wallets were NOT touched. Chain mechain_400-1 via the local relay;
block height 5,517,549 at start, 5,517,579 at the final resolve.

Invoice: seller → buyer, arbiter as above, amount **25,000,000 umec**,
description "Dispute-path demo — packaging design", pay_window 86400,
deliver_window 172800, review_window 259200. (Per the contract schema the
seller is the create sender — `seller: info.sender` — so the seller created
invoice #1, exactly as for invoice #0.)

### Pre-funding (deployer top-ups, before the lifecycle)

Starting balances: buyer 19,924,252 · seller 69,925,545 · arbiter 0 ·
deployer 108,798,376 umec (unchanged since the invoice #0 run).

| Send | Tx hash | Height | Verified state after |
|---|---|---|---|
| deployer → buyer 20,000,000 umec | `4B33A51CFACF520D86C18BA71F8508113DE17C9B54B9F58FD7FCDDCC2EC1F4CC` | 5,517,557 | buyer = 39,924,252 |
| deployer → arbiter 5,000,000 umec | `61019B9E0B823FD051D112338D99CA026C48C0B194DF21D56C62C3848B74DDA4` | 5,517,561 | arbiter = 5,000,000 (arbiter had 0 and needs gas for the resolve tx) |

### Lifecycle run — invoice #1 (dispute → refund)

| Step | From | Tx hash | Height | Gas used | Verified state after |
|---|---|---|---|---|---|
| create_invoice | seller | `8D8D664018930D036066C2F73906D103D66887855C074E6B091801ECE82CFCF1` | 5,517,567 | 230,690 | status **Created**, invoice_count = 2 |
| fund_invoice (+25,000,000 umec) | buyer | `B3C894CF342A329E4C026846870B2155E517AC80B859F58A60F1B18D0458A561` | 5,517,571 | 225,366 | status **Funded**, total_escrowed = 25,000,000 |
| mark_delivered | seller | `B9BC453936E08228A64462C134FF6682CFF625B2B8AF5CD58532BDA45637CDDA` | 5,517,574 | 204,487 | status **Delivered**, review_deadline set |
| raise_dispute | buyer | `EC37B1A69F470379C43B82BA6B7B60C9945E7E1B28DB71A1180D4B934F290CB8` | 5,517,576 | 204,540 | status **Disputed** |
| resolve_dispute (release_to_seller = false) | arbiter | `F0D17DF67387730FA4858ECF355AC755D28A9FAD48CD3F0D17DC9DE8312D53D5` | 5,517,579 | 231,398 | status **Refunded**, total_escrowed = 0 |

Status transitions: Created → Funded → Delivered → Disputed → **Refunded**
(the dispute branch invoice #0 never touched; the full 25,000,000 umec went
back to the buyer, nothing to the seller).

### Final verification (queries after resolve)

- `get_invoice {"id":1}` → status **Refunded**, amount "25000000",
  seller/buyer/arbiter as above, description as above.
- `total_escrowed {}` → 0 umec; contract bank balance = 0 umec.
- Buyer balance: **39,868,189 umec** = 39,924,252 (after top-up) − 25,000,000
  escrow − 56,063 fund+dispute fees + 25,000,000 refund → **refund proven
  on-chain to the exact umec**.
- Seller balance: 69,868,963 umec = 69,925,545 − 56,582 create+deliver fees
  (no payout — correct for the refund outcome).
- Arbiter balance: 4,970,319 umec = 5,000,000 − 29,681 resolve fee.
- Deployer balance after all steps: 83,814,021 umec (not a lifecycle party;
  spent 25,000,000 on the two top-ups plus their send fees).
