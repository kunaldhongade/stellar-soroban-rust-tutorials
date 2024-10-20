#![no_std]

use soroban_sdk::{contractimpl, symbol, Address, Env, Map, Symbol};

#[derive(Clone, Default)]
pub struct Token {
    pub total_supply: i128,
    pub balances: Map<Address, i128>,
    pub owner: Address,
}

#[contractimpl]
impl Token {
    pub fn init(
        env: Env,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: i128,
        owner: Address,
    ) {
        let mut token = Token::default();
        token.total_supply = initial_supply;
        token.owner = owner.clone();
        token.balances.set(owner, initial_supply);
        env.storage().set(symbol!("token"), token);

        env.events().publish((symbol!("Initialized"), name, symbol, initial_supply));
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let mut token: Token = env.storage().get(symbol!("token")).unwrap();
        assert!(env.invoker() == token.owner, "Only owner can mint");

        let balance = token.balances.get(to.clone()).unwrap_or(0);
        token.balances.set(to, balance + amount);
        token.total_supply += amount;

        env.storage().set(symbol!("token"), token);
        env.events().publish((symbol!("Mint"), to, amount));
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        let mut token: Token = env.storage().get(symbol!("token")).unwrap();
        let balance = token.balances.get(from.clone()).unwrap_or(0);
        assert!(balance >= amount, "Insufficient balance");

        token.balances.set(from, balance - amount);
        token.total_supply -= amount;

        env.storage().set(symbol!("token"), token);
        env.events().publish((symbol!("Burn"), from, amount));
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        let mut token: Token = env.storage().get(symbol!("token")).unwrap();
        let from_balance = token.balances.get(from.clone()).unwrap_or(0);
        assert!(from_balance >= amount, "Insufficient balance");

        token.balances.set(from, from_balance - amount);
        let to_balance = token.balances.get(to.clone()).unwrap_or(0);
        token.balances.set(to, to_balance + amount);

        env.storage().set(symbol!("token"), token);
        env.events().publish((symbol!("Transfer"), from, to, amount));
    }
}
