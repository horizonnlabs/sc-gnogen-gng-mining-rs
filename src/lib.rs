#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod config;
mod model;

use model::{Nonce, BattleStats};

#[elrond_wasm::contract]
pub trait GngMinting: config::ConfigModule {
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint]
    fn stake(&self) {
        let payments = self.call_value().all_esdt_transfers();
        todo!()
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
            let (token_id, nonce) = token.into_tuple();
            todo!()
        }
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
}
