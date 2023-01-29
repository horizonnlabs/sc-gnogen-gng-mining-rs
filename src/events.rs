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
}
