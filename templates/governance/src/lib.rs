#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Vec};

/// A governance proposal.
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    /// Absolute ledger sequence at which voting closes.
    pub voting_deadline: u32,
    /// Votes cast in favour (weighted by caller's token balance, simplified to
    /// 1 vote per call in this template — adapt to your token contract).
    pub votes_for: i128,
    /// Votes cast against.
    pub votes_against: i128,
    /// Whether the proposal has been executed.
    pub executed: bool,
}

#[contracttype]
pub enum DataKey {
    Proposal(u64),
    /// Set of (proposal_id, voter) pairs that have already voted.
    Voted(u64, Address),
    NextId,
    /// Minimum quorum (total votes required to execute).
    Quorum,
    /// Duration in ledgers for the voting period.
    VotingPeriod,
    Admin,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    /// Initialize the contract.
    ///
    /// * `admin`         — address allowed to change governance parameters
    /// * `quorum`        — minimum total votes (for + against) required to execute
    /// * `voting_period` — voting window in ledgers (~5 s per ledger on mainnet)
    pub fn initialize(env: Env, admin: Address, quorum: i128, voting_period: u32) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Quorum, &quorum);
        env.storage()
            .instance()
            .set(&DataKey::VotingPeriod, &voting_period);
        env.storage().instance().set(&DataKey::NextId, &0u64);
    }

    /// Create a new governance proposal.
    ///
    /// Returns the proposal id.
    pub fn create_proposal(env: Env, proposer: Address) -> u64 {
        proposer.require_auth();

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(0);
        let voting_period: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingPeriod)
            .unwrap_or(1000);
        let deadline = env.ledger().sequence() + voting_period;

        let proposal = Proposal {
            id,
            proposer,
            voting_deadline: deadline,
            votes_for: 0,
            votes_against: 0,
            executed: false,
        };
        env.storage()
            .instance()
            .set(&DataKey::Proposal(id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::NextId, &(id + 1));
        id
    }

    /// Cast a weighted vote on a proposal.
    ///
    /// * `in_favour` — `true` to vote for, `false` to vote against
    /// * `weight`    — voting power (e.g. token balance held by the voter)
    pub fn cast_vote(env: Env, voter: Address, proposal_id: u64, in_favour: bool, weight: i128) {
        voter.require_auth();

        // Prevent double-voting.
        let voted_key = DataKey::Voted(proposal_id, voter.clone());
        if env.storage().instance().has(&voted_key) {
            panic!("already voted on this proposal");
        }

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if env.ledger().sequence() > proposal.voting_deadline {
            panic!("voting period has ended");
        }

        if in_favour {
            proposal.votes_for += weight;
        } else {
            proposal.votes_against += weight;
        }

        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().set(&voted_key, &true);
    }

    /// Execute a proposal after its voting period has ended.
    ///
    /// Succeeds only when:
    /// 1. The voting deadline has passed.
    /// 2. Total votes meet or exceed the configured quorum.
    /// 3. Votes in favour exceed votes against.
    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if env.ledger().sequence() <= proposal.voting_deadline {
            panic!("voting period has not ended yet");
        }
        if proposal.executed {
            panic!("proposal already executed");
        }

        let quorum: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Quorum)
            .unwrap_or(1);
        let total = proposal.votes_for + proposal.votes_against;
        if total < quorum {
            panic!("quorum not reached");
        }
        if proposal.votes_for <= proposal.votes_against {
            panic!("proposal did not pass");
        }

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // TODO: trigger on-chain action here (e.g. call another contract).
    }

    /// Return the current state of a proposal.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found")
    }
}

#[cfg(test)]
mod test;
