elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type Nonce = u64;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct UserStats<M: ManagedTypeApi> {
    pub win: u64,
    pub loss: u64,
    pub rewards: BigUint<M>,
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
}

#[derive(PartialEq, TopEncode, TopDecode, TypeAbi)]
pub enum BattleStatus {
    Preparation,
    Battle,
}

// add battle status enum
