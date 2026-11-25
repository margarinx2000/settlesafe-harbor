# SettleSafe — Demo UI

A zero-dependency, single-page demo for the SettleSafe escrow contract on
ME Beta Testnet. Plain HTML/CSS/JS, no build step.

## Run it

```bash
cd demo-ui
python3 -m http.server 8000
# open http://localhost:8000
```

Any static file server works (`npx serve`, nginx, GitHub Pages, …).

## What is live vs recorded

**Live (read from chain in the browser, no wallet / no signing):**

- Contract config: escrow denom + total escrowed, via
  `GET https://beta-hub-1317.explorer-testnet.me/cosmwasm/wasm/v1/contract/<contract>/smart/<base64>`
  with queries `{"total_escrowed":{}}`, `{"invoice_count":{}}`,
  `{"get_invoice":{"id":0}}`.
- Invoice #0 card: status, amount, parties, deadlines — exactly as stored
  on-chain right now.
- The header badge shows **● LIVE on-chain data** when the queries succeed.

**Recorded (fallback):** if the REST endpoint is unreachable from the
visitor's browser, the page renders the verified end-state snapshot from the
deployment run record (`../DEPLOY-RUN.md`, 25 Nov 2026) and labels it
**● RECORDED snapshot** with an explanatory banner.

**Always recorded:** the lifecycle stepper's transaction hashes (create /
fund / deliver / release for invoice #0) come from `DEPLOY-RUN.md`. Each one
links to its transaction page on the testnet explorer
(`https://www.explorer-testnet.me/en-US/transactions/<hash>` — route pattern
verified against the explorer's own app bundle) and is also copyable.

## Reproduce the flow (judges)

The “Try the flow” panel builds, for every contract execute message
(create / fund / deliver / release / dispute / resolve / cancel /
claim-after-review / refund-after-deadline):

1. the exact CosmWasm execute JSON, and
2. the equivalent `med tx wasm execute` command — the same command shape used
   in the real deployment run.

Nothing is signed or broadcast from the page; there are no keys in it. To
execute for real you need the `med` CLI and funded testnet keys (see the
contract in `../contract/` and the deployment record in `../DEPLOY-RUN.md`).
The demo keys referenced
in the commands (`settlesafe-seller`, `settlesafe-buyer`,
`settlesafe-arbiter`, keyring-backend `test`) are throwaway Beta Testnet keys
created for the deployment run.

## Facts

- Contract: `me1qwy2pa40vhvt8hetkzky3shhd7pnau4rgerk3j75fpvt8gejallqw8d7u4`
  (code ID 45, label “SettleSafe Escrow v0.1.0”, no admin)
- Chain: ME Beta Testnet, `mechain_400-1`
- Amounts: umec is the 8-decimal base unit — 50,000,000 umec = 0.5 MEC.
- Invoice #0: “Logo design package”, 50,000,000 umec, full happy path
  Create → Fund → Deliver → Release; seller payout proven on-chain
  (see `../DEPLOY-RUN.md`).
