#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct BattleStats {
    win: u64,
    loss: u64
}

// add battle status enum

pub type Nonce = u64;

#[elrond_wasm::contract]
pub trait GngMinting {
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

    // token that can participate in the battle
    #[only_owner]
    #[endpoint(setBattleToken)]
    fn set_battle_token(&self, tokens: MultiValueEncoded<TokenIdentifier>) {
        for token in tokens.into_iter() {
            self.battle_tokens().insert(token);
        }
    }

    #[only_owner]
    #[endpoint(setPowerAndHeartScores)]
    fn set_power_and_heart_scores(
        &self,
        args: MultiValueEncoded<MultiValue4<TokenIdentifier, Nonce, u64, u64>>,
    ) {
        for arg in args.into_iter() {
            let (token, nonce, power, heart) = arg.into_tuple();

            todo!()
        }
    }

    #[view(getCurrentBattle)]
    #[storage_mapper("currentBattle")]
    fn current_battle(&self) -> SingleValueMapper<u64>;

    #[view(getScoresForAddress)]
    #[storage_mapper("getScoresForAddress")]
    fn get_scores_for_address(&self, address: &ManagedAddress) -> SingleValueMapper<BattleStats>;

    // I recommend to do a struct rather than a tuple
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

    #[storage_mapper("BattleTokens")]
    fn battle_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
