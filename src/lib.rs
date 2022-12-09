#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod config;
mod model;
mod operations;

use core::cmp::Ordering;

use config::State;
use model::{
    Attributes, BattleHistory, BattleStatus, Nonce, PendingRewards, Token, TokenStats, UserStats,
};
use operations::LoopOp;

const NFT_AMOUNT: u64 = 1;
const ONE_DAY_TIMESTAMP: u64 = 86400;
const DIVISION_PRECISION: u64 = 1000000;

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
        self.stats_for_address(&caller)
            .set_if_empty(UserStats::default());

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
                })
            } else {
                self.stats_for_nft(&token_id, nonce)
                    .update(|prev| prev.owner = caller.clone())
            }

            if self.first_stack(current_battle).len() > self.second_stack(current_battle).len() {
                self.second_stack(current_battle)
                    .push(&Token { token_id, nonce });
            } else {
                self.first_stack(current_battle)
                    .push(&Token { token_id, nonce });
            }
        }

        self.total_nft_engaged()
            .update(|prev| *prev += payments.len() as u64);
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
            if self.first_stack(current_battle).is_empty() {
                self.drain_stack_and_fill_next_battle(self.second_stack(current_battle));
                return LoopOp::Break;
            } else if self.second_stack(current_battle).is_empty() {
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

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );

        let caller = self.blockchain().get_caller();

        let total_rewards = self.get_pending_rewards_for_address(&caller);

        if total_rewards > 0 {
            self.send()
                .direct_esdt(&caller, &self.gng_token_id().get(), 0, &total_rewards);

            self.raw_pending_rewards_for_address(&caller).clear();
        }
    }

    #[endpoint]
    fn withdraw(&self, tokens: MultiValueEncoded<MultiValue2<TokenIdentifier, Nonce>>) {
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );

        self.claim_rewards();

        let caller = self.blockchain().get_caller();
        let amount_tokens = tokens.len();

        let mut output_payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();

        for token in tokens.into_iter() {
            let (token_id, nonce) = token.into_tuple();
            let token_stats = self.stats_for_nft(&token_id, nonce).get();

            require!(
                token_stats.owner == caller
                    && self.staked_for_address(&caller, &token_id).contains(&nonce),
                "Wrong token"
            );

            self.stats_for_nft(&token_id, nonce).update(|prev| {
                prev.owner = ManagedAddress::zero();
            });
            self.staked_for_address(&caller, &token_id)
                .swap_remove(&nonce);

            output_payments.push(EsdtTokenPayment::new(
                token_id,
                nonce,
                BigUint::from(NFT_AMOUNT),
            ));
        }

        self.total_nft_engaged()
            .update(|prev| *prev -= amount_tokens as u64);

        self.send().direct_multi(&caller, &output_payments);
    }

    fn single_battle(&self) {
        let current_battle = self.current_battle().get();
        let mut first_stack_mapper = self.first_stack(current_battle);
        let mut second_stack_mapper = self.second_stack(current_battle);

        let first_stack_len = first_stack_mapper.len();
        let second_stack_len = second_stack_mapper.len();

        let first_random_index = self.get_random_index(1, first_stack_len + 1);
        let second_random_index = self.get_random_index(1, second_stack_len + 1);

        let first_token = first_stack_mapper.get(first_random_index);
        let second_token = second_stack_mapper.get(second_random_index);

        // if one of the tokens has owner 0, it means the owner has withdrawn it during the preparation
        // we remove it from the stack and return
        // this is a workaround for the fact that it would be too expensive to iterate over the two whole stacks when user withdraws
        // TO CHECK:
        //   - if the user has withdrawn and another user has staked the same token during the same preparation
        //   - the stack should rebalance at the beginning of each battle's tx, but what if this happens during the last tx?

        // BEGIN
        if self
            .stats_for_nft(&first_token.token_id, first_token.nonce)
            .get()
            .owner
            .is_zero()
        {
            first_stack_mapper.swap_remove(first_random_index);
            return;
        } else if self
            .stats_for_nft(&second_token.token_id, second_token.nonce)
            .get()
            .owner
            .is_zero()
        {
            second_stack_mapper.swap_remove(second_random_index);
            return;
        }
        // END

        let first_token_attributes = self
            .token_attributes(&first_token.token_id, first_token.nonce)
            .get();
        let second_token_attributes = self
            .token_attributes(&second_token.token_id, second_token.nonce)
            .get();

        match first_token_attributes
            .power
            .cmp(&second_token_attributes.power)
        {
            Ordering::Greater => self.update_stats(&first_token, &second_token),
            Ordering::Less => self.update_stats(&second_token, &first_token),
            Ordering::Equal => self.handle_tiebreak(
                &first_token,
                &first_token_attributes,
                &second_token,
                &second_token_attributes,
            ),
        }

        self.first_stack(current_battle + 1).push(&second_token);
        self.second_stack(current_battle + 1).push(&first_token);

        first_stack_mapper.swap_remove(first_random_index);
        second_stack_mapper.swap_remove(second_random_index);
    }

    fn handle_tiebreak(
        &self,
        first_token: &Token<Self::Api>,
        first_token_attr: &Attributes,
        second_token: &Token<Self::Api>,
        second_token_attr: &Attributes,
    ) {
        let emidas_token_id = self.emidas_token_id().get();
        let gnogon_token_id = self.gnogon_token_id().get();
        let validator_token_id = self.validator_v2_token_id().get();

        if first_token.token_id == emidas_token_id && second_token.token_id != emidas_token_id {
            self.update_stats(first_token, second_token);
        } else if first_token.token_id != emidas_token_id
            && second_token.token_id == emidas_token_id
        {
            self.update_stats(second_token, first_token);
        } else if first_token.token_id == gnogon_token_id
            && second_token.token_id == validator_token_id
        {
            self.update_stats(first_token, second_token);
        } else if first_token.token_id == validator_token_id
            && second_token.token_id == gnogon_token_id
        {
            self.update_stats(second_token, first_token);
        } else if first_token.token_id == gnogon_token_id
            && second_token.token_id == gnogon_token_id
        {
            match first_token_attr.heart.cmp(&second_token_attr.heart) {
                Ordering::Greater => self.update_stats(first_token, second_token),
                Ordering::Less => self.update_stats(second_token, first_token),
                Ordering::Equal => self.update_stats_both_losers(first_token, second_token),
            }
        } else if first_token.token_id == validator_token_id
            && second_token.token_id == validator_token_id
        {
            match first_token_attr.ram.cmp(&second_token_attr.ram) {
                Ordering::Greater => self.update_stats(first_token, second_token),
                Ordering::Less => self.update_stats(second_token, first_token),
                Ordering::Equal => self.update_stats_both_losers(first_token, second_token),
            }
        }
    }

    fn update_stats<'a>(&self, mut winner: &'a Token<Self::Api>, mut loser: &'a Token<Self::Api>) {
        if self.is_today_a_sunday() {
            (winner, loser) = (loser, winner);
        }

        let winner_token_stats = self.stats_for_nft(&winner.token_id, winner.nonce);
        let loser_token_stats = self.stats_for_nft(&loser.token_id, loser.nonce);
        let winner_attributes = self.token_attributes(&winner.token_id, winner.nonce).get();
        let current_battle = self.current_battle().get();

        // update winner
        winner_token_stats.update(|prev| prev.win += 1);
        self.stats_for_address(&winner_token_stats.get().owner)
            .update(|prev| {
                prev.win += 1;
            });
        // rn adding one item by nft winner
        self.raw_pending_rewards_for_address(&winner_token_stats.get().owner)
            .insert(PendingRewards {
                battle_id: current_battle,
                power: winner_attributes.power as u64,
            });
        self.battle_history(current_battle).update(|prev| {
            prev.total_winner_power += winner_attributes.power as u64;
        });

        // update loser
        loser_token_stats.update(|prev| prev.loss += 1);
        self.stats_for_address(&loser_token_stats.get().owner)
            .update(|prev| prev.loss += 1);
    }

    fn update_stats_both_losers(&self, loser1: &Token<Self::Api>, loser2: &Token<Self::Api>) {
        let loser1_token_stats = self.stats_for_nft(&loser1.token_id, loser1.nonce);
        let loser2_token_stats = self.stats_for_nft(&loser2.token_id, loser2.nonce);

        // update loser1
        loser1_token_stats.update(|prev| prev.loss += 1);
        self.stats_for_address(&loser1_token_stats.get().owner)
            .update(|prev| prev.loss += 1);

        // update loser2
        loser2_token_stats.update(|prev| prev.loss += 1);
        self.stats_for_address(&loser2_token_stats.get().owner)
            .update(|prev| prev.loss += 1);
    }

    fn rebalance_stacks(&self) {
        let current_battle = self.current_battle().get();
        let mut first_stack_mapper = self.first_stack(current_battle);
        let mut second_stack_mapper = self.second_stack(current_battle);

        let initial_len_first_stack = first_stack_mapper.len();
        let initial_len_second_stack = second_stack_mapper.len();

        if initial_len_first_stack > initial_len_second_stack + 1 {
            let mut diff = initial_len_first_stack - initial_len_second_stack;

            while diff > 1 {
                // indexes start at 1
                let last_item_index = first_stack_mapper.len();
                let item_to_switch = first_stack_mapper.get(last_item_index);

                second_stack_mapper.push(&item_to_switch);
                first_stack_mapper.swap_remove(last_item_index);

                diff -= 2;
            }
        } else if initial_len_second_stack > initial_len_first_stack + 1 {
            let mut diff = initial_len_second_stack - initial_len_first_stack;

            while diff > 1 {
                let last_item_index = second_stack_mapper.len();
                let item_to_switch = second_stack_mapper.get(last_item_index);

                first_stack_mapper.push(&item_to_switch);
                second_stack_mapper.swap_remove(last_item_index);

                diff -= 2;
            }
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

            let token_stats = self.stats_for_nft(&token.token_id, token.nonce);

            token_stats.update(|prev| prev.loss += 1);
            self.stats_for_address(&token_stats.get().owner)
                .update(|prev| prev.loss += 1);
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

    #[view(isTodayASunday)]
    fn is_today_a_sunday(&self) -> bool {
        let current_timestamp = self.blockchain().get_block_timestamp();

        let days = current_timestamp / 60 / 60 / 24;
        let weekday = (days + 4) % 7;

        // Sunday is index 0 [sunday, monday, ... , saturday]
        weekday == 0
    }

    #[view(getAllStakedForAddress)]
    fn get_all_staked_for_address(
        &self,
        address: ManagedAddress,
    ) -> MultiValueEncoded<MultiValue2<TokenIdentifier, u64>> {
        let mut result: MultiValueEncoded<MultiValue2<TokenIdentifier, u64>> =
            MultiValueEncoded::new();

        for collection in self.battle_tokens().iter() {
            for nonce in self.staked_for_address(&address, &collection).iter() {
                result.push(MultiValue2::from((collection.clone(), nonce)));
            }
        }

        result
    }

    #[view(getPendingRewardsForAddress)]
    fn get_pending_rewards_for_address(&self, address: &ManagedAddress) -> BigUint {
        let raw_pending_rewards = self.raw_pending_rewards_for_address(address);
        let daily_reward_amount = self.daily_reward_amount().get();

        let mut result = BigUint::zero();

        for pending_reward in raw_pending_rewards.iter() {
            let battle_history = self.battle_history(pending_reward.battle_id).get();
            let total_winner_power = battle_history.total_winner_power;

            let big_power = BigUint::from(pending_reward.power).mul(DIVISION_PRECISION);
            let big_total_winner_power = BigUint::from(total_winner_power).mul(DIVISION_PRECISION);

            // TO DO: check if this is correct
            let rewards = big_power
                .div(big_total_winner_power)
                .mul(&daily_reward_amount)
                .div(DIVISION_PRECISION);

            result += rewards;
        }
        result
    }

    #[view(getStatsForAddress)]
    fn get_stats_for_address(&self, address: &ManagedAddress) -> UserStats {
        let stats_mapper = self.stats_for_address(address);

        if stats_mapper.is_empty() {
            UserStats::default()
        } else {
            stats_mapper.get()
        }
    }

    #[view(getStatsForNft)]
    fn get_stats_for_nft(
        &self,
        nfts: MultiValueEncoded<MultiValue2<TokenIdentifier, u64>>,
    ) -> MultiValueEncoded<MultiValue3<TokenIdentifier, u64, TokenStats<Self::Api>>> {
        let mut result = MultiValueEncoded::new();

        for nft in nfts.into_iter() {
            let (token_id, nonce) = nft.into_tuple();
            let stats_mapper = self.stats_for_nft(&token_id, nonce);

            if stats_mapper.is_empty() {
                result.push(MultiValue3::from((token_id, nonce, TokenStats::default())));
            } else {
                result.push(MultiValue3::from((token_id, nonce, stats_mapper.get())));
            }
        }
        result
    }

    #[storage_mapper("statsForAddress")]
    fn stats_for_address(&self, address: &ManagedAddress) -> SingleValueMapper<UserStats>;

    #[storage_mapper("statsForNft")]
    fn stats_for_nft(
        &self,
        token_id: &TokenIdentifier,
        nonce: Nonce,
    ) -> SingleValueMapper<TokenStats<Self::Api>>;

    #[view(getCurrentBattle)]
    #[storage_mapper("currentBattle")]
    fn current_battle(&self) -> SingleValueMapper<u64>;

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

    #[view(getTotalNftEngaged)]
    #[storage_mapper("totalNftEngaged")]
    fn total_nft_engaged(&self) -> SingleValueMapper<u64>;

    #[view(getBattleHistory)]
    #[storage_mapper("battleHistory")]
    fn battle_history(&self, battle: u64) -> SingleValueMapper<BattleHistory>;

    #[view(getRawPendingRewardsForAddress)]
    #[storage_mapper("rawPendingRewardsForAddress")]
    fn raw_pending_rewards_for_address(
        &self,
        address: &ManagedAddress,
    ) -> UnorderedSetMapper<PendingRewards>;
}
