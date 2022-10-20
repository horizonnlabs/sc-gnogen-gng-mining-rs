elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type Nonce = u64;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct BattleStats {
    pub win: u64,
    pub loss: u64
}

// add battle status enum