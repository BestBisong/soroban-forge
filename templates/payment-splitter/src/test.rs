extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token::StellarAssetClient, token::TokenClient, Address, Env};

struct Tok<'a> {
    address: Address,
    client: TokenClient<'a>,
    admin: StellarAssetClient<'a>,
}

fn make_token(env: &Env) -> Tok<'_> {
    let issuer = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let address = sac.address();
    Tok {
        client: TokenClient::new(env, &address),
        admin: StellarAssetClient::new(env, &address),
        address,
    }
}

/// A splitter with one payee per entry in `shares`, plus a funded depositor.
struct Fixture<'a> {
    contract: PaymentSplitterContractClient<'a>,
    contract_id: Address,
    token: Tok<'a>,
    funder: Address,
    payees: Vec<Address>,
}

fn setup<'a>(env: &'a Env, shares: &[u32], funder_balance: i128) -> Fixture<'a> {
    env.mock_all_auths();

    let token = make_token(env);
    let funder = Address::generate(env);
    token.admin.mint(&funder, &funder_balance);

    let mut payees = Vec::new(env);
    let mut share_vec = Vec::new(env);
    for share in shares {
        payees.push_back(Address::generate(env));
        share_vec.push_back(*share);
    }

    let contract_id = env.register(PaymentSplitterContract, ());
    let contract = PaymentSplitterContractClient::new(env, &contract_id);
    contract.initialize(&token.address, &payees, &share_vec);

    Fixture {
        contract,
        contract_id,
        token,
        funder,
        payees,
    }
}

#[test]
fn splits_deposits_proportionally() {
    let env = Env::default();
    let f = setup(&env, &[50, 30, 20], 1_000);

    f.contract.deposit(&f.funder, &1_000);

    let a = f.payees.get_unchecked(0);
    let b = f.payees.get_unchecked(1);
    let c = f.payees.get_unchecked(2);

    assert_eq!(f.contract.total_received(), 1_000);
    assert_eq!(f.contract.total_shares(), 100);
    assert_eq!(f.contract.releasable(&a), 500);
    assert_eq!(f.contract.releasable(&b), 300);
    assert_eq!(f.contract.releasable(&c), 200);
    assert_eq!(f.contract.undistributed(), 0);

    assert_eq!(f.contract.release(&a), 500);
    assert_eq!(f.contract.release(&b), 300);
    assert_eq!(f.contract.release(&c), 200);

    assert_eq!(f.token.client.balance(&a), 500);
    assert_eq!(f.token.client.balance(&b), 300);
    assert_eq!(f.token.client.balance(&c), 200);
    assert_eq!(f.contract.total_released(), 1_000);
    assert_eq!(f.contract.releasable(&a), 0);
    // Everything was distributed, so nothing is left in the splitter.
    assert_eq!(f.token.client.balance(&f.contract_id), 0);
}

#[test]
fn rounding_dust_stays_until_it_is_a_whole_unit() {
    let env = Env::default();
    // Three equal shares over 100 units: 33 each, one unit of dust.
    let f = setup(&env, &[1, 1, 1], 200);

    f.contract.deposit(&f.funder, &100);

    for payee in f.payees.iter() {
        assert_eq!(f.contract.releasable(&payee), 33);
    }
    assert_eq!(f.contract.undistributed(), 1);

    for payee in f.payees.iter() {
        assert_eq!(f.contract.release(&payee), 33);
        assert_eq!(f.token.client.balance(&payee), 33);
    }
    assert_eq!(f.contract.total_released(), 99);
    // The undistributed unit is still held by the contract, not lost.
    assert_eq!(f.token.client.balance(&f.contract_id), 1);

    // A later deposit pushes each entitlement over the next whole unit and
    // the dust becomes claimable.
    f.contract.deposit(&f.funder, &2);
    assert_eq!(f.contract.total_received(), 102);
    for payee in f.payees.iter() {
        assert_eq!(f.contract.releasable(&payee), 1);
        assert_eq!(f.contract.release(&payee), 1);
        assert_eq!(f.token.client.balance(&payee), 34);
    }
    assert_eq!(f.contract.undistributed(), 0);
    assert_eq!(f.token.client.balance(&f.contract_id), 0);
}

#[test]
fn entitlements_accrue_across_deposits() {
    let env = Env::default();
    let f = setup(&env, &[3, 1], 400);
    let big = f.payees.get_unchecked(0);
    let small = f.payees.get_unchecked(1);

    f.contract.deposit(&f.funder, &100);
    assert_eq!(f.contract.release(&big), 75);
    assert_eq!(f.contract.released(&big), 75);

    // A second deposit only ever adds to what is still owed.
    f.contract.deposit(&f.funder, &200);
    assert_eq!(f.contract.releasable(&big), 150);
    assert_eq!(f.contract.releasable(&small), 75);
    assert_eq!(f.contract.release(&big), 150);
    assert_eq!(f.token.client.balance(&big), 225);
}

#[test]
#[should_panic(expected = "nothing to release")]
fn releasing_twice_without_a_deposit_fails() {
    let env = Env::default();
    let f = setup(&env, &[1, 1], 100);
    let payee = f.payees.get_unchecked(0);

    f.contract.deposit(&f.funder, &100);
    f.contract.release(&payee);
    f.contract.release(&payee);
}

#[test]
#[should_panic(expected = "not a payee")]
fn unknown_address_is_not_a_payee() {
    let env = Env::default();
    let f = setup(&env, &[1, 1], 100);
    let stranger = Address::generate(&env);

    f.contract.releasable(&stranger);
}

#[test]
#[should_panic(expected = "payees and shares must have the same length")]
fn mismatched_payees_and_shares_are_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let token = make_token(&env);
    let contract_id = env.register(PaymentSplitterContract, ());
    let contract = PaymentSplitterContractClient::new(&env, &contract_id);

    let mut payees = Vec::new(&env);
    payees.push_back(Address::generate(&env));
    payees.push_back(Address::generate(&env));
    let mut shares = Vec::new(&env);
    shares.push_back(1_u32);

    contract.initialize(&token.address, &payees, &shares);
}
