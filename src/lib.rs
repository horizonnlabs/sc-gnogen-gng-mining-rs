#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod config;
mod model;
mod operations;

use config::State;
use model::{BattleStatus, Nonce, Token, TokenStats, UserStats};
use operations::LoopOp;

const NFT_AMOUNT: u64 = 1;
const ONE_DAY_TIMESTAMP: u64 = 86400;

#[elrond_wasm::contract]
pub trait GngMinting: config::ConfigModule + operations::OngoingOperationModule {
    #[init]
    fn init(&self, first_battle_timestamp: u64, gng_token_id: TokenIdentifier) {
        self.current_battle().set_if_empty(1);
        self.first_battle_timestamp()
            .set_if_empty(first_battle_timestamp);
        self.gng_token_id().set_if_empty(gng_token_id);

        self.state().set(State::Active);
    }

    #[payable("*")]
    #[endpoint]
    fn stake(&self) {
        require!(self.is_active(), "Not active");
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );

        let payments = self.call_value().all_esdt_transfers();
        require!(!payments.is_empty(), "No payment");

        let caller = self.blockchain().get_caller();

        for payment in payments.iter() {
            let (token_id, nonce, amount) = payment.into_tuple();
            require!(self.battle_tokens().contains(&token_id), "Wrong token");
            require!(amount == NFT_AMOUNT, "Invalid token amount");

            let current_battle = self.current_battle().get();

            self.staked_for_address(&caller, &token_id).insert(nonce);

            if self.stats_for_nft(&token_id, nonce).is_empty() {
                self.stats_for_nft(&token_id, nonce).set(TokenStats {
                    win: 0,
                    loss: 0,
                    owner: caller.clone(),
                    rewards: BigUint::zero(),
                })
            } else {
                self.stats_for_nft(&token_id, nonce)
                    .update(|prev| (*prev).owner = caller.clone())
            }

            if self.first_stack(current_battle).len() > self.second_stack(current_battle).len() {
                self.second_stack(current_battle)
                    .push(&Token { token_id, nonce });
            } else {
                self.first_stack(current_battle)
                    .push(&Token { token_id, nonce });
            }
        }
    }

    #[endpoint]
    fn battle(&self) -> OperationCompletionStatus {
        require!(
            self.get_battle_status() == BattleStatus::Battle,
            "Battle in preparation"
        );

        self.rebalance_stacks();

        let current_battle = self.current_battle().get();

        let result = self.run_while_it_has_gas(|| {
            if self.first_stack(current_battle).len() == 0 {
                self.drain_stack_and_fill_next_battle(self.second_stack(current_battle));
                return LoopOp::Break;
            } else if self.second_stack(current_battle).len() == 0 {
                self.drain_stack_and_fill_next_battle(self.first_stack(current_battle));
                return LoopOp::Break;
            }

            self.single_battle();

            LoopOp::Continue
        });

        match result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {}
            OperationCompletionStatus::Completed => {
                self.current_battle().update(|current| *current += 1);
            }
        }
        result
    }

    // perhaps we can "customize" this endpoint and call it something like "claimGng" or "claimGngRewards"
    #[endpoint(claimRewards)]
    fn claim_rewards(&self, tokens: MultiValueEncoded<MultiValue2<TokenIdentifier, Nonce>>) {
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );

        let caller = self.blockchain().get_caller();

        let mut total_rewards = BigUint::zero();

        for token in tokens.into_iter() {
            let (token_id, nonce) = token.into_tuple();
            let token_stats = self.stats_for_nft(&token_id, nonce).get();

            require!(token_stats.owner == caller, "Wrong token");

            total_rewards += token_stats.rewards;

            self.stats_for_nft(&token_id, nonce)
                .update(|prev| (*prev).rewards = BigUint::zero());
        }

        self.send()
            .direct_esdt(&caller, &self.gng_token_id().get(), 0, &total_rewards);
    }

    #[endpoint]
    fn withdraw(&self, tokens: MultiValueEncoded<MultiValue2<TokenIdentifier, Nonce>>) {
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );

        let caller = self.blockchain().get_caller();

        let mut output_payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let mut total_rewards = BigUint::zero();

        for token in tokens.into_iter() {
            let (token_id, nonce) = token.into_tuple();
            let token_stats = self.stats_for_nft(&token_id, nonce).get();

            require!(token_stats.owner == caller, "Wrong token");

            total_rewards += token_stats.rewards;

            self.stats_for_nft(&token_id, nonce).update(|prev| {
                (*prev).rewards = BigUint::zero();
                (*prev).owner = ManagedAddress::zero();
            });

            output_payments.push(EsdtTokenPayment::new(
                token_id,
                nonce,
                BigUint::from(NFT_AMOUNT),
            ));
        }

        output_payments.push(EsdtTokenPayment::new(
            self.gng_token_id().get(),
            0,
            total_rewards,
        ));

        self.send().direct_multi(&caller, &output_payments);
    }

    fn single_battle(&self) {
        let current_battle = self.current_battle().get();

        let first_stack_len = self.first_stack(current_battle).len();
        let second_stack_len = self.second_stack(current_battle).len();

        let first_random_index = self.get_random_index(1, first_stack_len + 1);
        let second_random_index = self.get_random_index(1, second_stack_len + 1);

        let first_token = self.first_stack(current_battle).get(first_random_index);
        let second_token = self.second_stack(current_battle).get(second_random_index);

        let first_token_attributes = self
            .token_attributes(&first_token.token_id, first_token.nonce)
            .get();
        let second_token_attributes = self
            .token_attributes(&second_token.token_id, second_token.nonce)
            .get();

        let first_token_stats = self
            .stats_for_nft(&first_token.token_id, first_token.nonce)
            .get();
        let second_token_stats = self
            .stats_for_nft(&second_token.token_id, second_token.nonce)
            .get();

        // TODO: update winning conditions (for now its only based on power)
        if first_token_attributes.power > second_token_attributes.power {
            // update winner
            self.stats_for_nft(&first_token.token_id, first_token.nonce)
                .update(|prev| {
                    (*prev).win += 1;
                    (*prev).rewards += BigUint::from(1_000_000_000u64); // default value to change
                });
            self.stats_for_address(&first_token_stats.owner)
                .update(|prev| (*prev).win += 1);

            // update loser
            self.stats_for_nft(&second_token.token_id, second_token.nonce)
                .update(|prev| (*prev).loss += 1);
            self.stats_for_address(&second_token_stats.owner)
                .update(|prev| (*prev).loss += 1);
        } else {
            // update winner
            self.stats_for_nft(&second_token.token_id, second_token.nonce)
                .update(|prev| {
                    (*prev).win += 1;
                    (*prev).rewards += BigUint::from(1_000_000_000u64); // default value to change
                });
            self.stats_for_address(&second_token_stats.owner)
                .update(|prev| (*prev).win += 1);

            // update loser
            self.stats_for_nft(&first_token.token_id, first_token.nonce)
                .update(|prev| (*prev).loss += 1);
            self.stats_for_address(&first_token_stats.owner)
                .update(|prev| (*prev).loss += 1);
        }

        self.first_stack(current_battle + 1).push(&second_token);
        self.second_stack(current_battle + 1).push(&first_token);

        self.first_stack(current_battle)
            .swap_remove(first_random_index);
        self.second_stack(current_battle)
            .swap_remove(second_random_index);
    }

    fn rebalance_stacks(&self) {
        let current_battle = self.current_battle().get();
        let len_first_stack = self.first_stack(current_battle).len();
        let len_second_stack = self.second_stack(current_battle).len();

        if len_first_stack > len_second_stack + 1 {
            let mut diff = len_first_stack - len_second_stack;

            while diff > 1 {
                // indexes start at 1
                let last_item_index = self.first_stack(current_battle).len();
                let item_to_switch = self.first_stack(current_battle).get(last_item_index);

                self.second_stack(current_battle).push(&item_to_switch);
                self.first_stack(current_battle)
                    .swap_remove(last_item_index);

                diff -= 2;
            }
        } else if len_second_stack > len_first_stack + 1 {
            let mut diff = len_second_stack - len_first_stack;

            while diff > 1 {
                let last_item_index = self.second_stack(current_battle).len();
                let item_to_switch = self.second_stack(current_battle).get(last_item_index);

                self.first_stack(current_battle).push(&item_to_switch);
                self.second_stack(current_battle)
                    .swap_remove(last_item_index);

                diff -= 2;
            }
        } else {
            return;
        }
    }

    fn drain_stack_and_fill_next_battle(&self, mut stack_to_drain: VecMapper<Token<Self::Api>>) {
        let current_battle = self.current_battle().get();
        let len = stack_to_drain.len();
        for i in 1..len {
            let token = stack_to_drain.get(i);
            if self.first_stack(current_battle + 1).len()
                > self.second_stack(current_battle + 1).len()
            {
                self.second_stack(current_battle + 1).push(&token);
            } else {
                self.first_stack(current_battle + 1).push(&token);
            }
        }
        stack_to_drain.clear();
    }

    fn get_random_index(&self, min: usize, max: usize) -> usize {
        let mut rand = RandomnessSource::<Self::Api>::new();
        rand.next_usize_in_range(min, max)
    }

    #[view(getBattleStatus)]
    fn get_battle_status(&self) -> BattleStatus {
        let current_timestamp = self.blockchain().get_block_timestamp();
        let first_battle_timestamp = self.first_battle_timestamp().get();
        let current_battle = self.current_battle().get();

        if current_timestamp >= first_battle_timestamp + (ONE_DAY_TIMESTAMP * (current_battle - 1))
        {
            return BattleStatus::Battle;
        }
        BattleStatus::Preparation
    }

    #[view(getCurrentBattle)]
    #[storage_mapper("currentBattle")]
    fn current_battle(&self) -> SingleValueMapper<u64>;

    #[view(getStatsForAddress)]
    #[storage_mapper("statsForAddress")]
    fn stats_for_address(&self, address: &ManagedAddress) -> SingleValueMapper<UserStats>;

    #[view(getStatsForNft)]
    #[storage_mapper("statsForNft")]
    fn stats_for_nft(
        &self,
        token_id: &TokenIdentifier,
        nonce: Nonce,
    ) -> SingleValueMapper<TokenStats<Self::Api>>;

    #[view(getCurrentRewardsForAddress)]
    #[storage_mapper("currentRewardsForAddress")]
    fn current_rewards_for_address(&self) -> SingleValueMapper<BigUint>;

    #[view(getStakedForAddress)]
    #[storage_mapper("stakedForAddress")]
    fn staked_for_address(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> UnorderedSetMapper<Nonce>;

    #[view(getFirstStack)]
    #[storage_mapper("firstStack")]
    fn first_stack(&self, battle: u64) -> VecMapper<Token<Self::Api>>;

    #[view(getSecondStack)]
    #[storage_mapper("secondStack")]
    fn second_stack(&self, battle: u64) -> VecMapper<Token<Self::Api>>;

    #[view(getFirstBattleTimestamp)]
    #[storage_mapper("firstBattleTimestamp")]
    fn first_battle_timestamp(&self) -> SingleValueMapper<u64>;
}
