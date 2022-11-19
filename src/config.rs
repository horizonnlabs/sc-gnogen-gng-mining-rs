elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, PartialEq, TypeAbi)]
pub enum State {
    Inactive,
    Active,
}

use super::model::{Attributes, Nonce};

#[elrond_wasm::module]
pub trait ConfigModule {
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
        args: MultiValueEncoded<MultiValue4<TokenIdentifier, Nonce, u16, u16>>,
    ) {
        for arg in args.into_iter() {
            let (token, nonce, power, heart) = arg.into_tuple();

            self.token_attributes(&token, nonce)
                .set(Attributes { power, heart });
        }
    }

    #[only_owner]
    #[endpoint]
    fn pause(&self) {
        self.state().set(&State::Inactive);
    }

    #[only_owner]
    #[endpoint]
    fn resume(&self) {
        self.state().set(&State::Active);
    }

    #[payable("*")]
    #[endpoint(depositGng)]
    fn deposit_gng(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.admin().contains(&caller),
            "Only admin can call this endpoint"
        );

        let (token, amount) = self.call_value().single_fungible_esdt();
        let gng_token_id = self.gng_token_id().get();
        require!(token == gng_token_id, "Invalid token sent");

        self.reward_vault().update(|reward| *reward += amount);
    }

    #[inline]
    fn is_active(&self) -> bool {
        self.state().get() == State::Active
    }

    #[only_owner]
    #[endpoint(addAdmin)]
    fn add_admin(&self, admin: ManagedAddress) {
        let is_new = self.admin().insert(admin);
        require!(is_new, "Address is already an admin");
    }

    #[only_owner]
    #[endpoint(removeAdmin)]
    fn remove_admin(&self, admin: ManagedAddress) {
        let is_removed = self.admin().swap_remove(&admin);
        require!(is_removed, "Address is not an admin");
    }

    #[storage_mapper("admin")]
    fn admin(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getBattleTokens)]
    #[storage_mapper("battleTokens")]
    fn battle_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[view(getTokenAttributes)]
    #[storage_mapper("tokenAttributes")]
    fn token_attributes(
        &self,
        token_id: &TokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<Attributes>;

    #[view(getGngTokenId)]
    #[storage_mapper("gngTokenId")]
    fn gng_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("rewardCapacity")]
    fn reward_vault(&self) -> SingleValueMapper<BigUint>;
}
