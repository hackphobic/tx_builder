use cardano_serialization_lib as csl;

/// A vote on a governance action.
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: VoterSpec,
    pub action: csl::GovernanceActionId,
    pub choice: VoteChoice,
    pub anchor: Option<csl::Anchor>,
}

#[derive(Debug, Clone, Copy)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

impl VoteChoice {
    pub fn to_csl(self) -> csl::VoteKind {
        match self {
            VoteChoice::Yes => csl::VoteKind::Yes,
            VoteChoice::No => csl::VoteKind::No,
            VoteChoice::Abstain => csl::VoteKind::Abstain,
        }
    }
}

/// Who's casting the vote.
#[derive(Debug, Clone)]
pub enum VoterSpec {
    Drep(csl::Credential),
    ConstitutionalCommitteeHot(csl::Credential),
    Spo(csl::Ed25519KeyHash),
}

impl VoterSpec {
    pub fn to_csl(&self) -> csl::Voter {
        match self {
            VoterSpec::Drep(c) => csl::Voter::new_drep_credential(c),
            VoterSpec::ConstitutionalCommitteeHot(c) => {
                csl::Voter::new_constitutional_committee_hot_credential(c)
            }
            VoterSpec::Spo(k) => csl::Voter::new_stake_pool_key_hash(k),
        }
    }
}

impl Vote {
    pub fn to_csl_procedure(&self) -> csl::VotingProcedure {
        match &self.anchor {
            Some(a) => csl::VotingProcedure::new_with_anchor(self.choice.to_csl(), a),
            None => csl::VotingProcedure::new(self.choice.to_csl()),
        }
    }
}

/// A governance proposal submission.
///
/// Wrap whichever variant of CSL's `GovernanceAction` you need; this struct
/// is just sugar for collecting the rest of the bookkeeping.
#[derive(Debug, Clone)]
pub struct ProposalSpec {
    pub deposit: u64,
    pub reward_account: csl::RewardAddress,
    pub anchor: csl::Anchor,
    pub action: csl::GovernanceAction,
}

impl ProposalSpec {
    pub fn to_csl(&self) -> csl::VotingProposal {
        csl::VotingProposal::new(
            &self.action,
            &self.anchor,
            &self.reward_account,
            &csl::BigNum::from(self.deposit),
        )
    }
}
