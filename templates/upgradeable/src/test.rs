use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env};

fn setup(env: &Env) -> (UpgradeableContractClient<'_>, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register(UpgradeableContract, (admin.clone(),));
    (UpgradeableContractClient::new(env, &contract_id), admin)
}

/// Upload empty WASM to get a valid hash for testing.
/// In test mode the host accepts empty WASM blobs (see `soroban-env-host`'s
/// `upload_contract_wasm` — the `testutils` feature allows zero-byte payloads).
fn test_wasm_hash(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(Bytes::new(env))
}

#[test]
fn constructor_sets_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    assert_eq!(client.admin(), admin);
}

#[test]
fn admin_can_upgrade() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let wasm_hash = test_wasm_hash(&env);
    client.upgrade(&wasm_hash);
}

#[test]
#[should_panic(expected = "HostError")]
fn non_admin_cannot_upgrade() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(UpgradeableContract, (admin.clone(),));
    let client = UpgradeableContractClient::new(&env, &contract_id);
    let wasm_hash = BytesN::from_array(&env, &[0; 32]);
    client.upgrade(&wasm_hash);
}
