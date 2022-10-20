elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, PartialEq, TypeAbi)]
pub enum State {
    Inactive,
    Active,
}

use super::model::Nonce;

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
        args: MultiValueEncoded<MultiValue4<TokenIdentifier, Nonce, u64, u64>>,
    ) {
        for arg in args.into_iter() {
            let (token, nonce, power, heart) = arg.into_tuple();

            todo!()
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

    #[inline]
    fn is_active(&self) -> bool {
        self.state().get() == State::Active
    }

    #[storage_mapper("battleTokens")]
    fn battle_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;
}