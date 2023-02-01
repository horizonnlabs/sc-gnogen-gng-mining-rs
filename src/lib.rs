#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod config;
mod events;
mod model;
mod operations;

use core::cmp::Ordering;

use config::State;
use model::{Attributes, BattleStatus, Nonce, PendingRewards, Token, UserStats};
use operations::{LoopOp, OperationCompletionStatus};

const NFT_AMOUNT: u64 = 1;
const ONE_DAY_TIMESTAMP: u64 = 86400;
const DIVISION_PRECISION: u64 = 1000000;

#[elrond_wasm::contract]
pub trait GngMinting:
    config::ConfigModule + operations::OngoingOperationModule + events::EventsModule
{
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
        self.raw_pending_rewards_for_address(&caller)
            .set_if_empty(PendingRewards::default());

        let mut total_power = 0;

        for payment in payments.iter() {
            let (token_id, nonce, amount) = payment.into_tuple();
            require!(self.battle_tokens().contains(&token_id), "Wrong token");
            require!(amount == NFT_AMOUNT, "Invalid token amount");

            self.staked_for_address(&caller, &token_id).insert(nonce);
            let attributes = self.token_attributes(&token_id, nonce).get();
            total_power += attributes.power;

            self.nft_owner(&token_id, nonce).set(caller.clone());

            self.battle_stack().insert(Token { token_id, nonce });
        }

        self.stats_for_address(&caller).update(|prev| {
            prev.power += total_power as u64;
        });

        self.addresses().insert(caller);
        self.total_nft_engaged()
            .update(|prev| *prev += payments.len() as u64);
    }

    #[endpoint]
    fn battle(&self) -> MultiValue2<OperationCompletionStatus, u64> {
        require!(self.is_active(), "Not active");
        require!(
            self.get_battle_status() == BattleStatus::Battle,
            "Battle in preparation"
        );

        let current_battle = self.current_battle().get();

        if !self.has_battle_started(current_battle).get() {
            self.unique_id_battle_stack(current_battle)
                .set_initial_len(self.battle_stack().len());
            self.has_battle_started(current_battle).set(true);
            self.start_battle_event(current_battle);
        }

        let mut amount_of_clashes_done: u64 = 0;
        let mut total_winner_power: u64 = 0;

        let result =
            self.run_while_it_has_gas(|| match self.unique_id_battle_stack(current_battle).len() {
                0 => LoopOp::Break,
                1 => {
                    self.drain_stack();
                    LoopOp::Break
                }
                _ => {
                    let clash_winner_power = self.clash();
                    total_winner_power += clash_winner_power;
                    amount_of_clashes_done += 1;

                    LoopOp::Continue
                }
            });

        if amount_of_clashes_done > 0 {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &self.gng_token_id().get(),
                0,
                &self.calculate_clash_operator_rewards(amount_of_clashes_done),
            );
            self.reward_capacity().update(|prev| {
                *prev -= self.base_battle_reward_amount().get() * amount_of_clashes_done
            });
            self.total_battle_winner_power(current_battle)
                .update(|prev| *prev += total_winner_power);
        }

        if result.is_completed() {
            self.end_battle_event(current_battle);
            self.current_battle().update(|current| *current += 1);
        }

        MultiValue2::from((result, amount_of_clashes_done))
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(self.is_active(), "Not active");
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
        require!(self.is_active(), "Not active");
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );

        self.claim_rewards();

        let caller = self.blockchain().get_caller();
        let amount_tokens = tokens.len();

        let mut output_payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let mut total_power = 0;

        for token in tokens.into_iter() {
            let (token_id, nonce) = token.into_tuple();
            let token_owner = self.nft_owner(&token_id, nonce).get();

            let attributes = self.token_attributes(&token_id, nonce).get();
            total_power += attributes.power;

            require!(
                token_owner == caller
                    && self.staked_for_address(&caller, &token_id).contains(&nonce),
                "Wrong token"
            );

            self.nft_owner(&token_id, nonce).set(ManagedAddress::zero());

            self.staked_for_address(&caller, &token_id)
                .swap_remove(&nonce);

            self.battle_stack().swap_remove(&Token {
                token_id: token_id.clone(),
                nonce,
            });

            output_payments.push(EsdtTokenPayment::new(
                token_id,
                nonce,
                BigUint::from(NFT_AMOUNT),
            ));
        }

        self.stats_for_address(&caller).update(|prev| {
            prev.power -= total_power as u64;
        });

        self.total_nft_engaged()
            .update(|prev| *prev -= amount_tokens as u64);

        self.send().direct_multi(&caller, &output_payments);
    }

    /// We assume there is at least 2 tokens in the stack
    fn clash(&self) -> u64 {
        let current_battle = self.current_battle().get();
        let battle_stack_mapper = self.battle_stack();
        let mut unique_id_battle_stack_mapper = self.unique_id_battle_stack(current_battle);

        let first_random_index = self.get_random_index(1, unique_id_battle_stack_mapper.len() + 1);
        let first_token_idx = unique_id_battle_stack_mapper.get(first_random_index);
        let first_token = battle_stack_mapper.get_by_index(first_token_idx);
        unique_id_battle_stack_mapper.swap_remove(first_random_index);

        let second_random_index = self.get_random_index(1, unique_id_battle_stack_mapper.len() + 1);
        let second_token_idx = unique_id_battle_stack_mapper.get(second_random_index);
        let second_token = battle_stack_mapper.get_by_index(second_token_idx);
        unique_id_battle_stack_mapper.swap_remove(second_random_index);

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
    }

    fn handle_tiebreak(
        &self,
        first_token: &Token<Self::Api>,
        first_token_attr: &Attributes,
        second_token: &Token<Self::Api>,
        second_token_attr: &Attributes,
    ) -> u64 {
        let emidas_token_id = self.emidas_token_id().get();
        let gnogon_token_id = self.gnogon_token_id().get();
        let validator_token_id = self.validator_v2_token_id().get();

        if first_token.token_id == emidas_token_id && second_token.token_id != emidas_token_id {
            self.update_stats(first_token, second_token)
        } else if first_token.token_id != emidas_token_id
            && second_token.token_id == emidas_token_id
        {
            self.update_stats(second_token, first_token)
        } else if first_token.token_id == gnogon_token_id
            && second_token.token_id == validator_token_id
        {
            self.update_stats(first_token, second_token)
        } else if first_token.token_id == validator_token_id
            && second_token.token_id == gnogon_token_id
        {
            self.update_stats(second_token, first_token)
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
            self.update_stats_both_losers(first_token, second_token)
        }
    }

    fn update_stats<'a>(
        &self,
        mut winner: &'a Token<Self::Api>,
        mut loser: &'a Token<Self::Api>,
    ) -> u64 {
        if self.is_today_special() {
            (winner, loser) = (loser, winner);
        }

        let winner_address = self.nft_owner(&winner.token_id, winner.nonce).get();
        let winner_power = self
            .get_token_attributes(&winner.token_id, winner.nonce)
            .power as u64;
        let current_battle = self.current_battle().get();

        let winner_rewards_mapper = self.raw_pending_rewards_for_address(&winner_address);
        let winner_rewards = self.raw_pending_rewards_for_address(&winner_address).get();
        if winner_rewards.awaiting_battle_id == 0 {
            winner_rewards_mapper.update(|prev| {
                prev.awaiting_battle_id = current_battle;
                prev.awaiting_power = winner_power;
            });
        } else if winner_rewards.awaiting_battle_id < current_battle {
            let rewards_to_add = self.calculate_clash_rewards(
                winner_rewards.awaiting_battle_id,
                winner_rewards.awaiting_power,
            );
            winner_rewards_mapper.update(|prev| {
                prev.calculated_rewards += rewards_to_add;
                prev.awaiting_battle_id = current_battle;
                prev.awaiting_power = winner_power;
            });
        } else {
            winner_rewards_mapper.update(|prev| {
                prev.awaiting_power += winner_power;
            });
        }

        self.clash_event(
            current_battle,
            winner,
            loser,
            false,
            &winner_address,
            &self.nft_owner(&loser.token_id, loser.nonce).get(),
        );

        winner_power
    }

    fn update_stats_both_losers(
        &self,
        loser1: &Token<Self::Api>,
        loser2: &Token<Self::Api>,
    ) -> u64 {
        self.clash_event(
            self.current_battle().get(),
            loser1,
            loser2,
            true,
            &self.nft_owner(&loser1.token_id, loser1.nonce).get(),
            &self.nft_owner(&loser2.token_id, loser2.nonce).get(),
        );
        0
    }

    /// We assume there is exactly one token in the stack
    fn drain_stack(&self) {
        let current_battle = self.current_battle().get();
        let mut unique_id_stack_mapper = self.unique_id_battle_stack(current_battle);
        let token_idx = unique_id_stack_mapper.get(1);
        let token = self.battle_stack().get_by_index(token_idx);

        //emit event
        self.single_token_clash_event(
            current_battle,
            &token,
            self.nft_owner(&token.token_id, token.nonce).get(),
        );

        unique_id_stack_mapper.swap_remove(1);
    }

    fn get_random_index(&self, min: usize, max: usize) -> usize {
        let mut rand = RandomnessSource::new();
        rand.next_usize_in_range(min, max)
    }

    fn calculate_clash_rewards(&self, battle_id: u64, power: u64) -> BigUint {
        let daily_reward_amount = self.get_daily_reward_amount_with_halving();
        let total_winner_power = self.total_battle_winner_power(battle_id).get();

        let big_power = BigUint::from(power).mul(DIVISION_PRECISION);

        big_power
            .div(total_winner_power)
            .mul(&daily_reward_amount)
            .div(DIVISION_PRECISION)
    }

    fn calculate_clash_operator_rewards(&self, amount_of_clashes_performed: u64) -> BigUint {
        let total_rewards_for_one_battle = self.base_battle_reward_amount().get();
        let total_clashes_amount = (self.battle_stack().len() / 2) as u64;

        let big_amount_of_clashes_performed =
            BigUint::from(amount_of_clashes_performed).mul(DIVISION_PRECISION);

        big_amount_of_clashes_performed
            .div(total_clashes_amount)
            .mul(&total_rewards_for_one_battle)
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

    #[view(isTodaySpecial)]
    fn is_today_special(&self) -> bool {
        let current_timestamp = self.blockchain().get_block_timestamp();

        let days = current_timestamp / 60 / 60 / 24;
        let weekday = (days + 4) % 7;

        // Sunday is index 0 [sun, mon, tue, wed, thu, fri, sat]
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
            let rewards_to_add = self.calculate_clash_rewards(
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

    #[view(getGlobalStats)]
    fn get_global_stats(&self) -> MultiValueEncoded<MultiValue3<ManagedAddress, BigUint, u64>> {
        let mut result = MultiValueEncoded::new();

        for address in self.addresses().iter() {
            let stats_for_address = self.stats_for_address(&address).get();
            let power = stats_for_address.power;
            let mut total_gng = BigUint::zero();

            total_gng += stats_for_address.gng_claimed;
            total_gng += self.get_pending_rewards_for_address(&address);

            result.push(MultiValue3::from((address.clone(), total_gng, power)));
        }
        result
    }

    #[view(getGlobalStatsByAddress)]
    fn get_global_stats_by_address(
        &self,
        address: &ManagedAddress,
    ) -> MultiValue3<ManagedAddress, u64, BigUint> {
        let stats_for_address = self.stats_for_address(address).get();
        let power = stats_for_address.power;
        let mut total_gng = BigUint::zero();

        total_gng += self.stats_for_address(address).get().gng_claimed;
        total_gng += self.get_pending_rewards_for_address(address);

        MultiValue3::from((address.clone(), power, total_gng))
    }

    #[storage_mapper("statsForAddress")]
    fn stats_for_address(
        &self,
        address: &ManagedAddress,
    ) -> SingleValueMapper<UserStats<Self::Api>>;

    #[view(getNftOwner)]
    #[storage_mapper("nft_owner")]
    fn nft_owner(
        &self,
        token_id: &TokenIdentifier,
        nonce: Nonce,
    ) -> SingleValueMapper<ManagedAddress>;

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
    fn battle_stack(&self) -> UnorderedSetMapper<Token<Self::Api>>;

    #[view(getUniqueIdBattleStack)]
    #[storage_mapper("uniqueIdBattleStack")]
    fn unique_id_battle_stack(&self, battle: u64) -> UniqueIdMapper<Self::Api>;

    #[view(hasBattleStarted)]
    #[storage_mapper("hasBattleStarted")]
    fn has_battle_started(&self, battle: u64) -> SingleValueMapper<bool>;

    #[view(getTotalNftEngaged)]
    #[storage_mapper("totalNftEngaged")]
    fn total_nft_engaged(&self) -> SingleValueMapper<u64>;

    #[view(getTotalBattleWinnerPower)]
    #[storage_mapper("totalBattleWinnerPower")]
    fn total_battle_winner_power(&self, battle: u64) -> SingleValueMapper<u64>;

    #[view(getRawPendingRewardsForAddress)]
    #[storage_mapper("rawPendingRewardsForAddress")]
    fn raw_pending_rewards_for_address(
        &self,
        address: &ManagedAddress,
    ) -> SingleValueMapper<PendingRewards<Self::Api>>;

    #[view(getAddresses)]
    #[storage_mapper("addresses")]
    fn addresses(&self) -> UnorderedSetMapper<ManagedAddress>;
}
