# GNG Minting Smart Contract

## How to interact with the contract as a user

- `stake` endpoint. Call this endpoint with Gnogen NFTs as payment to stake them. Your NFTs will be engaged in all battles until you withdraw them.
- `claimRewards` endpoint. Call this endpoint to claim your rewards after your staked NFTs have participated in a battle. You can cumulate rewards from multiple battles and claim them all at once. Rewards are paid in GNG tokens.
- `withdraw` endpoint. Call this endpoint to withdraw some of your staked NFTs. The withdrawn NFTs will be returned to you and will no longer participate in battles. This endpoint also call `claimRewards` internally.
- `battle` endpoint. Call this endpoint to perform clashes when a battle has started. You can call this endpoint multiple times to perform multiple clashes. You will receive rewards for each clash you perform. This is a compensation for the gas cost of the transaction. Rewards are paid in GNG tokens.

## Smart Contract State

The contract can be under two different states: `Preparation` and `Battle`. In the `Preparation` state, users can stake their NFTs, withdraw them and claim rewards. In the `Battle` state, users can perform clashes. Each 24 hours the contract will switch from `Preparation` to `Battle` until all clashes have been performed. Then the contract will switch back to `Preparation` state.

## Clash mechanics

Each clash is a battle between two NFTs. The winner of the clash is the NFT with the highest `power` attribute. If both NFTs have the same `power` attribute, the winner is chosen with different rules. The rules are the following:

- An Emidas Genesis NFT always wins against a non-Emidas Genesis NFT.
- A Gnogon NFT always wins against a Validator NFT.
- When a Gnogon faces off against another Gnogon, the winner is the one with the highest `heart` attribute.
- When a Validator faces off against another Validator, the winner is the one with the highest `ram` attribute.

If both NFTs have the same `power` attribute and there is no rule to decide the winner, the clash is a draw and both NFTs are considered as losers.

## Rewards

### Passive rewards

When a clash is performed, the winner of the clash receives rewards. The rewards are paid in GNG tokens. The rewards are calculated as follows:

$$rewards = (NFTPower / totalWinnerPowerOfTheBattle) * totalRewardsForTheBattle$$

Where:

- `NFTPower` is the `power` attribute of the NFT that won the clash.
- `totalWinnerPowerOfTheBattle` is the sum of the `power` attributes of all NFTs that won the a clash in the battle.
- `totalRewardsForTheBattle` is the total amount of rewards that will be distributed in the battle.

A user can obviously obtain rewards from multiple clashes in the same battle. The rewards are cumulated at the user level and can be claimed with the `claimRewards` endpoint.

❗️ A user can see two of his NFTs fighting against each other.

### Battle operator rewards

When a user runs a battle transaction, he receives rewards. The rewards are paid in GNG tokens. The rewards are calculated as follows:

$$rewards = (amountOfClashes / totalAmountOfClashes) * totalRewardsForTheBattle$$

Where:

- `amountOfClashes` is the amount of clashes performed by the user in the transaction.
- `totalAmountOfClashes` is the total amount of clashes that will be performed in the battle.
- `totalRewardsForTheBattle` is the total amount of rewards that will be distributed in the battle.

## How to interact with the contract as an admin

- `depositGng` endpoint. Call this endpoint to deposit GNG tokens in the contract. The deposited GNG tokens will be used to pay rewards to users.

## How to interact with the contract as the owner

- `setBattleToken` endpoint. Call this endpoint to set the TokenIdentifier of the 5 first gnogen collections.
- `addExtraBattleToken` endpoint. Call this endpoint to add a TokenIdentifier to the list of the allowed tokens for battles. An extra token collection can be added only if it owns `power` attribute. The attributes have to be pushed in the contract before the collection is added (Cf. `setAttributes`). An extra collection cannot interact with any tiebreaker rules.
- `removeExtraBattleToken` endpoint. Call this endpoint to remove a TokenIdentifier from the list of the allowed tokens for battles. Users won't be able to stake NFTs from this collection anymore, but NFTs from this collection in the contract will still be able to participate in battles.
- `setAttributes` endpoint. Call this endpoint to set the attributes of NFTs. Attributes are pushed in the contract because some collections don't have on-chain attributes. This is a way to normalize the attributes of all collections.
- `pause` endpoint. Call this endpoint to pause the contract. All the users level endpoints will be disabled, but the time for the battles won't be paused.
- `resume` endpoint. Call this endpoint to resume the contract. All the users level endpoints will be enabled again.
- `addAdmin` endpoint. Call this endpoint to add an admin to the contract. The specified address wil be able to call the admin level endpoints.
- `removeAdmin` endpoint. Call this endpoint to remove an admin from the contract. The specified address won't be able to call the admin level endpoints anymore.
- `setBattleRewardAmount` endpoint. Call this endpoint to set the amount of rewards that will be distributed in a battle before the first halving. The amount is in GNG tokens.
- `setBattleOperatorRewardAmount` endpoint. Call this endpoint to set the amount of rewards that will be distributed to the battle operators. The amount is in GNG tokens.
