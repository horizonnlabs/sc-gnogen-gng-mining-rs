elrond_wasm::imports!();

use crate::model::Token;

#[elrond_wasm::module]
trait EventsModule {
    #[event("clash")]
    fn clash_event(
        &self,
        #[indexed] winner: &Token<Self::Api>,
        #[indexed] loser: &Token<Self::Api>,
        #[indexed] is_draw: bool,
    );

    #[event("startBattle")]
    fn start_battle_event(&self, #[indexed] battle_id: u64);

    #[event("endBattle")]
    fn end_battle_event(&self, #[indexed] battle_id: u64);
}
