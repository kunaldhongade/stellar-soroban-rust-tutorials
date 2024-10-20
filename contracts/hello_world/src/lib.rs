#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, log, Address, BytesN, Env, Map, Symbol, String};

#[contracttype]
#[derive(Clone, Default)]
pub struct Token {
    pub total_supply: i128,
    pub balances: Map<Address, i128>,
    pub owner: Address,
}

#[contracttype]
pub enum TokenRegistry {
    Token(Address),
}

#[contracttype]
pub struct ICO {
    pub token: BytesN<32>,
    pub price: i128,
    pub remaining_supply: i128,
    pub cap: i128,
    pub raised: i128,
}

#[contracttype]
pub enum ICORegistry {
    Campaign(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Default)]
pub struct LiquidityPool {
    pub reserves: Map<Symbol, (i128, i128)>, // token -> (token reserve, XLM reserve)
    pub lp_tokens: Map<Address, i128>,       // LP provider -> LP token balance
}

#[contracttype]
pub enum LiquidityRegistry {
    Pool(Symbol),
}

#[contract]
pub struct LumiFi;

#[contractimpl]
impl LumiFi {
    // 1️⃣ Create a new token
    pub fn create_token(
        env: Env,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: i128,
        owner: Address,
    ) -> BytesN<32> {
        let token_id = BytesN::from_array(&env, &[0; 32]); // Replace with actual WASM hash logic
        let mut token = Token::default();
        token.total_supply = initial_supply;
        token.owner = owner.clone();
        token.balances.set(owner, initial_supply);

        env.storage().instance().set(&TokenRegistry::Token(owner.clone()), &token);
        log!(&env, "Token {} created by {}", symbol, owner);
        token_id
    }

    // 2️⃣ Mint tokens (only owner)
    pub fn mint(env: Env, token: Address, amount: i128) {
        let mut token_data: Token = env.storage().instance().get(&TokenRegistry::Token(token.clone())).unwrap();
        assert!(env.invoker() == token_data.owner, "Only owner can mint tokens.");

        let owner_balance = token_data.balances.get(token_data.owner.clone()).unwrap_or(0);
        token_data.total_supply += amount;
        token_data.balances.set(token_data.owner.clone(), owner_balance + amount);

        env.storage().instance().set(&TokenRegistry::Token(token), &token_data);
        log!(&env, "Minted {} tokens", amount);
    }

    // 3️⃣ Burn tokens (only owner)
    pub fn burn(env: Env, token: Address, amount: i128) {
        let mut token_data: Token = env.storage().instance().get(&TokenRegistry::Token(token.clone())).unwrap();
        assert!(env.invoker() == token_data.owner, "Only owner can burn tokens.");

        let owner_balance = token_data.balances.get(token_data.owner.clone()).unwrap_or(0);
        assert!(owner_balance >= amount, "Insufficient balance to burn.");

        token_data.total_supply -= amount;
        token_data.balances.set(token_data.owner.clone(), owner_balance - amount);

        env.storage().instance().set(&TokenRegistry::Token(token), &token_data);
        log!(&env, "Burned {} tokens", amount);
    }

    // 4️⃣ Start an ICO campaign
    pub fn start_ico(env: Env, token: BytesN<32>, price: i128, supply: i128, cap: i128) {
        let ico = ICO { token: token.clone(), price, remaining_supply: supply, cap, raised: 0 };
        env.storage().instance().set(&ICORegistry::Campaign(token.clone()), &ico);
        log!(&env, "ICO started for token {:?}", token);
    }

    // 5️⃣ Buy tokens in ICO with slippage control
    pub fn buy_token(env: Env, token: BytesN<32>, buyer: Address, xlm_amount: i128, slippage: i128) -> i128 {
        let mut ico: ICO = env.storage().instance().get(&ICORegistry::Campaign(token.clone())).unwrap();
        let token_amount = xlm_amount / ico.price;
        assert!(ico.remaining_supply >= token_amount, "Not enough tokens available.");
        assert!(ico.raised + xlm_amount <= ico.cap, "ICO cap reached.");

        let allowed_slippage = token_amount * (100 + slippage) / 100;
        assert!(allowed_slippage >= token_amount, "Slippage too high.");

        ico.remaining_supply -= token_amount;
        ico.raised += xlm_amount;
        env.storage().instance().set(&ICORegistry::Campaign(token.clone()), &ico);

        log!(&env, "{} tokens bought by {}", token_amount, buyer);
        token_amount
    }
}
