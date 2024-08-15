# StreamSwap Smart Contracts

[![GitHub tag (with filter)](https://img.shields.io/github/v/tag/StreamSwapProtocol/streamswap-contracts?label=Latest%20version&logo=github)](https://github.com/StreamSwapProtocol/streamswap-contracts/releases/latest)
![Codecov](https://img.shields.io/codecov/c/github/StreamSwapProtocol/streamswap-contracts)
![GitHub](https://img.shields.io/github/license/StreamSwapProtocol/streamswap-contracts)
![X (formerly Twitter) Follow](https://img.shields.io/twitter/follow/StreamSwap_io)

![alt text](https://i.imgur.com/P7hF5uG.png)

## Overview
[StreamSwap](https://www.streamswap.io/) introduces a novel approach to token sales, allowing for a continuous and dynamic sale process. This smart
contract platform enables users to create and participate in token sales that unfold over time, providing a more
equitable distribution mechanism compared to traditional methods.

## Why?

Since the first ICO boom, token sale mechanisms have been one of the driving
forces for web3 onboarding.
The promise of cheap tokens that can quickly accrue value is very attractive
to all types of investors. The easy way of fundraising opened doors for cohorts
of new teams to focus on building.

Traditional mechanisms of token sales include:

- Automated ICO, where the team decides the issuance price, and the sale
  happens through a swap controlled by a smart contract.
- Regulated, centralized ICO - token sales controlled by a dedicated company,
  which performs all operations using centralized services meeting
  regulatory requirements (KYC, etc.). Example: Coinlist sales.
- Balancer-style ICO: a novel solution that utilizes the Dutch Auction mechanism to
  find a fair strike price.

The first two mechanisms are not well suited for early-stage startups, where
the token sale price is usually defined by the founding team and can't be
impacted by the ecosystem's wisdom. False marketing actions are usually set up
to support the initial price.

The latter mechanism is not democratic - large entities can control the
price movements by placing big orders, leaving smaller investors with nothing.

StreamSwap is a new mechanism that allows anyone to create a new sale event where the sale happens continuously over a
period of time.
You can imagine it as two flasks of liquid, mixing together over time, reaching equilibrium.

The price is determined on the fly as the sell/buy balance fluctuates with the buy supply increasing through incoming
subscriptions.
The sell side is distributed among the subscribers, in proportion to their subscription amount relative to the total
subscription amount.
The buy side is spent according to the remaining time until the end date of the sale event.
Example: When the stream ends, the tokens of a buyer who subscribed at 80 percent of the stream will be fully spent,
just like the tokens of a buyer who subscribed at the beginning of the stream.

## Architecture

StreamSwap's architecture is designed to support the continuous and fair execution of token sales. The main components include:

- **Controller Contract**: Manages the creation of new streams (sales) and handles protocol-level operations.
- **Stream Contract**: Implements the logic for managing individual token sales, including subscription, distribution, and finalization.

![alt text](https://gist.github.com/user-attachments/assets/fa8e3b0b-6b31-48c4-86da-b0f1bae5bc45)


## Stream Lifecycle

The lifecycle of a StreamSwap sale is divided into several states:

1. **Waiting**: The stream has been created but is not yet active. No interactions are allowed.
2. **Bootstrapping**: Participants can subscribe to the stream, but distribution has not yet started. No assets can be spent.
3. **Active**: The stream is live, and tokens are distributed according to the subscription amounts and timing.
4. **Ended**: The sale has concluded. Participants can exit the stream, and the creator can finalize and collect the proceeds.
5. **Finalized**: The stream is finalized. The creator can withdraw any remaining tokens, and vesting or pools (if configured) are set up.
6. **Cancelled**: The stream is cancelled. Participants can withdraw their assets, and the creator can reclaim any unsold tokens.

## Smart Contracts

Here’s a breakdown of the main contracts in the StreamSwap system:

| Contract Name                   | Description                                                                                                    |
|---------------------------------|----------------------------------------------------------------------------------------------------------------|
| [controller](contracts/factory) | Handles the creation of new streams and manages protocol-wide functions.                                       |
| [stream](contracts/stream)      | Manages individual sales, including user subscriptions and token distribution, pool creation, vesting and more |

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

## Deployed Contract Addresses

The addresses of the deployed contracts will be listed here once available.

**TODO**: Add deployed contract addresses

## Audit

The security audit of the StreamSwap contracts will be published here.

**TODO**: Add audit report

## License

This repo is licensed under [Apache 2.0](LICENSE).
