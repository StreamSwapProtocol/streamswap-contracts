# StreamSwap Protocol
![alt text](https://i.imgur.com/P7hF5uG.png)
## Abstract

StreamSwap is a protocol designed for time-based token swaps, enabling projects, DAOs, and users to create **streams** that last for a set duration and facilitate token swaps.

## **Context**

Token exchange mechanisms have been a driving force for web3 onboarding since the ICO boom. Traditional token exchange methods include:

- Automated ICOs: Team sets the issuance price, and a smart contract controls the swap.
- Regulated, centralized ICOs: Managed by a dedicated company using centralized services that comply with regulatory requirements (e.g., KYC). Example: Coinlist sales.
- Balancer-style LPBs: Employ Dutch Auction mechanism to determine a fair strike price.

StreamSwap introduces a continuous token swap mechanism, allowing dynamic price determination by the community, based on the final swap amount and time.

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

- DAO governs contract changes, fee amounts, and fee distribution.
- Governance ensures project funding for future development.

---

# **Objects**

- **name**: Name of the stream.
- **url**: An external resource describing the stream. Can be an IPFS link or a Commonwealth post.
- **treasury**: The address where the distribution earnings will go. When the stream is over, the treasury can trigger a `MsgFinalizeStream` to clean up the stream state and move the earnings to the treasury.
- **id**: The unique identifier of the stream.
- **out_denom**: The denom to distribute (distributed to the investors). Also known as the base currency.
- **in_denom**: Payment denom - used to participate with out_token. Also known as the quote currency.
- **out_supply**: Total initial supply of token_out for distribution.
- **in_supply**: Total supply of in tokens at the latest distribution.
- **spent_in**: Total number of token_in used at the latest state.
- **start_time**: Unix timestamp when the stream starts.
- **end_time**: Unix timestamp when the stream ends.
- **current_streamed_price**: Current price of the stream.
- **dist_index**: Variable to hold the latest distribution index. Used to calculate how much proportionally a position holder is entitled to receive from the stream.
- **last_updated**: Last updated time of the stream.
- **out_remaining**: Total number of remaining out tokens at the time of the update.
- **shares**: Total number of shares in the stream.

**Position**
The Position object represents a particular position in a stream. It is created when a user subscribes to a stream.

- **owner**: Owner of the position.
- **last_updated**: Last updated time of the position.
- **shares**: Number of shares of the position.
- **in_balance**: Balance of token_in currently in the position.
- **index**: Index of the position. Used to calculate incoming distribution belonging to the position.
- **distributed**: The total amount of token_out distributed to the position at the latest calculation.
- **spent**: The total amount of token_in used at the latest calculation.
- **operator**: The operator of the position. Can be used to delegate position management to another account.
- **pending_distribution**: Accumulated decimals of position.distributed on update_position.

## **Consequences**

StreamSwap democratizes token distribution and can integrate with AMM pools for automated proceeds pledging.

- Empowers the community to come together and set a fair value for an asset
- Is composable enough to integrate with various workflows after the stream (into vaults or DEXs, etc.)

## **Future Directions**

- Working on features to increase total swap volume introducing community driven time-based swaps for tokens with a price already discovered (i.e. tokens that are already trading on a DEX/CEX)
- Incentivizing distributions with OSMO or ATOM as base currency.
- Establishing a DAO for fee distribution and project maintenance.

## **Contact Us**

Reach out to us on Twitter ([x.com/StreamSwap_io](http://x.com/StreamSwap_io)) or on Telegram (https://t.me/@StreamSwap_io) — to connect with the team.