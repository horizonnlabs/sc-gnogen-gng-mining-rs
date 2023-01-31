elrond_wasm::imports!();

use crate::model::{ClashEventStruct, Token};

#[elrond_wasm::module]
trait EventsModule {
    #[event("clashes")]
    fn clashes_event(
        &self,
        #[indexed] battle_id: u64,
        data: ManagedVec<ClashEventStruct<Self::Api>>,
    );

    #[event("clash")]
    fn clash_event(
        &self,
        #[indexed] battle_id: u64,
        #[indexed] winner: &Token<Self::Api>,
        #[indexed] loser: &Token<Self::Api>,
        #[indexed] is_draw: bool,
        #[indexed] winner_owner: &ManagedAddress,
        #[indexed] loser_owner: &ManagedAddress,
    );

    #[event("singleTokenClash")]
    fn single_token_clash_event(
        &self,
        #[indexed] battle_id: u64,
        #[indexed] token: &Token<Self::Api>,
        #[indexed] owner: ManagedAddress,
    );

    #[event("startBattle")]
    fn start_battle_event(&self, #[indexed] battle_id: u64);

    #[event("endBattle")]
    fn end_battle_event(&self, #[indexed] battle_id: u64);
}
