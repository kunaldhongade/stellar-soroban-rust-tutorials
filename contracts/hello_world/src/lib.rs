#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, BytesN, Env, Map, Symbol, log,
};

#[contracterror]
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum LumiFiError {
    Unauthorized = 1,
    InsufficientFunds = 2,
    ICOExpired = 3,
    AlreadyInitialized = 4,
    InvalidAmount = 5,
    TokenNotFound = 6,
    ICONotFound = 7,
}

#[contracttype]
pub enum DataKey {
    Token(Address),
    ICO(BytesN<32>),
    User(Address),
    LiquidityPool(Symbol),
}

#[contracttype]
pub struct Token {
    pub total_supply: i128,
    pub balances: Map<Address, i128>,
    pub owner: Address,
}

impl Token {
    pub fn new(env: &Env, owner: Address, initial_supply: i128) -> Self {
        let mut balances = Map::new(env);
        balances.set(owner.clone(), initial_supply);
        Token {
            total_supply: initial_supply,
            balances,
            owner,
        }
    }
}

#[contract]
pub struct LumiFi;

#[contractimpl]
impl LumiFi {
    /// Create a new token and store it in the contract storage
    pub fn create_token(
        env: Env,
        owner: Address,
        initial_supply: i128,
    ) -> Result<Address, LumiFiError> {
        log!(&env, "Starting token creation for owner: {}", owner);

        owner.require_auth();

        if initial_supply < 0 {
            log!(&env, "Invalid initial supply: {}", initial_supply);
            return Err(LumiFiError::InvalidAmount);
        }

        let token = Token::new(&env, owner.clone(), initial_supply);
        env.storage().instance().set(&DataKey::Token(owner.clone()), &token);

        log!(&env, "Token successfully created for owner: {}", owner);
        Ok(owner)
    }

    /// Mint new tokens (only the token owner can call this)
    pub fn mint(env: Env, token_address: Address, amount: i128) -> Result<(), LumiFiError> {
        let mut token: Token = env
            .storage()
            .instance()
            .get(&DataKey::Token(token_address.clone()))
            .ok_or(LumiFiError::TokenNotFound)?;

        token.owner.require_auth();

        token.total_supply += amount;
        let owner_balance = token.balances.get(token.owner.clone()).unwrap_or(0);
        token.balances.set(token.owner.clone(), owner_balance + amount);

        env.storage().instance().set(&DataKey::Token(token_address), &token);
        Ok(())
    }

    /// Start a new ICO
    pub fn start_ico(
        env: Env,
        token: Address,
        target_amount: i128,
        deadline: u64,
    ) -> Result<BytesN<32>, LumiFiError> {
        let ico_id = BytesN::from_array(&env, &[0; 32]);
        env.storage()
            .instance()
            .set(&DataKey::ICO(ico_id.clone()), &(token, target_amount, deadline));
        Ok(ico_id)
    }

    /// Buy tokens during the ICO
    pub fn buy_token(
        env: Env,
        ico_id: BytesN<32>,
        buyer: Address,
        amount: i128,
    ) -> Result<(), LumiFiError> {
        buyer.require_auth();
        if amount <= 0 {
            return Err(LumiFiError::InvalidAmount);
        }

        let (token, _, deadline): (Address, i128, u64) = env
            .storage()
            .instance()
            .get(&DataKey::ICO(ico_id.clone()))
            .ok_or(LumiFiError::ICONotFound)?;

        if env.ledger().timestamp() > deadline {
            return Err(LumiFiError::ICOExpired);
        }

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        let mut buyer_balance = env
            .storage()
            .instance()
            .get::<_, i128>(&DataKey::User(buyer.clone()))
            .unwrap_or(0);
        buyer_balance += amount;
        env.storage().instance().set(&DataKey::User(buyer), &buyer_balance);

        Ok(())
    }

    /// Withdraw tokens after the ICO ends
    pub fn withdraw(
        env: Env,
        token: Address,
        recipient: Address,
        amount: i128,
    ) -> Result<(), LumiFiError> {
        recipient.require_auth();
        let token_client = token::Client::new(&env, &token);
        let contract_balance = token_client.balance(&env.current_contract_address());

        if amount > contract_balance {
            return Err(LumiFiError::InsufficientFunds);
        }

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);
        Ok(())
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        env: Env,
        pool_symbol: Symbol,
        provider: Address,
        amount_token: i128,
        amount_xlm: i128,
    ) -> Result<(), LumiFiError> {
        provider.require_auth();

        if amount_token <= 0 || amount_xlm <= 0 {
            return Err(LumiFiError::InvalidAmount);
        }

        let mut pool: Map<Symbol, (i128, i128)> = env
            .storage()
            .instance()
            .get(&DataKey::LiquidityPool(pool_symbol.clone()))
            .unwrap_or(Map::new(&env));

        let (token_reserve, xlm_reserve) = pool.get(pool_symbol.clone()).unwrap_or((0, 0));
        pool.set(
            pool_symbol.clone(),
            (token_reserve + amount_token, xlm_reserve + amount_xlm),
        );

        env.storage()
            .instance()
            .set(&DataKey::LiquidityPool(pool_symbol), &pool);
        Ok(())
    }

    /// Swap tokens using the liquidity pool
    pub fn swap(
        env: Env,
        pool_symbol: Symbol,
        amount_xlm: i128,
    ) -> Result<i128, LumiFiError> {
        let mut pool: Map<Symbol, (i128, i128)> = env
            .storage()
            .instance()
            .get(&DataKey::LiquidityPool(pool_symbol.clone()))
            .ok_or(LumiFiError::TokenNotFound)?;

        let (token_reserve, xlm_reserve) = pool.get(pool_symbol.clone()).unwrap();

        let token_out = (amount_xlm * token_reserve) / (xlm_reserve + amount_xlm);
        if token_out > token_reserve {
            return Err(LumiFiError::InsufficientFunds);
        }

        pool.set(
            pool_symbol.clone(),
            (token_reserve - token_out, xlm_reserve + amount_xlm),
        );
        env.storage()
            .instance()
            .set(&DataKey::LiquidityPool(pool_symbol), &pool);

        Ok(token_out)
    }
}
