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
    fn set_battle_token(
        &self,
        tokens: MultiValue5<
            TokenIdentifier,
            TokenIdentifier,
            TokenIdentifier,
            TokenIdentifier,
            TokenIdentifier,
        >,
    ) {
        let (emidas, supreme, gnogon, validator, doga) = tokens.into_tuple();

        self.emidas_token_id().set(emidas.clone());
        self.battle_tokens().insert(emidas);
        self.supreme_token_id().set(supreme.clone());
        self.battle_tokens().insert(supreme);
        self.gnogon_token_id().set(gnogon.clone());
        self.battle_tokens().insert(gnogon);
        self.validator_v2_token_id().set(validator.clone());
        self.battle_tokens().insert(validator);
        self.doga_token_id().set(doga.clone());
        self.battle_tokens().insert(doga);
    }

    #[only_owner]
    #[endpoint(setAttributes)]
    fn set_attributes(
        &self,
        args: MultiValueEncoded<MultiValue5<TokenIdentifier, Nonce, u16, u16, u16>>,
    ) {
        for arg in args.into_iter() {
            let (token, nonce, power, heart, ram) = arg.into_tuple();

            self.token_attributes(&token, nonce)
                .set(Attributes { power, heart, ram });
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

        self.reward_capacity().update(|reward| *reward += amount);
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

    #[only_owner]
    #[endpoint(setDailyRewardAmount)]
    fn set_daily_reward_amount(&self, amount: BigUint) {
        self.daily_reward_amount().set(amount);
    }

    #[only_owner]
    #[endpoint(setBaseBattleRewardAmount)]
    fn set_base_battle_reward_amount(&self, amount: BigUint) {
        self.base_battle_reward_amount().set(amount);
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

    #[view(getEmidasTokenId)]
    #[storage_mapper("emidasTokenId")]
    fn emidas_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getSupremeTokenId)]
    #[storage_mapper("supremeTokenId")]
    fn supreme_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getGnogonTokenId)]
    #[storage_mapper("gnogonTokenId")]
    fn gnogon_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getValidatorV2TokenId)]
    #[storage_mapper("validatorV2TokenId")]
    fn validator_v2_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getDogaTokenId)]
    #[storage_mapper("dogaTokenId")]
    fn doga_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getRewardCapacity)]
    #[storage_mapper("rewardCapacity")]
    fn reward_capacity(&self) -> SingleValueMapper<BigUint>;

    #[view(getDailyRewardAmount)]
    #[storage_mapper("dailyRewardAmount")]
    fn daily_reward_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getBaseBattleRewardAmount)]
    #[storage_mapper("baseBattleRewardAmount")]
    fn base_battle_reward_amount(&self) -> SingleValueMapper<BigUint>;
}
