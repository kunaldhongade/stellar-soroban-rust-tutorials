#![no_std]

use soroban_sdk::{contractimpl, symbol, BytesN, Env, Address};

#[derive(Clone)]
pub struct TokenFactory;

#[contractimpl]
impl TokenFactory {
    pub fn create_token(
        env: Env,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: i128,
        owner: Address,
    ) -> BytesN<32> {
        let token_wasm_hash = BytesN::from_array(&env, &[0u8; 32]); // Replace with actual WASM hash

        let token_id = env.deploy_contract(
            &token_wasm_hash,
            symbol!("init"),
            (name, symbol, decimals, initial_supply, owner.clone()),
        );

        token_id
    }
}
