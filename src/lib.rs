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
use operations::{LoopOp, OperationCompletionStatus};

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

        self.last_staked_id().set_if_empty(0);

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
        self.raw_pending_rewards_for_address(&caller)
            .set_if_empty(PendingRewards::default());

        let mut staked_id = self.last_staked_id().get();

        for payment in payments.iter() {
            let (token_id, nonce, amount) = payment.into_tuple();
            require!(self.battle_tokens().contains(&token_id), "Wrong token");
            require!(amount == NFT_AMOUNT, "Invalid token amount");

            let current_battle = self.current_battle().get();

            self.staked_for_address(&caller, &token_id).insert(nonce);

            staked_id += 1;

            if self.stats_for_nft(&token_id, nonce).is_empty() {
                self.stats_for_nft(&token_id, nonce).set(TokenStats {
                    win: 0,
                    loss: 0,
                    owner: caller.clone(),
                    current_id_token: staked_id,
                });
            } else {
                self.stats_for_nft(&token_id, nonce).update(|prev| {
                    prev.owner = caller.clone();
                    prev.current_id_token = staked_id;
                });
            }

            self.battle_stack(current_battle).push(&staked_id);
            self.token_by_unique_id(staked_id)
                .set(Token { token_id, nonce });
        }

        self.addresses().insert(caller);
        self.last_staked_id().set(staked_id);
        self.total_nft_engaged()
            .update(|prev| *prev += payments.len() as u64);
    }

    #[endpoint]
    fn battle(&self) -> MultiValue2<OperationCompletionStatus, u64> {
        require!(
            self.get_battle_status() == BattleStatus::Battle,
            "Battle in preparation"
        );

        let current_battle = self.current_battle().get();

        if self.battle_history(current_battle).is_empty() {
            self.battle_history(current_battle).set(BattleHistory {
                battle_id: current_battle,
                total_winner_power: 0,
            });
        }

        let mut amount_of_battles_done: u64 = 0;

        let result = self.run_while_it_has_gas(|| {
            if self.battle_stack(current_battle).len() <= 1 {
                self.drain_stack_and_fill_next_battle(self.battle_stack(current_battle));
                return LoopOp::Break;
            }

            let is_real_battle = self.single_battle();
            if is_real_battle {
                amount_of_battles_done += 1;
            }

            LoopOp::Continue
        });

        if amount_of_battles_done > 0 {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &self.gng_token_id().get(),
                0,
                &(self.base_battle_reward_amount().get() * amount_of_battles_done),
            );
            self.reward_capacity().update(|prev| {
                *prev -= self.base_battle_reward_amount().get() * amount_of_battles_done
            });
        }

        if result.is_completed() {
            self.current_battle().update(|current| *current += 1);
        }

        MultiValue2::from((result, amount_of_battles_done))
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

            self.raw_pending_rewards_for_address(&caller)
                .set(PendingRewards::default());
            self.reward_capacity()
                .update(|prev| *prev -= total_rewards.clone());
            self.stats_for_address(&caller)
                .update(|prev| prev.gng_claimed += total_rewards);
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

            self.staked_for_address(&caller, &token_id)
                .swap_remove(&nonce);
            self.token_by_unique_id(token_stats.current_id_token)
                .clear();
            self.stats_for_nft(&token_id, nonce).update(|prev| {
                prev.owner = ManagedAddress::zero();
                prev.current_id_token = 0;
            });

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

    fn single_battle(&self) -> bool {
        let current_battle = self.current_battle().get();
        let mut battle_stack_mapper = self.battle_stack(current_battle);

        let battle_stack_len = battle_stack_mapper.len();

        let (first_random_index, second_random_index) =
            self.get_two_distinct_random_index(1, battle_stack_len + 1);

        let first_token_unique_id = battle_stack_mapper.get(first_random_index);
        let second_token_unique_id = battle_stack_mapper.get(second_random_index);

        // if one of the tokens has its id different than the one in token_stats, it means the owner has withdrawn it during the preparation
        // we remove it from the stack and return
        // this is a workaround for the fact that it would be too expensive to iterate over the two whole stacks when user withdraws
        // TO CHECK:
        //   - if the user has withdrawn and another user has staked the same token during the same preparation
        //   - the stack should rebalance at the beginning of each battle's tx, but what if this happens during the last tx?

        // BEGIN
        if self.token_by_unique_id(first_token_unique_id).is_empty() {
            battle_stack_mapper.swap_remove(first_random_index);
            return false;
        } else if self.token_by_unique_id(second_token_unique_id).is_empty() {
            battle_stack_mapper.swap_remove(second_random_index);
            return false;
        }
        // END

        let first_token = self.token_by_unique_id(first_token_unique_id).get();
        let second_token = self.token_by_unique_id(second_token_unique_id).get();

        let first_token_attributes =
            self.get_token_attributes(&first_token.token_id, first_token.nonce);
        let second_token_attributes =
            self.get_token_attributes(&second_token.token_id, second_token.nonce);

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

        self.battle_stack(current_battle + 1)
            .push(&first_token_unique_id);
        self.battle_stack(current_battle + 1)
            .push(&second_token_unique_id);

        // needs to remove the greater index first because of the behaviour of swap_remove
        if first_random_index > second_random_index {
            battle_stack_mapper.swap_remove(first_random_index);
            battle_stack_mapper.swap_remove(second_random_index);
        } else {
            battle_stack_mapper.swap_remove(second_random_index);
            battle_stack_mapper.swap_remove(first_random_index);
        }

        return true;
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
        } else {
            self.update_stats_both_losers(first_token, second_token);
        }
    }

    fn update_stats<'a>(&self, mut winner: &'a Token<Self::Api>, mut loser: &'a Token<Self::Api>) {
        if self.is_today_a_sunday() {
            (winner, loser) = (loser, winner);
        }

        let winner_token_stats = self.stats_for_nft(&winner.token_id, winner.nonce);
        let loser_token_stats = self.stats_for_nft(&loser.token_id, loser.nonce);
        let winner_attributes = self.get_token_attributes(&winner.token_id, winner.nonce);
        let current_battle = self.current_battle().get();

        // update winner
        winner_token_stats.update(|prev| prev.win += 1);
        self.stats_for_address(&winner_token_stats.get().owner)
            .update(|prev| {
                prev.win += 1;
            });

        let winner_rewards_mapper =
            self.raw_pending_rewards_for_address(&winner_token_stats.get().owner);
        let winner_rewards = self
            .raw_pending_rewards_for_address(&winner_token_stats.get().owner)
            .get();
        if winner_rewards.awaiting_battle_id == 0 {
            winner_rewards_mapper.update(|prev| {
                prev.awaiting_battle_id = current_battle;
                prev.awaiting_power = winner_attributes.power as u64;
            });
        } else if winner_rewards.awaiting_battle_id < current_battle {
            let rewards_to_add = self.calculate_single_battle_rewards(
                winner_rewards.awaiting_battle_id,
                winner_rewards.awaiting_power,
            );
            winner_rewards_mapper.update(|prev| {
                prev.calculated_rewards += rewards_to_add;
                prev.awaiting_battle_id = current_battle;
                prev.awaiting_power = winner_attributes.power as u64;
            });
        } else {
            winner_rewards_mapper.update(|prev| {
                prev.awaiting_power += winner_attributes.power as u64;
            });
        }

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

    fn drain_stack_and_fill_next_battle(&self, mut stack_to_drain: VecMapper<u64>) {
        let current_battle = self.current_battle().get();
        let len = stack_to_drain.len();
        for i in 1..(len + 1) {
            let unique_token_id = stack_to_drain.get(i);

            if self.token_by_unique_id(unique_token_id).is_empty() {
                continue;
            }

            let token = self.token_by_unique_id(unique_token_id).get();
            let token_stats = self.stats_for_nft(&token.token_id, token.nonce);

            self.battle_stack(current_battle + 1).push(&unique_token_id);

            token_stats.update(|prev| prev.loss += 1);
            self.stats_for_address(&token_stats.get().owner)
                .update(|prev| prev.loss += 1);
        }
        stack_to_drain.clear();
    }

    // TO CHECK
    fn get_two_distinct_random_index(&self, min: usize, max: usize) -> (usize, usize) {
        let mut rand = RandomnessSource::<Self::Api>::new();
        let first_idx = rand.next_usize_in_range(min, max);
        let mut second_idx = rand.next_usize_in_range(min, max);

        // we assume that max > (min + 1)
        if second_idx == first_idx {
            second_idx = second_idx + 1;
            if second_idx == max {
                second_idx = min;
            }
        }

        (first_idx, second_idx)
    }

    fn calculate_single_battle_rewards(&self, battle_id: u64, power: u64) -> BigUint {
        let daily_reward_amount = self.get_daily_reward_amount_with_halving();
        let battle_history = self.battle_history(battle_id).get();
        let total_winner_power = battle_history.total_winner_power;

        let big_power = BigUint::from(power).mul(DIVISION_PRECISION);
        let big_total_winner_power = BigUint::from(total_winner_power);

        big_power
            .div(big_total_winner_power)
            .mul(&daily_reward_amount)
            .div(DIVISION_PRECISION)
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
        address: &ManagedAddress,
    ) -> MultiValueEncoded<MultiValue2<TokenIdentifier, u64>> {
        let mut result: MultiValueEncoded<MultiValue2<TokenIdentifier, u64>> =
            MultiValueEncoded::new();

        for collection in self.battle_tokens().iter() {
            for nonce in self.staked_for_address(address, &collection).iter() {
                result.push(MultiValue2::from((collection.clone(), nonce)));
            }
        }

        result
    }

    #[view(getPendingRewardsForAddress)]
    fn get_pending_rewards_for_address(&self, address: &ManagedAddress) -> BigUint {
        if self.raw_pending_rewards_for_address(address).is_empty() {
            return BigUint::zero();
        }
        let pending_rewards = self.raw_pending_rewards_for_address(address).get();

        if (pending_rewards.awaiting_battle_id == self.current_battle().get()
            && self.get_battle_status() == BattleStatus::Battle)
            || pending_rewards.awaiting_battle_id == 0
        {
            pending_rewards.calculated_rewards
        } else {
            let rewards_to_add = self.calculate_single_battle_rewards(
                pending_rewards.awaiting_battle_id,
                pending_rewards.awaiting_power,
            );
            pending_rewards.calculated_rewards + rewards_to_add
        }
    }

    #[view(getStatsForAddress)]
    fn get_stats_for_address(&self, address: &ManagedAddress) -> UserStats<Self::Api> {
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

    #[view(getGlobalStats)]
    fn get_global_stats(
        &self,
    ) -> MultiValueEncoded<MultiValue4<ManagedAddress, u64, BigUint, u16>> {
        let mut result = MultiValueEncoded::new();

        for address in self.addresses().iter() {
            let mut total_power = 0u64;
            let mut total_gng = BigUint::zero();
            let mut total_nfts = 0u16;
            let nfts_staked = self.get_all_staked_for_address(&address);

            for nft in nfts_staked.into_iter() {
                let (token_id, nonce) = nft.into_tuple();
                total_power += self.get_token_attributes(&token_id, nonce).power as u64;
                total_nfts += 1;
            }

            total_gng += self.stats_for_address(&address).get().gng_claimed;
            total_gng += self.get_pending_rewards_for_address(&address);

            result.push(MultiValue4::from((
                address.clone(),
                total_power,
                total_gng,
                total_nfts,
            )));
        }
        result
    }

    #[storage_mapper("statsForAddress")]
    fn stats_for_address(
        &self,
        address: &ManagedAddress,
    ) -> SingleValueMapper<UserStats<Self::Api>>;

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

    #[view(getBattleStack)]
    #[storage_mapper("battleStack")]
    fn battle_stack(&self, battle: u64) -> VecMapper<u64>;

    #[view(getTokenByUniqueId)]
    #[storage_mapper("tokenByUniqueId")]
    fn token_by_unique_id(&self, id: u64) -> SingleValueMapper<Token<Self::Api>>;

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
    ) -> SingleValueMapper<PendingRewards<Self::Api>>;

    #[view(getLastStakedId)]
    #[storage_mapper("lastStakedId")]
    fn last_staked_id(&self) -> SingleValueMapper<u64>;

    #[view(getAddresses)]
    #[storage_mapper("addresses")]
    fn addresses(&self) -> UnorderedSetMapper<ManagedAddress>;
}
