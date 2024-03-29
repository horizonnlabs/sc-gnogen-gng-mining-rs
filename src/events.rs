multiversx_sc::imports!();

use crate::model::Token;

#[multiversx_sc::module]
trait EventsModule {
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
}
