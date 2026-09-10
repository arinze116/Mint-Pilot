# Mint-Pilot

Mint-Pilot is a Rust CLI for scheduling OpenSea mint transactions. It supports a single eligible wallet, a small self-funded wallet manifest, and an experimental EIP-7702 sponsored multi-wallet workflow.

Commands that mint, fund, withdraw, deploy, or revoke can send real blockchain transactions. Use only wallets and funds you control.

## Features

- Resolves OpenSea collection metadata, stages, and wallet eligibility from a slug, collection URL, or NFT contract address.
- Authenticates wallets, locally validates OpenSea mint actions, schedules selected phases, and handles EIP-1559 replacement/receipt polling.
- Creates strict versioned wallet manifests and offers read-only calldata inspection.
- Supports one wallet, up to 10 self-funded manifest wallets, or up to 25 sponsored manifest wallets.
- Funds and withdraws manifest wallets, deploys a deterministic sponsored executor, and revokes EIP-7702 delegations.
- Includes optional Telegram notices for multi-wallet work.

## Install

Rust `1.97.1` is required; `rust-toolchain.toml` pins that version with `clippy` and `rustfmt`. You also need an HTTPS JSON-RPC endpoint (HTTP is accepted only on loopback test endpoints).

```sh
git clone https://github.com/arinze116/Mint-Pilot.git
cd Mint-Pilot/pilot
cp .env.example .env
cargo build --release --locked
./target/release/opensea-mint --help
```

## Configuration

The CLI searches upward from the executable/current directory for `.env`. Copy `.env.example`, use placeholders only in templates, and keep the real file private.

### Single wallet

Set `WALLET_KEY`, `RPC_URL`, `FEE_AUTOMATIC`, and `GAS_LIMIT`. `WALLET_KEY` is a 32-byte, `0x`-prefixed private key. Do not combine this with multi-wallet settings.

### Multi-wallet

Generate a new private manifest without connecting to an RPC or OpenSea:

```sh
opensea-mint wallets create --count 2 --quantity 1 --output wallets.json
```

Set `WALLETS_FILE=wallets.json`, `SPONSORED=false`, and `RECIPIENT_ADDRESS`. The strict manifest format is:

```json
{"version":1,"wallets":[{"private_key":"0x<private-key>","quantity":1}]}
```

Manifest files must be regular files no larger than 1 MiB, with no unknown fields, duplicate derived addresses, or zero quantities. `wallets.json` is Git-ignored. Self-funded mode permits at most 10 wallets. `SPONSOR_KEY` is needed for `mint --fund` and supplies the default recipient when `RECIPIENT_ADDRESS` is absent.

### Sponsored minting

Sponsored mode is experimental. Set `WALLETS_FILE`, `SPONSORED=true`, `SPONSOR_KEY`, and `SPONSORED_EXECUTOR_ADDRESS`; `RECIPIENT_ADDRESS` and `SPONSORED_OPERATION_DEADLINE_SECONDS` are optional. The sponsor pays transaction gas, but each wallet still needs its mint value and OpenSea action reserve. The recipient cannot be the executor.

Deploy after reviewing the interactive plan:

```sh
opensea-mint deploy-executor
opensea-mint mint --undelegate
```

The deployment checks the deterministic factory, EIP-7702/EIP-1153 liveness, funding, and runtime hash. Use the printed address as `SPONSORED_EXECUTOR_ADDRESS`. Sponsored mode allows at most 25 wallets. See [contracts/README.md](contracts/README.md) for executor details and security boundaries.

### OpenSea and Telegram

OpenSea URLs and application identity are fixed in the program; normal minting does not need an API key. Telegram notifications activate only when both `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` are exported in the process environment. `OPENSEA_API_KEY` is likewise read from the process environment for optional Telegram floor-price and collection-name lookups.

### Settings

| Setting | Meaning, default, or range |
| --- | --- |
| `WALLET_KEY` | Required single-wallet private key. |
| `WALLETS_FILE` | Multi-wallet manifest path. |
| `SPONSORED` | Required with a manifest: `true` or `false`. |
| `RECIPIENT_ADDRESS` | NFT recipient; required without a sponsor fallback. |
| `SPONSOR_KEY` | Sponsor private key for sponsored mode, funding, undelegation, and deployment. |
| `SPONSORED_EXECUTOR_ADDRESS` | Required when sponsored. |
| `SPONSORED_OPERATION_DEADLINE_SECONDS` | Default `120`; `30-3600`. |
| `RPC_URL` | Required HTTPS RPC URL (or loopback HTTP for tests). |
| `RPC_URL_2`, `RPC_URL_3`, `RPC_URL_4`, `RPC_URL_5` | Optional ordered fallback RPC endpoints. |
| `FEE_AUTOMATIC` | Required boolean; use `false` for manual fees. |
| `MAX_FEE_PER_GAS_GWEI` | Required manual max fee, decimal gwei up to 9 decimals. |
| `MAX_PRIORITY_FEE_PER_GAS_GWEI` | Required manual priority fee; cannot exceed max fee. |
| `GAS_LIMIT` | Required positive mint gas limit. |
| `SCHEDULE_REFRESH_INTERVAL_SECONDS` | Default `600`; `10-86400`. |
| `TRANSACTION_MAX_ATTEMPTS` | Default `3`; `1-10`. |
| `PENDING_TIMEOUT_SECONDS` | Default `20`; `1-86400`. |
| `RECEIPT_POLL_BASE_DELAY_MS` | Default `250`; `50-60000`. |
| `RECEIPT_POLL_MAX_DELAY_MS` | Default `2000`; `50-60000`; no lower than base delay. |
| `REPLACEMENT_BUMP_BPS` | Default `11250`; `10001-20000`. |
| `LAUNCH_PRIORITY_FEE_MULTIPLIER_BPS` | Default `10000`; `10000-50000`. |
| `OPENSEA_REQUEST_TIMEOUT_MS` | Default `10000`; `100-120000`. |
| `ELIGIBILITY_REQUEST_TIMEOUT_MS` | Default `5000`; `100-120000`. |
| `OPENSEA_MAX_ATTEMPTS` | Default `3`; `1-10`. |
| `OPENSEA_RETRY_INTERVAL_MS` | Default `250`; `50-30000`. |
| `OPENSEA_CALLDATA_MAX_ATTEMPTS` | Default `40`; `1-1000`. |
| `TELEGRAM_BOT_TOKEN` | Optional exported Telegram bot token. |
| `TELEGRAM_CHAT_ID` | Optional exported Telegram chat ID. |
| `OPENSEA_API_KEY` | Optional exported API key for Telegram enrichment. |

## Commands

```text
opensea-mint doctor
opensea-mint deploy-executor
opensea-mint mint [--fund NATIVE_AMOUNT | --withdraw | --undelegate]
opensea-mint calldata --collection COLLECTION --wallets WALLETS [--token-id TOKEN_ID]
opensea-mint wallets create [--count COUNT] [--quantity QUANTITY] [--output PATH]
```

`doctor` validates configuration, manifest mode, RPC, and the OpenSea client boundary. `calldata` authenticates manifest wallets and fetches/validates mint calldata without signing or broadcasting. Plain `mint` is interactive and prompts for collection/phase choices. `mint --fund`, `mint --withdraw`, `mint --undelegate`, and `deploy-executor` can broadcast real transactions.

```sh
opensea-mint doctor
opensea-mint calldata --collection https://opensea.io/collection/example --wallets wallets.json
opensea-mint mint --fund 0.01
```

## Testing

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
```

Tests cover CLI parsing, configuration/signing, wallet manifests, OpenSea response validation, transaction and fee logic, and chain-gateway parsing. They use fixtures and do not need real wallets or transactions.

If Foundry is installed, contract tests are configured by `foundry.toml`:

```sh
forge fmt --check
forge test --match-path 'contracts/test/*.t.sol' -vv
```

## Troubleshooting and security

Run `opensea-mint doctor` first. A rejected configuration commonly means mixed wallet modes, a non-HTTPS non-loopback RPC URL, missing manual fee values, or unsupported settings. Sponsored mode fails closed when required chain capabilities or the executor hash do not match. Inspect every reported transaction hash before retrying an uncertain broadcast or receipt.

Never commit `.env`, `wallets.json`, private keys, seed phrases, bot tokens, RPC credentials, or API keys. Generated manifests contain private keys and receive private Unix permissions. The sponsored executor is experimental and has not been independently audited; test with disposable wallets first.

## Layout

```text
src/        CLI, configuration, RPC, OpenSea, signing, and mint workflows
tests/      Rust integration tests
contracts/  Sponsored executor and Foundry tests
assets/     Repository assets
```

MIT licensed; see [LICENSE](LICENSE).
