#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;
mod events;
mod model;
mod operations;

use core::cmp::Ordering;

use model::{Attributes, BattleMode, BattleStatus, Nonce, PendingRewards, Token, UserStats};
use operations::{LoopOp, OperationCompletionStatus};

const NFT_AMOUNT: u64 = 1;
const ONE_DAY_TIMESTAMP: u64 = 86400;
const ONE_WEEK_TIMESTAMP: u64 = ONE_DAY_TIMESTAMP * 7;
const SPECIAL_WEEK_RECURRENCE: u64 = 7;

#[multiversx_sc::contract]
pub trait GngMinting:
    config::ConfigModule + operations::OngoingOperationModule + events::EventsModule
{
    #[init]
    fn init(&self, first_battle_timestamp: u64, gng_token_id: TokenIdentifier) {
        if self.first_battle_timestamp().is_empty() {
            require!(
                first_battle_timestamp > self.blockchain().get_block_timestamp(),
                "Cannot backdate first battle"
            );
            self.first_battle_timestamp()
                .set_if_empty(first_battle_timestamp);
        }

        self.current_battle().set_if_empty(1);
        self.gng_token_id().set_if_empty(gng_token_id);

        self.first_battle_timestamp_current_period()
            .set_if_empty(self.first_battle_timestamp().get());

        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );
    }

    #[upgrade]
    fn upgrade(&self) {
        self.init(
            self.first_battle_timestamp().get(),
            self.gng_token_id().get(),
        );
    }

    #[only_owner]
    #[endpoint(switchMode)]
    fn switch_mode(&self, mode: &BattleMode) {
        require!(
            self.get_battle_status() == BattleStatus::Preparation,
            "Battle in progress"
        );
        require!(&self.battle_mode().get() != mode, "Already in this mode");

        let current_battle = self.current_battle().get();
        let past_battle_amount = self.past_battle_amount().get();
        let first_battle_timestamp_current_period =
            self.first_battle_timestamp_current_period().get();
        let first_battle_timestamp_new_period = match self.battle_mode().get() {
            BattleMode::Daily => {
                first_battle_timestamp_current_period
                    + (ONE_DAY_TIMESTAMP * (current_battle - past_battle_amount - 1))
            }
            BattleMode::Weekly => {
                if current_battle == past_battle_amount + 1 {
                    first_battle_timestamp_current_period
                } else {
                    let last_battle_timestamp = first_battle_timestamp_current_period
                        + (ONE_WEEK_TIMESTAMP * ((current_battle - 1) - past_battle_amount - 1));
                    require!(
                        last_battle_timestamp + ONE_DAY_TIMESTAMP
                            > self.blockchain().get_block_timestamp(),
                        "Cannot switch to daily mode too late"
                    );
                    last_battle_timestamp + ONE_DAY_TIMESTAMP
                }
            }
        };

        self.past_battle_amount()
            .set(self.current_battle().get() - 1);
        self.first_battle_timestamp_current_period()
            .set(first_battle_timestamp_new_period);
        self.battle_mode().set(mode);
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

            let owner_mapper = self.nft_owner(&token_id, nonce);
            require!(owner_mapper.is_empty(), "Cannot stake SFT");
            owner_mapper.set(caller.clone());

            self.staked_for_address(&caller, &token_id).insert(nonce);

            require!(
                !self.token_attributes(&token_id, nonce).is_empty(),
                "Cant stake NFT without attributes"
            );
            let attributes = self.token_attributes(&token_id, nonce).get();
            total_power += attributes.power as u64;

            self.battle_stack().insert(Token { token_id, nonce });
        }

        self.stats_for_address(&caller).update(|prev| {
            prev.power += total_power;
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
            let battle_reward_amount = self.get_battle_reward_amount_with_halving();
            let operators_reward_amount = self.get_battle_operator_rewards();

            require!(
                self.reward_capacity().get() >= &battle_reward_amount + &operators_reward_amount,
                "Not enough rewards to start battle"
            );
            self.total_rewards_for_stakers(current_battle)
                .set(&battle_reward_amount);
            self.total_rewards_for_operators(current_battle)
                .set(&operators_reward_amount);
            self.reward_capacity()
                .update(|prev| *prev -= &battle_reward_amount + &operators_reward_amount);

            let battle_stack_len = self.battle_stack().len();
            self.remaining_nfts_in_battle(current_battle)
                .set(battle_stack_len);
            self.total_number_clashes_current_battle()
                .set(battle_stack_len / 2);
            self.has_battle_started(current_battle).set(true);
        }

        let mut amount_of_clashes_done: u64 = 0;
        let mut total_winner_power: u64 = 0;
        let mut rand_source = RandomnessSource::new();
        let mut remaining_nfts = self.remaining_nfts_in_battle(current_battle).get();

        let result = self.run_while_it_has_gas(|| match remaining_nfts {
            0 => LoopOp::Break,
            1 => {
                self.drain_stack();
                remaining_nfts -= 1;
                LoopOp::Break
            }
            _ => {
                let clash_winner_power = self.clash(&mut rand_source, remaining_nfts);
                total_winner_power += clash_winner_power;
                amount_of_clashes_done += 1;
                remaining_nfts -= 2;

                LoopOp::Continue
            }
        });

        self.remaining_nfts_in_battle(current_battle)
            .set(remaining_nfts);

        if amount_of_clashes_done > 0 {
            let rewards_amount =
                self.calculate_clash_operator_rewards(amount_of_clashes_done, current_battle);
            if rewards_amount > 0 {
                self.send().direct_esdt(
                    &self.blockchain().get_caller(),
                    &self.gng_token_id().get(),
                    0,
                    &rewards_amount,
                );
            }
            self.total_battle_winner_power(current_battle)
                .update(|prev| *prev += total_winner_power);
        }

        if result.is_completed() {
            self.current_battle().update(|current| *current += 1);
        }

        MultiValue2::from((result, amount_of_clashes_done))
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(self.is_active(), "Not active");

        let caller = self.blockchain().get_caller();

        let total_rewards = self.get_pending_rewards_for_address(&caller);

        if total_rewards > 0 {
            self.send()
                .direct_esdt(&caller, &self.gng_token_id().get(), 0, &total_rewards);

            self.clear_pending_rewards_for_address(&caller);
            self.stats_for_address(&caller)
                .update(|prev| prev.gng_claimed += total_rewards);
        }
    }

    #[endpoint]
    fn withdraw(&self, tokens: MultiValueEncoded<MultiValue2<TokenIdentifier, Nonce>>) {
        require!(self.is_active(), "Not active");

        self.claim_rewards();

        let caller = self.blockchain().get_caller();
        let amount_tokens = tokens.len();

        let mut output_payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let mut total_power = 0;

        let current_battle = self.current_battle().get();
        let battle_status = self.get_battle_status();
        let remaining_nfts_mapper = self.remaining_nfts_in_battle(current_battle);

        for token in tokens.into_iter() {
            let (token_id, nonce) = token.into_tuple();
            let token_owner = self.nft_owner(&token_id, nonce).get();

            let attributes = self.token_attributes(&token_id, nonce).get();
            total_power += attributes.power as u64;

            require!(
                token_owner == caller
                    && self.staked_for_address(&caller, &token_id).contains(&nonce),
                "Wrong token"
            );

            self.nft_owner(&token_id, nonce).clear();

            self.staked_for_address(&caller, &token_id)
                .swap_remove(&nonce);

            let token_idx = self.battle_stack().get_index(&Token {
                token_id: token_id.clone(),
                nonce,
            });
            if battle_status == BattleStatus::Battle && token_idx <= remaining_nfts_mapper.get() {
                self.battle_stack()
                    .swap_indexes(token_idx, remaining_nfts_mapper.get());
                remaining_nfts_mapper.update(|prev| *prev -= 1);
            }

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
            prev.power -= total_power;
        });

        self.total_nft_engaged()
            .update(|prev| *prev -= amount_tokens as u64);

        self.send().direct_multi(&caller, &output_payments);
    }

    /// We assume there is at least 2 tokens in the stack
    fn clash(&self, rand_source: &mut RandomnessSource, remaining_nfts: usize) -> u64 {
        let first_token = self.get_and_swap_token(rand_source, remaining_nfts);
        let second_token = self.get_and_swap_token(rand_source, remaining_nfts - 1);

        let first_token_attributes =
            self.get_token_attributes(&first_token.token_id, first_token.nonce);
        let second_token_attributes =
            self.get_token_attributes(&second_token.token_id, second_token.nonce);

        match first_token_attributes
            .power
            .cmp(&second_token_attributes.power)
        {
            Ordering::Greater => self.update_rewards_and_emit_event(&first_token, &second_token),
            Ordering::Less => self.update_rewards_and_emit_event(&second_token, &first_token),
            Ordering::Equal => self.handle_tiebreak(
                &first_token,
                &first_token_attributes,
                &second_token,
                &second_token_attributes,
            ),
        }
    }

    /// Get a random token from the battle stack between 1 and remaining_nfts
    /// And swap it with the item at the remaining_nfts position
    fn get_and_swap_token(
        &self,
        rand_source: &mut RandomnessSource,
        remaining_nfts: usize,
    ) -> Token<Self::Api> {
        let battle_stack_mapper = self.battle_stack();

        let random_index = rand_source.next_usize_in_range(1, remaining_nfts + 1);
        let token = battle_stack_mapper.get_by_index(random_index);

        self.battle_stack()
            .swap_indexes(random_index, remaining_nfts);

        token
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
            self.update_rewards_and_emit_event(first_token, second_token)
        } else if first_token.token_id != emidas_token_id
            && second_token.token_id == emidas_token_id
        {
            self.update_rewards_and_emit_event(second_token, first_token)
        } else if first_token.token_id == gnogon_token_id
            && second_token.token_id == validator_token_id
        {
            self.update_rewards_and_emit_event(first_token, second_token)
        } else if first_token.token_id == validator_token_id
            && second_token.token_id == gnogon_token_id
        {
            self.update_rewards_and_emit_event(second_token, first_token)
        } else if first_token.token_id == gnogon_token_id
            && second_token.token_id == gnogon_token_id
        {
            match first_token_attr.heart.cmp(&second_token_attr.heart) {
                Ordering::Greater => self.update_rewards_and_emit_event(first_token, second_token),
                Ordering::Less => self.update_rewards_and_emit_event(second_token, first_token),
                Ordering::Equal => self.emit_draw_event(first_token, second_token),
            }
        } else if first_token.token_id == validator_token_id
            && second_token.token_id == validator_token_id
        {
            match first_token_attr.ram.cmp(&second_token_attr.ram) {
                Ordering::Greater => self.update_rewards_and_emit_event(first_token, second_token),
                Ordering::Less => self.update_rewards_and_emit_event(second_token, first_token),
                Ordering::Equal => self.emit_draw_event(first_token, second_token),
            }
        } else {
            self.emit_draw_event(first_token, second_token)
        }
    }

    fn update_rewards_and_emit_event<'a>(
        &self,
        mut winner: &'a Token<Self::Api>,
        mut loser: &'a Token<Self::Api>,
    ) -> u64 {
        if self.is_current_battle_special() {
            (winner, loser) = (loser, winner);
        }

        let winner_address = self.nft_owner(&winner.token_id, winner.nonce).get();
        let winner_power = self
            .get_token_attributes(&winner.token_id, winner.nonce)
            .power as u64;
        let current_battle = self.current_battle().get();

        let winner_rewards_mapper = self.raw_pending_rewards_for_address(&winner_address);
        let winner_rewards = winner_rewards_mapper.get();
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

    fn emit_draw_event(&self, loser1: &Token<Self::Api>, loser2: &Token<Self::Api>) -> u64 {
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
        let token = self.battle_stack().get_by_index(1);

        //emit event
        self.single_token_clash_event(
            current_battle,
            &token,
            self.nft_owner(&token.token_id, token.nonce).get(),
        );
    }

    fn calculate_clash_rewards(&self, battle_id: u64, power: u64) -> BigUint {
        let total_rewards_for_battle = self.total_rewards_for_stakers(battle_id).get();
        let total_winner_power = self.total_battle_winner_power(battle_id).get();

        if total_winner_power == 0 {
            return BigUint::zero();
        }

        BigUint::from(power)
            .mul(&total_rewards_for_battle)
            .div(total_winner_power)
    }

    /// We assume there is at least one clash (stack length of 2 or more)
    fn calculate_clash_operator_rewards(
        &self,
        amount_of_clashes_performed: u64,
        battle_id: u64,
    ) -> BigUint {
        let total_rewards_for_one_battle = self.total_rewards_for_operators(battle_id).get();
        let total_clashes_amount = (self.total_number_clashes_current_battle().get()) as u64;

        BigUint::from(amount_of_clashes_performed)
            .mul(&total_rewards_for_one_battle)
            .div(total_clashes_amount)
    }

    #[view(getBattleStatus)]
    fn get_battle_status(&self) -> BattleStatus {
        let current_timestamp = self.blockchain().get_block_timestamp();
        let first_battle_timestamp = self.first_battle_timestamp_current_period().get();
        let current_battle = self.current_battle().get();
        let past_battle_amount = self.past_battle_amount().get();

        let battle_duration = match self.battle_mode().get() {
            BattleMode::Daily => ONE_DAY_TIMESTAMP,
            BattleMode::Weekly => ONE_WEEK_TIMESTAMP,
        };

        if current_timestamp
            >= first_battle_timestamp
                + (battle_duration * (current_battle - past_battle_amount - 1))
        {
            BattleStatus::Battle
        } else {
            BattleStatus::Preparation
        }
    }

    /// Returns if whether the current day is Sunday
    #[view(isTodaySpecial)]
    fn is_today_special(&self) -> bool {
        let current_timestamp = self.blockchain().get_block_timestamp();

        self.is_sunday(current_timestamp)
    }

    /// Returns if whether the current battle corresponds to Sunday
    #[view(isCurrentBattleSpecial)]
    fn is_current_battle_special(&self) -> bool {
        match self.battle_mode().get() {
            BattleMode::Daily => {
                let first_battle_timestamp = self.first_battle_timestamp_current_period().get();
                let current_battle = self.current_battle().get();
                let past_battle_amount = self.past_battle_amount().get();
                let current_battle_timestamp = first_battle_timestamp
                    + (ONE_DAY_TIMESTAMP * (current_battle - past_battle_amount - 1));

                self.is_sunday(current_battle_timestamp)
            }
            BattleMode::Weekly => {
                self.is_special_week(self.current_battle().get() - self.past_battle_amount().get())
            }
        }
    }

    fn is_sunday(&self, timestamp: u64) -> bool {
        let days = timestamp / 60 / 60 / 24;
        let weekday = (days + 4) % 7;

        // Sunday is index 0 [sun, mon, tue, wed, thu, fri, sat]
        weekday == 0
    }

    fn is_special_week(&self, current_battle: u64) -> bool {
        current_battle % SPECIAL_WEEK_RECURRENCE == 0
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

    /// Does not include the rewards of the current battle
    #[view(getPendingRewardsForAddress)]
    fn get_pending_rewards_for_address(&self, address: &ManagedAddress) -> BigUint {
        if self.raw_pending_rewards_for_address(address).is_empty() {
            return BigUint::zero();
        }
        let pending_rewards = self.raw_pending_rewards_for_address(address).get();

        if pending_rewards.awaiting_battle_id == self.current_battle().get()
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

    /// Does not clear the rewards of the current battle
    fn clear_pending_rewards_for_address(&self, address: &ManagedAddress) {
        if self.raw_pending_rewards_for_address(address).is_empty() {
            return;
        }

        let mut pending_rewards = self.raw_pending_rewards_for_address(address).get();

        if pending_rewards.awaiting_battle_id == self.current_battle().get() {
            pending_rewards.calculated_rewards = BigUint::zero();
        } else {
            pending_rewards = PendingRewards::default();
        }

        self.raw_pending_rewards_for_address(address)
            .set(&pending_rewards);
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
    fn get_global_stats(
        &self,
        from: usize,
        size: usize,
    ) -> MultiValueEncoded<MultiValue3<ManagedAddress, BigUint, u64>> {
        let mut result = MultiValueEncoded::new();

        for address in self.addresses().iter().skip(from).take(size) {
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

        total_gng += stats_for_address.gng_claimed;
        total_gng += self.get_pending_rewards_for_address(address);

        MultiValue3::from((address.clone(), power, total_gng))
    }

    #[view(getAmountOfUsers)]
    fn get_amount_of_users(&self) -> usize {
        self.addresses().len()
    }

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

    #[view(getRemainingNftsInBattle)]
    #[storage_mapper("remainingNftsInBattle")]
    fn remaining_nfts_in_battle(&self, battle: u64) -> SingleValueMapper<usize>;

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

    #[view(getTotalRewardsForStakers)]
    #[storage_mapper("totalRewardsForStakers")]
    fn total_rewards_for_stakers(&self, battle_id: u64) -> SingleValueMapper<BigUint>;

    #[view(getTotalRewardsForOperators)]
    #[storage_mapper("totalRewardsForOperators")]
    fn total_rewards_for_operators(&self, battle_id: u64) -> SingleValueMapper<BigUint>;

    #[view(getTotalNumberClashesCurrentBattle)]
    #[storage_mapper("totalNumberClashesCurrentBattle")]
    fn total_number_clashes_current_battle(&self) -> SingleValueMapper<usize>;

    #[view(getPastBattleAmount)]
    #[storage_mapper("pastBattleAmount")]
    fn past_battle_amount(&self) -> SingleValueMapper<u64>;
}
