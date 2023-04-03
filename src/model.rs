multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type Nonce = u64;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct UserStats<M: ManagedTypeApi> {
    pub power: u64,
    pub gng_claimed: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedDecode, NestedEncode, TypeAbi, ManagedVecItem, Clone)]
pub struct Token<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi, Default)]
pub struct Attributes {
    pub power: u16,
    pub heart: u16,
    pub ram: u16,
}

#[derive(PartialEq, TopEncode, TopDecode, TypeAbi)]
pub enum BattleStatus {
    Preparation,
    Battle,
}

#[derive(TopEncode, TopDecode, TypeAbi, NestedEncode, NestedDecode)]
pub struct PendingRewards<M: ManagedTypeApi> {
    pub calculated_rewards: BigUint<M>,
    pub awaiting_battle_id: u64,
    pub awaiting_power: u64,
}

impl<M: ManagedTypeApi> Default for UserStats<M> {
    fn default() -> Self {
        Self {
            power: 0,
            gng_claimed: BigUint::zero(),
        }
    }
}

impl<M: ManagedTypeApi> Default for PendingRewards<M> {
    fn default() -> Self {
        Self {
            calculated_rewards: BigUint::zero(),
            awaiting_battle_id: 0,
            awaiting_power: 0,
        }
    }
}
