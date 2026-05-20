use cardano_serialization_lib as csl;

/// A simplified certificate enum that covers the common dApp / wallet use
/// cases. Wrap CSL's [`csl::Certificate`] directly for anything exotic via
/// [`Cert::Raw`].
#[derive(Debug, Clone)]
pub enum Cert {
    // ---- Pre-Conway staking ---------------------------------------------
    StakeRegistration {
        stake_cred: csl::Credential,
        /// Pass `None` for the legacy (pre-Conway) form. From Conway on,
        /// the on-chain encoding requires the key-deposit amount; CSL
        /// expects it explicitly.
        deposit: Option<u64>,
    },
    StakeDeregistration {
        stake_cred: csl::Credential,
        deposit_refund: Option<u64>,
    },
    StakeDelegation {
        stake_cred: csl::Credential,
        pool: csl::Ed25519KeyHash,
    },

    // ---- Conway: vote delegation ----------------------------------------
    VoteDelegation {
        stake_cred: csl::Credential,
        drep: csl::DRep,
    },
    StakeAndVoteDelegation {
        stake_cred: csl::Credential,
        pool: csl::Ed25519KeyHash,
        drep: csl::DRep,
    },
    StakeRegistrationAndDelegation {
        stake_cred: csl::Credential,
        pool: csl::Ed25519KeyHash,
        deposit: u64,
    },
    VoteRegistrationAndDelegation {
        stake_cred: csl::Credential,
        drep: csl::DRep,
        deposit: u64,
    },
    StakeVoteRegistrationAndDelegation {
        stake_cred: csl::Credential,
        pool: csl::Ed25519KeyHash,
        drep: csl::DRep,
        deposit: u64,
    },

    // ---- Conway: DRep ---------------------------------------------------
    DRepRegistration {
        drep_cred: csl::Credential,
        deposit: u64,
        anchor: Option<csl::Anchor>,
    },
    DRepDeregistration {
        drep_cred: csl::Credential,
        deposit_refund: u64,
    },
    DRepUpdate {
        drep_cred: csl::Credential,
        anchor: Option<csl::Anchor>,
    },

    // ---- Conway: constitutional committee -------------------------------
    CommitteeHotAuth {
        cold_cred: csl::Credential,
        hot_cred: csl::Credential,
    },
    CommitteeColdResign {
        cold_cred: csl::Credential,
        anchor: Option<csl::Anchor>,
    },

    /// Escape hatch — pass a fully constructed CSL `Certificate`.
    Raw(csl::Certificate),
}

impl Cert {
    /// Convert to CSL's `Certificate`.
    pub fn to_csl(&self) -> csl::Certificate {
        use csl::*;
        match self {
            Cert::StakeRegistration { stake_cred, deposit } => match deposit {
                Some(d) => Certificate::new_stake_registration(&StakeRegistration::new_with_explicit_deposit(
                    stake_cred,
                    &BigNum::from(*d),
                )),
                None => Certificate::new_stake_registration(&StakeRegistration::new(stake_cred)),
            },
            Cert::StakeDeregistration { stake_cred, deposit_refund } => match deposit_refund {
                Some(d) => Certificate::new_stake_deregistration(&StakeDeregistration::new_with_explicit_refund(
                    stake_cred,
                    &BigNum::from(*d),
                )),
                None => Certificate::new_stake_deregistration(&StakeDeregistration::new(stake_cred)),
            },
            Cert::StakeDelegation { stake_cred, pool } => {
                Certificate::new_stake_delegation(&StakeDelegation::new(stake_cred, pool))
            }
            Cert::VoteDelegation { stake_cred, drep } => {
                Certificate::new_vote_delegation(&VoteDelegation::new(stake_cred, drep))
            }
            Cert::StakeAndVoteDelegation { stake_cred, pool, drep } => {
                Certificate::new_stake_and_vote_delegation(
                    &StakeAndVoteDelegation::new(stake_cred, pool, drep),
                )
            }
            Cert::StakeRegistrationAndDelegation { stake_cred, pool, deposit } => {
                Certificate::new_stake_registration_and_delegation(
                    &StakeRegistrationAndDelegation::new(stake_cred, pool, &BigNum::from(*deposit)),
                )
            }
            Cert::VoteRegistrationAndDelegation { stake_cred, drep, deposit } => {
                Certificate::new_vote_registration_and_delegation(
                    &VoteRegistrationAndDelegation::new(stake_cred, drep, &BigNum::from(*deposit)),
                )
            }
            Cert::StakeVoteRegistrationAndDelegation {
                stake_cred,
                pool,
                drep,
                deposit,
            } => Certificate::new_stake_vote_registration_and_delegation(
                &StakeVoteRegistrationAndDelegation::new(
                    stake_cred,
                    pool,
                    drep,
                    &BigNum::from(*deposit),
                ),
            ),
            Cert::DRepRegistration { drep_cred, deposit, anchor } => {
                let reg = match anchor {
                    Some(a) => DRepRegistration::new_with_anchor(drep_cred, &BigNum::from(*deposit), a),
                    None => DRepRegistration::new(drep_cred, &BigNum::from(*deposit)),
                };
                Certificate::new_drep_registration(&reg)
            }
            Cert::DRepDeregistration { drep_cred, deposit_refund } => {
                Certificate::new_drep_deregistration(&DRepDeregistration::new(
                    drep_cred,
                    &BigNum::from(*deposit_refund),
                ))
            }
            Cert::DRepUpdate { drep_cred, anchor } => {
                let upd = match anchor {
                    Some(a) => DRepUpdate::new_with_anchor(drep_cred, a),
                    None => DRepUpdate::new(drep_cred),
                };
                Certificate::new_drep_update(&upd)
            }
            Cert::CommitteeHotAuth { cold_cred, hot_cred } => {
                Certificate::new_committee_hot_auth(&CommitteeHotAuth::new(cold_cred, hot_cred))
            }
            Cert::CommitteeColdResign { cold_cred, anchor } => {
                let resign = match anchor {
                    Some(a) => CommitteeColdResign::new_with_anchor(cold_cred, a),
                    None => CommitteeColdResign::new(cold_cred),
                };
                Certificate::new_committee_cold_resign(&resign)
            }
            Cert::Raw(c) => c.clone(),
        }
    }
}
