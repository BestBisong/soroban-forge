extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{testutils::Ledger, token::StellarAssetClient, token::TokenClient, Address, Env};

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

#[test]
fn test_english_auction_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EnglishAuctionContract, ());
    let client = EnglishAuctionContractClient::new(&env, &contract_id);

    let seller = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    let asset = make_token(&env);
    let payment = make_token(&env);

    asset.admin.mint(&seller, &100);
    payment.admin.mint(&bidder1, &1000);
    payment.admin.mint(&bidder2, &1000);

    let end_time = 1_000_100_u64;
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    client.initialize(
        &seller,
        &asset.address,
        &payment.address,
        &100,
        &50,
        &10,
        &end_time,
    );

    assert_eq!(client.get_state(), Some(AuctionState::Open));
    assert_eq!(asset.client.balance(&contract_id), 100);

    // Bidder 1 bids 50
    client.bid(&bidder1, &50);
    assert_eq!(client.get_highest_bid(), 50);
    assert_eq!(client.get_highest_bidder(), Some(bidder1.clone()));
    assert_eq!(payment.client.balance(&contract_id), 50);

    // Bidder 2 outbids with 70
    client.bid(&bidder2, &70);
    assert_eq!(client.get_highest_bid(), 70);
    assert_eq!(client.get_highest_bidder(), Some(bidder2.clone()));
    // Bidder 1 was refunded
    assert_eq!(payment.client.balance(&bidder1), 1000);
    assert_eq!(payment.client.balance(&contract_id), 70);

    // Fast-forward past end time
    env.ledger().with_mut(|l| l.timestamp = end_time + 1);

    client.settle();
    assert_eq!(client.get_state(), Some(AuctionState::Settled));
    assert_eq!(payment.client.balance(&seller), 70);
    assert_eq!(asset.client.balance(&bidder2), 100);
}
