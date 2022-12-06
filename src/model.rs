elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type Nonce = u64;

#[derive(TopEncode, TopDecode, TypeAbi, Default)]
pub struct UserStats {
    pub win: u64,
    pub loss: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct TokenStats<M: ManagedTypeApi> {
    pub win: u64,
    pub loss: u64,
    pub owner: ManagedAddress<M>,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Token<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
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
pub struct BattleHistory {
    pub battle_id: u64,
    pub total_winner_power: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi, NestedEncode, NestedDecode)]
pub struct PendingRewards {
    pub battle_id: u64,
    pub power: u64,
}

impl<M: ManagedTypeApi> Default for TokenStats<M> {
    fn default() -> Self {
        Self {
            win: 0,
            loss: 0,
            owner: ManagedAddress::zero(),
        }
    }
}
