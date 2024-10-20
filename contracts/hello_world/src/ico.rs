#![no_std]

use soroban_sdk::{contractimpl, symbol, Address, Env, Symbol};

#[derive(Clone, Default)]
pub struct ICO {
    pub price: i128,
    pub supply: i128,
    pub token: Symbol,
}

#[contractimpl]
impl ICO {
    pub fn init(env: Env, token: Symbol, price: i128, supply: i128) {
        let ico = ICO { price, supply, token: token.clone() };
        env.storage().set(symbol!("ico"), ico);

        env.events().publish((symbol!("ICOInitialized"), token, price, supply));
    }

    pub fn buy(env: Env, buyer: Address, xlm_amount: i128) -> i128 {
        let mut ico: ICO = env.storage().get(symbol!("ico")).unwrap();
        let token_amount = xlm_amount / ico.price;
        assert!(ico.supply >= token_amount, "Not enough tokens");

        ico.supply -= token_amount;
        env.storage().set(symbol!("ico"), ico);

        env.events().publish((symbol!("TokenBought"), buyer, token_amount));
        token_amount
    }
}
