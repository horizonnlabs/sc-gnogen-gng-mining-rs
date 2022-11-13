#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod config;
mod model;

use config::State;
use model::{BattleStats, BattleStatus, Nonce, Token};

#[elrond_wasm::contract]
pub trait GngMinting: config::ConfigModule {
    #[init]
    fn init(&self) {
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
            require!(
                self.battle_tokens().contains(&token_id) && amount == 1,
                "Wrong token"
            );

            self.staked_for_address(&caller, &token_id).insert(nonce);
            if self.first_stack().len() > self.second_stack().len() {
                self.second_stack().push(&Token { token_id, nonce });
            } else {
                self.first_stack().push(&Token { token_id, nonce });
            }
        }
    }

    #[endpoint]
    fn battle(&self) {
        todo!()
    }

    // perhaps we can "customize" this endpoint and call it something like "claimGng" or "claimGngRewards"
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        todo!()
    }

    fn calculate_rewards(&self) {
        todo!()
    }

    #[endpoint]
    fn withdraw(&self, tokens: MultiValueEncoded<MultiValue2<TokenIdentifier, Nonce>>) {
        for token in tokens.into_iter() {
            let (_token_id, _nonce) = token.into_tuple();
            todo!()
        }
    }

    #[view(getBattleStatus)]
    fn get_battle_status(&self) -> BattleStatus {
        //TODO
        BattleStatus::Preparation
    }

    #[view(getCurrentBattle)]
    #[storage_mapper("currentBattle")]
    fn current_battle(&self) -> SingleValueMapper<u64>;

    #[view(getScoresForAddress)]
    #[storage_mapper("getScoresForAddress")]
    fn get_scores_for_address(&self, address: &ManagedAddress) -> SingleValueMapper<BattleStats>;

    #[view(getScoresForNft)]
    #[storage_mapper("getScoresForNft")]
    fn get_scores_for_nft(
        &self,
        token_id: &TokenIdentifier,
        nonce: Nonce,
    ) -> SingleValueMapper<BattleStats>;

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
    fn first_stack(&self) -> VecMapper<Token<Self::Api>>;

    #[view(getSecondStack)]
    #[storage_mapper("secondStack")]
    fn second_stack(&self) -> VecMapper<Token<Self::Api>>;
}
