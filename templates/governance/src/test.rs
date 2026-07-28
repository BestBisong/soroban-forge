#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, GovernanceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &10, &100);
    (env, admin, client)
}

#[test]
fn create_and_vote_and_execute() {
    let (env, _admin, client) = setup();
    let proposer = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);

    let pid = client.create_proposal(&proposer);
    client.cast_vote(&voter_a, &pid, &true, &7);
    client.cast_vote(&voter_b, &pid, &false, &4);

    // Advance ledger past the voting deadline (100 ledgers).
    env.ledger().with_mut(|li| li.sequence_number += 101);

    client.execute_proposal(&pid);
    let proposal = client.get_proposal(&pid);
    assert!(proposal.executed);
    assert_eq!(proposal.votes_for, 7);
    assert_eq!(proposal.votes_against, 4);
}

#[test]
#[should_panic(expected = "already voted")]
fn double_vote_is_rejected() {
    let (env, _admin, client) = setup();
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let pid = client.create_proposal(&proposer);
    client.cast_vote(&voter, &pid, &true, &5);
    client.cast_vote(&voter, &pid, &true, &5); // should panic
}

#[test]
#[should_panic(expected = "quorum not reached")]
fn execute_fails_below_quorum() {
    let (env, _admin, client) = setup();
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let pid = client.create_proposal(&proposer);
    client.cast_vote(&voter, &pid, &true, &3); // quorum is 10

    env.ledger().with_mut(|li| li.sequence_number += 101);
    client.execute_proposal(&pid);
}
