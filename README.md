# FlowEscrow

Trustless, milestone-based escrow for freelance payments on Stellar.

## Problem

A freelance designer in Lagos delivers work to a client in Berlin, then waits
7–10 days for a wire transfer or loses 8–12% of the payment to PayPal/Wise
fees and currency spread — with no recourse if the client disappears after
seeing the draft.

## Solution

The client locks USDC into a Soroban escrow contract when the job starts.
The freelancer marks the milestone complete on-chain, and the client
approves release — settling in seconds for fractions of a cent. Stellar's
near-zero fees and fast finality make small, frequent freelance payments
practical in a way high-fee rails never could, while trustlines keep every
balance transparent and auditable for both sides.

## Timeline

| Day | Milestone |
|-----|-----------|
| 1   | Contract design, storage schema, `create_escrow` + `get_escrow` |
| 2   | `mark_complete`, `release_funds`, `refund`, full test suite |
| 3   | Frontend wallet integration (Freighter), demo wiring |
| 4   | Polish, deploy to testnet, record demo, anchor cash-out stretch goal |

## Stellar Features Used

- **USDC transfers / Trustlines** — the asset that moves through escrow
- **Soroban smart contracts** — encode the escrow + milestone state machine
  so neither party can unilaterally take the funds

## Vision and Purpose

Freelance platforms currently take 10–20% in fees and add days of payout
delay. FlowEscrow removes the middleman's cut and the wait: funds are
provably locked the moment a job starts, and release is a single on-chain
approval. The long-term vision is a payment layer any freelance platform,
DAO, or two strangers on the internet can plug into for trust-minimized
work agreements — with local anchor integration eventually letting
freelancers cash out straight to mobile money or a bank account.

## Prerequisites

- Rust (stable, 1.74+) with the `wasm32-unknown-unknown` target
- Soroban CLI v21+ (`cargo install --locked soroban-cli`)
- A funded Stellar testnet account (via [Friendbot](https://friendbot.stellar.org))

## How to Build

```bash
soroban contract build
```

The compiled Wasm will be at:
`target/wasm32-unknown-unknown/release/flow_escrow.wasm`

## How to Test

```bash
cargo test
```

Runs the 5-test suite covering the happy path, an unauthorized-caller
failure, and state verification after each major transition.

## How to Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/flow_escrow.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

This returns a contract ID, e.g. `CABCD...XYZ`.

## Sample CLI Invocation (MVP function, dummy arguments)

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- \
  create_escrow \
  --client GCLIENT...EXAMPLE \
  --freelancer GFREELANCER...EXAMPLE \
  --token CUSDC...TOKENCONTRACT \
  --amount 500
```

Follow-up calls use the returned `escrow_id`:

```bash
soroban contract invoke --id <CONTRACT_ID> --source <FREELANCER_KEY> --network testnet \
  -- mark_complete --escrow_id 1 --freelancer GFREELANCER...EXAMPLE

soroban contract invoke --id <CONTRACT_ID> --source <CLIENT_KEY> --network testnet \
  -- release_funds --escrow_id 1 --client GCLIENT...EXAMPLE
```

## License

MIT
## Deployed Contract

| Field | Value |
|-------|-------|
| Contract ID | `CDYERS3DMFVUBDH2C6NVS5DFPHX5XZXHTYTN3AKBBL7IB7U6KV4FYTS7` |
| Network | testnet |
| Explorer | [View on stellar.expert](https://stellar.expert/explorer/testnet/contract/CDYERS3DMFVUBDH2C6NVS5DFPHX5XZXHTYTN3AKBBL7IB7U6KV4FYTS7) |
| Deploy Tx | [View transaction](https://stellar.expert/explorer/testnet/tx/d81088f891aadc2b76f6a5c3f7e578d5d803a7ad1dc93b01962e8ef979d0c9f1) |
| Deployed | 2026-06-26 07:07:50 UTC |
| Wallet | freighter (`GAQ5…H2SY`) |
