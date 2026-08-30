use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token::StellarAssetClient, token::TokenClient, Address, Env, InvokeError};

const FEE_BPS: u32 = 30; // 0.30%
const POOL_LIQUIDITY: i128 = 1_000_000;
const BORROWER_RESERVES: i128 = 10_000;
const LOAN: i128 = 100_000;
const LOAN_FEE: i128 = 300; // 30 bps of 100_000

/// The pool reports failures by panicking with a [`Error`] variant, so a
/// `try_` call surfaces it as a raw host error rather than the enum. This
/// turns a variant into the value to compare against.
fn contract_err(error: Error) -> Result<soroban_sdk::Error, InvokeError> {
    Ok(soroban_sdk::Error::from_contract_error(error as u32))
}

/// Storage keys for the example borrowers below.
#[contracttype]
pub enum BorrowerKey {
    Pool,
    SeenBalance,
}

/// A well-behaved borrower: repays principal + fee out of its own reserves.
///
/// This is the shape a real borrower takes — the arbitrage or liquidation that
/// makes the loan worth taking would go where the comment is.
#[contract]
pub struct GoodBorrower;

#[contractimpl]
impl GoodBorrower {
    pub fn __constructor(env: Env, pool: Address) {
        env.storage().instance().set(&BorrowerKey::Pool, &pool);
    }

    pub fn exec(env: Env, pool: Address, token: Address, amount: i128, fee: i128) {
        // `exec` is public: anyone can call it with arguments they chose. Two
        // checks make that harmless. First, the pool must be the one we were
        // deployed against — a caller cannot point us at their own "pool".
        let expected: Address = env.storage().instance().get(&BorrowerKey::Pool).unwrap();
        assert_eq!(pool, expected, "callback from an unexpected pool");
        // Second, the pool must actually be the caller. A contract address's
        // authorization is satisfied by it being the direct caller, so this
        // holds during a real loan and fails for a direct call by anyone else.
        pool.require_auth();

        let client = token::Client::new(&env, &token);
        let me = env.current_contract_address();

        // Record what we hold mid-loan so a test can prove the funds really
        // arrived before the pool's repayment check ran.
        env.storage()
            .instance()
            .set(&BorrowerKey::SeenBalance, &client.balance(&me));

        // ... put the strategy here: arbitrage, liquidation, collateral swap.

        client.transfer(&me, &pool, &(amount + fee));
    }

    pub fn seen_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&BorrowerKey::SeenBalance)
            .unwrap_or(0)
    }
}

/// A borrower that keeps the money. Every loan to it must revert.
#[contract]
pub struct BadBorrower;

#[contractimpl]
impl BadBorrower {
    pub fn exec(_env: Env, _pool: Address, _token: Address, _amount: i128, _fee: i128) {
        // Deliberately does nothing: the borrowed funds stay right here.
    }
}

/// Returns the principal but pockets the fee — repayment must be exact.
#[contract]
pub struct StingyBorrower;

#[contractimpl]
impl StingyBorrower {
    pub fn exec(env: Env, pool: Address, token: Address, amount: i128, _fee: i128) {
        let client = token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &pool, &amount);
    }
}

/// Tries to take a second loan while still inside the first one's callback.
#[contract]
pub struct ReentrantBorrower;

#[contractimpl]
impl ReentrantBorrower {
    pub fn exec(env: Env, pool: Address, token: Address, amount: i128, fee: i128) {
        let me = env.current_contract_address();
        FlashLoanPoolClient::new(&env, &pool).flash_loan(&me, &amount);
        let client = token::Client::new(&env, &token);
        client.transfer(&me, &pool, &(amount + fee));
    }
}

struct Fixture<'a> {
    pool: FlashLoanPoolClient<'a>,
    pool_id: Address,
    token: TokenClient<'a>,
    token_id: Address,
    admin: Address,
}

fn setup(env: &Env) -> Fixture<'_> {
    env.mock_all_auths();

    let issuer = Address::generate(env);
    let token_id = env.register_stellar_asset_contract_v2(issuer).address();
    let admin = Address::generate(env);
    let pool_id = env.register(FlashLoanPool, (admin.clone(), token_id.clone(), FEE_BPS));

    StellarAssetClient::new(env, &token_id).mint(&pool_id, &POOL_LIQUIDITY);

    Fixture {
        pool: FlashLoanPoolClient::new(env, &pool_id),
        pool_id,
        token: TokenClient::new(env, &token_id),
        token_id,
        admin,
    }
}

/// Register a borrower contract and give it reserves to pay fees from.
fn register_borrower(env: &Env, fx: &Fixture, funded: bool) -> Address {
    let id = env.register(GoodBorrower, (fx.pool_id.clone(),));
    if funded {
        StellarAssetClient::new(env, &fx.token_id).mint(&id, &BORROWER_RESERVES);
    }
    id
}

#[test]
fn repaying_borrower_completes_the_loan() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = register_borrower(&env, &fx, true);

    assert_eq!(fx.pool.fee_for(&LOAN), LOAN_FEE);
    fx.pool.flash_loan(&borrower, &LOAN);

    // The pool is up exactly one fee; the borrower paid it out of reserves.
    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY + LOAN_FEE);
    assert_eq!(fx.token.balance(&borrower), BORROWER_RESERVES - LOAN_FEE);
}

#[test]
fn borrower_holds_the_funds_during_the_callback() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = register_borrower(&env, &fx, true);

    fx.pool.flash_loan(&borrower, &LOAN);

    // The principal really was in the borrower's hands mid-call — the loan is
    // not an accounting trick.
    let seen = GoodBorrowerClient::new(&env, &borrower).seen_balance();
    assert_eq!(seen, BORROWER_RESERVES + LOAN);
}

#[test]
#[should_panic]
fn non_repaying_borrower_reverts() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = env.register(BadBorrower, ());

    fx.pool.flash_loan(&borrower, &LOAN);
}

#[test]
fn non_repaying_borrower_leaves_no_trace() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = env.register(BadBorrower, ());

    // Same failure as above, caught so we can inspect the aftermath.
    assert_eq!(
        fx.pool.try_flash_loan(&borrower, &LOAN),
        Err(contract_err(Error::NotRepaid))
    );

    // The transfer that funded the loan was rolled back with the panic: the
    // borrower never got to keep a stroop. This is the whole security model.
    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY);
    assert_eq!(fx.token.balance(&borrower), 0);
}

#[test]
fn partial_repayment_reverts() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = env.register(StingyBorrower, ());
    StellarAssetClient::new(&env, &fx.token_id).mint(&borrower, &BORROWER_RESERVES);

    // Returning the principal is not enough — the fee is part of repayment.
    assert_eq!(
        fx.pool.try_flash_loan(&borrower, &LOAN),
        Err(contract_err(Error::NotRepaid))
    );
    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY);
    assert_eq!(fx.token.balance(&borrower), BORROWER_RESERVES);
}

#[test]
fn back_to_back_loans_accumulate_fees() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = register_borrower(&env, &fx, true);

    fx.pool.flash_loan(&borrower, &LOAN);
    fx.pool.flash_loan(&borrower, &LOAN);

    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY + 2 * LOAN_FEE);
    assert_eq!(
        fx.token.balance(&borrower),
        BORROWER_RESERVES - 2 * LOAN_FEE
    );
}

#[test]
fn zero_amount_is_rejected() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = register_borrower(&env, &fx, true);

    assert_eq!(
        fx.pool.try_flash_loan(&borrower, &0),
        Err(contract_err(Error::InvalidAmount))
    );
}

#[test]
fn loan_larger_than_liquidity_is_rejected() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = register_borrower(&env, &fx, true);

    assert_eq!(
        fx.pool.try_flash_loan(&borrower, &(POOL_LIQUIDITY + 1)),
        Err(contract_err(Error::InsufficientLiquidity))
    );
    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY);
}

#[test]
fn fee_is_rounded_up() {
    let env = Env::default();
    let fx = setup(&env);

    // 30 bps of 1 unit is 0.003 — rounded in the pool's favour, never to zero.
    assert_eq!(fx.pool.fee_for(&1), 1);
    assert_eq!(fx.pool.fee_for(&10_000), 30);
    assert_eq!(fx.pool.fee_for(&10_001), 31);
}

#[test]
fn deposit_and_withdraw_move_liquidity() {
    let env = Env::default();
    let fx = setup(&env);
    let lender = Address::generate(&env);
    StellarAssetClient::new(&env, &fx.token_id).mint(&lender, &50_000);

    fx.pool.deposit(&lender, &50_000);
    assert_eq!(fx.pool.balance(), POOL_LIQUIDITY + 50_000);

    fx.pool.withdraw(&fx.admin, &20_000);
    assert_eq!(fx.pool.balance(), POOL_LIQUIDITY + 30_000);
    assert_eq!(fx.token.balance(&fx.admin), 20_000);
}

#[test]
#[should_panic]
fn constructor_rejects_a_fee_above_the_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let issuer = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(issuer).address();
    let admin = Address::generate(&env);

    env.register(FlashLoanPool, (admin, token_id, MAX_FEE_BPS + 1));
}

#[test]
fn reentering_the_pool_during_the_callback_is_rejected() {
    use soroban_sdk::xdr::{ScErrorCode, ScErrorType};

    let env = Env::default();
    let fx = setup(&env);
    let borrower = env.register(ReentrantBorrower, ());
    StellarAssetClient::new(&env, &fx.token_id).mint(&borrower, &BORROWER_RESERVES);

    // Note the error: `Context, InvalidAction` is a *host* error, not one of
    // this contract's variants. Soroban refuses to call a contract already on
    // the call stack, so the pool cannot be re-entered mid-callback — no
    // reentrancy guard of our own is involved. Do not carry that assumption
    // to platforms that lack it.
    assert_eq!(
        fx.pool.try_flash_loan(&borrower, &LOAN),
        Err(Ok(soroban_sdk::Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY);
}

#[test]
fn the_loan_path_needs_no_external_authorization() {
    let env = Env::default();
    let fx = setup(&env);
    let borrower = register_borrower(&env, &fx, true);

    // Drop the mocked authorizations `setup` installed: nothing in the loan
    // path needs a signature. The pool moves its own funds because it is the
    // caller of the token contract, and the `pool.require_auth()` inside the
    // borrower's callback is satisfied by the pool being the direct caller.
    // That is what makes the check in `GoodBorrower::exec` a real guard
    // rather than decoration.
    env.set_auths(&[]);
    fx.pool.flash_loan(&borrower, &LOAN);

    assert_eq!(fx.token.balance(&fx.pool_id), POOL_LIQUIDITY + LOAN_FEE);
}
