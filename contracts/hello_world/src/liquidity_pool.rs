#![no_std]

use soroban_sdk::{contractimpl, symbol, Address, Env, Map, Symbol};

#[derive(Clone, Default)]
pub struct LiquidityPool {
    pub reserves: Map<Symbol, (i128, i128)>,  // token -> (token reserve, XLM reserve)
}

#[contractimpl]
impl LiquidityPool {
    pub fn add_liquidity(
        env: Env,
        token: Symbol,
        amount_token: i128,
        amount_xlm: i128,
        provider: Address,
    ) {
        let mut pool: LiquidityPool = env.storage().get(symbol!("pool")).unwrap_or_default();
        let (reserve_token, reserve_xlm) = pool.reserves.get(token.clone()).unwrap_or((0, 0));

        pool.reserves.set(
            token.clone(),
            (reserve_token + amount_token, reserve_xlm + amount_xlm),
        );

        env.storage().set(symbol!("pool"), pool);
        env.events().publish((symbol!("LiquidityAdded"), provider, token, amount_token, amount_xlm));
    }

    pub fn swap(env: Env, token: Symbol, xlm_amount: i128) -> i128 {
        let mut pool: LiquidityPool = env.storage().get(symbol!("pool")).unwrap();
        let (reserve_token, reserve_xlm) = pool.reserves.get(token.clone()).unwrap();

        let token_out = (xlm_amount * reserve_token) / (reserve_xlm + xlm_amount);
        assert!(token_out <= reserve_token, "Insufficient liquidity");

        pool.reserves.set(token.clone(), (reserve_token - token_out, reserve_xlm + xlm_amount));
        env.storage().set(symbol!("pool"), pool);

        env.events().publish((symbol!("Swap"), token, xlm_amount, token_out));
        token_out
    }
}
