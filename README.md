# StreamSwap Smart Contracts

[![GitHub tag (with filter)](https://img.shields.io/github/v/tag/StreamSwapProtocol/streamswap-contracts?label=Latest%20version&logo=github)](https://github.com/StreamSwapProtocol/streamswap-contracts/releases/latest)
![Codecov](https://img.shields.io/codecov/c/github/StreamSwapProtocol/streamswap-contracts)
![GitHub](https://img.shields.io/github/license/StreamSwapProtocol/streamswap-contracts)
![X (formerly Twitter) Follow](https://img.shields.io/twitter/follow/StreamSwap_io)

![alt text](https://i.imgur.com/P7hF5uG.png)

## Overview

[StreamSwap](https://www.streamswap.io) is a protocol designed for time-based token swaps, enabling projects, DAOs, and users to create streams that last for a set duration and facilitate token swaps.

## Context

Token exchange mechanisms have been a driving force for web3 onboarding since the ICO boom. Traditional token exchange methods include:

- Automated ICOs: Team sets the issuance price, and a smart contract controls the swap.
- Regulated, centralized ICOs: Managed by a dedicated company using centralized services that comply with regulatory requirements (e.g., KYC). Example: Coinlist sales.
- Balancer-style LPBs: Employ Dutch Auction mechanism to determine a fair strike price.

StreamSwap introduces a continuous token swap mechanism, allowing dynamic price determination by the community, based on the final swap amount and time.

## Architecture

StreamSwap's architecture is designed to support the continuous and fair execution of token distribution. The main components include:

- **Controller Contract**: Manages the creation of new streams and handles protocol-level operations.
- **Stream Contract**: Implements the logic for managing individual token streams, including subscription, distribution, and finalization.

![alt text](https://gist.github.com/user-attachments/assets/fa8e3b0b-6b31-48c4-86da-b0f1bae5bc45)

### Smart Contracts

Here’s a breakdown of the main contracts in the StreamSwap system:

| Contract Name                      | Description                                                                                                    |
|------------------------------------|----------------------------------------------------------------------------------------------------------------|
| [controller](contracts/controller) | Handles the creation of new streams and manages protocol-wide functions.                                       |
| [stream](contracts/stream)         | Manages individual streams, including user subscriptions and token distribution, pool creation, vesting and more |

### Stream Lifecycle

The lifecycle of a StreamSwap stream is divided into several states:

1. **Waiting**: The stream has been created but is not yet active. No interactions are allowed.
2. **Bootstrapping**: Participants can subscribe to the stream, but distribution has not yet started. No assets can be spent.
3. **Active**: The stream is live, and tokens are distributed according to the subscription amounts and timing.
4. **Ended**: The stream has concluded. Participants can exit the stream, and the creator can finalize and collect the proceeds.
5. **Finalized**: The stream is finalized. The creator can withdraw any remaining tokens, and vesting or pools (if configured) are set up.
6. **Cancelled**: The stream is cancelled. Participants can withdraw their assets, and the creator can reclaim any unsold tokens.

## **Design**

### **Stream Creation**

- Create a stream by submitting a `CreateStream` transaction.
- Treasury owner sends creation fee tokens and `out_denom` tokens to the contract.
- Fees are collected and managed through governance voting.

### **Subscription**

- Join a stream by submitting a `SubscribeMsg` transaction.
- Transaction funds are pledged, minting new shares.
- Shares are calculated based on the subscription amount.

### **Distribution**

- Distribution is based on total shares and time.
- `update_stream` calculates the amount to be distributed to investors.

### **Spending**

- Spend calculations occur during `update_position`.
- Updates to the stream and position state are done continuously.

### **Withdraw**

- Withdraw unspent tokens via `WithdrawMsg`.
- Shares are reduced proportionally to the withdrawn amount.

### **Exit Stream**

- After the stream ends, participants can withdraw distributed tokens and claim unspent tokens via `ExitMsg`.

### **Finalize Stream**

- Treasury can finalize the stream to collect tokens post-distribution, applying an exit fee.

### **Price**

- Average price: `stream.spent_in / (stream.out_supply - stream.out_remaining)`.
- Last streamed price calculated during the latest `update_stream`.

### **Creation Fee**

- Collected to prevent spam, managed by the fee collector.

## **DAO Governance**

- DAO governs contract changes, fee amounts, fee distribution and emergency interventions.
- Governance ensures project funding for future development.

## Getting Started

### Prerequisites

Ensure you have Rust installed with the `wasm32-unknown-unknown` target. You can install it using:

```bash
rustup target add wasm32-unknown-unknown
```

### Building

Navigate to the workspace directory and compile the contracts:

```bash
cargo build
```

### Testing

To ensure everything is working correctly, you can run the tests from the workspace root:

```bash
cargo test
```

### Production Build

For a production-ready build, run the following command to generate optimized contracts:

```bash
cargo run-script optimize
```

The optimized contracts will be placed in the artifacts/ directory.

### Typescript Codegen

To generate the typescript bindings for the contracts, run the following command:

```bash
scripts/schema.sh
```

## Deployed Contract Addresses

The addresses of the deployed contracts will be listed here once available.

**TODO**: Add deployed contract addresses

## Audit

The security audit of the StreamSwap contracts will be published here.

**TODO**: Add audit report

## License

This repo is licensed under [Apache 2.0](LICENSE).
