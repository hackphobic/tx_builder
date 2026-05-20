//! The main [`TxBuilder`] that assembles inputs, outputs, mints, certs,
//! governance items, etc. and produces an unsigned [`csl::Transaction`].

use cardano_serialization_lib as csl;

use crate::cert::Cert;
use crate::config::ProtocolParameters;
use crate::error::{Error, Result};
use crate::governance::{ProposalSpec, Vote};
use crate::input::SpendingInput;
use crate::mint::{MintAction, MintWitness};
use crate::output::Output;
use crate::script::{DatumSpec, ScriptSource};
use crate::utxo::Utxo;

/// Fluent transaction builder. See the crate-level docs for the full
/// lifecycle.
pub struct TxBuilder<'a> {
    params: &'a ProtocolParameters,
    change_address: Option<csl::Address>,

    inputs: Vec<SpendingInput>,
    reference_inputs: Vec<Utxo>,

    outputs: Vec<Output>,

    collateral: Vec<Utxo>,
    collateral_return: Option<Output>,
    total_collateral: Option<u64>,

    required_signers: Vec<csl::Ed25519KeyHash>,

    mints: Vec<MintAction>,
    certs: Vec<Cert>,
    withdrawals: Vec<Withdrawal>,
    votes: Vec<Vote>,
    proposals: Vec<ProposalSpec>,

    validity_start: Option<u64>,
    ttl: Option<u64>,

    metadata: Vec<(u64, csl::TransactionMetadatum)>,
    auxiliary_scripts: Vec<csl::NativeScript>,
}

#[derive(Debug, Clone)]
struct Withdrawal {
    address: csl::RewardAddress,
    amount: u64,
    /// `None` ⇒ withdrawn by key, signature required.
    witness: Option<WithdrawalWitness>,
}

#[derive(Debug, Clone)]
enum WithdrawalWitness {
    Native(csl::NativeScript),
    Plutus {
        script: ScriptSource,
        redeemer: csl::PlutusData,
        ex_units: csl::ExUnits,
    },
}

impl<'a> TxBuilder<'a> {
    pub fn new(params: &'a ProtocolParameters) -> Self {
        Self {
            params,
            change_address: None,
            inputs: Vec::new(),
            reference_inputs: Vec::new(),
            outputs: Vec::new(),
            collateral: Vec::new(),
            collateral_return: None,
            total_collateral: None,
            required_signers: Vec::new(),
            mints: Vec::new(),
            certs: Vec::new(),
            withdrawals: Vec::new(),
            votes: Vec::new(),
            proposals: Vec::new(),
            validity_start: None,
            ttl: None,
            metadata: Vec::new(),
            auxiliary_scripts: Vec::new(),
        }
    }

    // ------------------------------------------------------------- setters

    pub fn change_address(&mut self, address: &csl::Address) -> &mut Self {
        self.change_address = Some(address.clone());
        self
    }

    pub fn add_input(&mut self, input: SpendingInput) -> &mut Self {
        self.inputs.push(input);
        self
    }

    /// Shorthand for a pubkey-locked input.
    pub fn add_pubkey_input(&mut self, utxo: &Utxo) -> &mut Self {
        self.add_input(SpendingInput::pubkey(utxo.clone()))
    }

    pub fn add_reference_input(&mut self, utxo: &Utxo) -> &mut Self {
        self.reference_inputs.push(utxo.clone());
        self
    }

    pub fn add_output(&mut self, output: Output) -> &mut Self {
        self.outputs.push(output);
        self
    }

    pub fn add_collateral(&mut self, utxo: &Utxo) -> &mut Self {
        self.collateral.push(utxo.clone());
        self
    }

    pub fn collateral_return(&mut self, output: Output) -> &mut Self {
        self.collateral_return = Some(output);
        self
    }

    pub fn total_collateral(&mut self, amount: u64) -> &mut Self {
        self.total_collateral = Some(amount);
        self
    }

    pub fn required_signer(&mut self, key: csl::Ed25519KeyHash) -> &mut Self {
        self.required_signers.push(key);
        self
    }

    pub fn add_mint(&mut self, mint: MintAction) -> &mut Self {
        self.mints.push(mint);
        self
    }

    pub fn add_cert(&mut self, cert: Cert) -> &mut Self {
        self.certs.push(cert);
        self
    }

    /// Reward-address withdrawal locked by a pubkey credential (vkey
    /// signature required at signing time).
    pub fn add_withdrawal(&mut self, address: &csl::RewardAddress, amount: u64) -> &mut Self {
        self.withdrawals.push(Withdrawal {
            address: address.clone(),
            amount,
            witness: None,
        });
        self
    }

    /// Withdrawal from a script-locked reward address using a Plutus script.
    pub fn add_plutus_withdrawal(
        &mut self,
        address: &csl::RewardAddress,
        amount: u64,
        script: ScriptSource,
        redeemer: csl::PlutusData,
        ex_units: csl::ExUnits,
    ) -> &mut Self {
        self.withdrawals.push(Withdrawal {
            address: address.clone(),
            amount,
            witness: Some(WithdrawalWitness::Plutus {
                script,
                redeemer,
                ex_units,
            }),
        });
        self
    }

    /// Withdrawal from a reward address locked by a native script.
    pub fn add_native_withdrawal(
        &mut self,
        address: &csl::RewardAddress,
        amount: u64,
        script: csl::NativeScript,
    ) -> &mut Self {
        self.withdrawals.push(Withdrawal {
            address: address.clone(),
            amount,
            witness: Some(WithdrawalWitness::Native(script)),
        });
        self
    }

    pub fn add_vote(&mut self, vote: Vote) -> &mut Self {
        self.votes.push(vote);
        self
    }

    pub fn add_proposal(&mut self, proposal: ProposalSpec) -> &mut Self {
        self.proposals.push(proposal);
        self
    }

    pub fn validity_start(&mut self, slot: u64) -> &mut Self {
        self.validity_start = Some(slot);
        self
    }

    pub fn ttl(&mut self, slot: u64) -> &mut Self {
        self.ttl = Some(slot);
        self
    }

    /// Attach a CIP-25-style metadata label.
    pub fn metadata(&mut self, label: u64, datum: csl::TransactionMetadatum) -> &mut Self {
        self.metadata.push((label, datum));
        self
    }

    pub fn auxiliary_script(&mut self, script: csl::NativeScript) -> &mut Self {
        self.auxiliary_scripts.push(script);
        self
    }

    // ----------------------------------------------------- build pipeline

    /// Finalize the transaction. Returns an *unsigned* [`csl::Transaction`]:
    /// the witness set carries scripts / datums / redeemers but no
    /// `Vkeywitness` entries. Sign externally and merge into the witness
    /// set.
    pub fn build(&self) -> Result<csl::Transaction> {
        let change = self
            .change_address
            .as_ref()
            .ok_or(Error::MissingChangeAddress)?;

        let has_plutus = self.has_plutus_witnesses();
        if has_plutus && self.collateral.is_empty() {
            return Err(Error::MissingCollateral);
        }

        let cfg = self.params.to_csl_config()?;
        let mut tb = csl::TransactionBuilder::new(&cfg);

        // 1. Inputs (regular + script).
        let inputs = self.build_inputs()?;
        tb.set_inputs(&inputs);

        // 2. Reference inputs (extras the user added; reference-script
        //    UTxOs are already pulled in by `to_plutus_script_source`
        //    inside `build_inputs` / `build_mint`).
        for u in &self.reference_inputs {
            tb.add_reference_input(&u.input());
        }

        // 3. Outputs.
        for out in &self.outputs {
            tb.add_output(&out.to_csl()?)?;
        }

        // 4. Collateral.
        if !self.collateral.is_empty() {
            let mut coll = csl::TxInputsBuilder::new();
            for c in &self.collateral {
                coll.add_regular_input(&c.address, &c.input(), &c.output()?.amount())?;
            }
            tb.set_collateral(&coll);

            match (&self.collateral_return, self.total_collateral) {
                (Some(ret), Some(total)) => {
                    tb.set_collateral_return(&ret.to_csl()?);
                    tb.set_total_collateral(&csl::BigNum::from(total));
                }
                (Some(ret), None) => {
                    // CSL auto-computes total from collateral inputs - return.
                    tb.set_collateral_return_and_total(&ret.to_csl()?)?;
                }
                (None, Some(total)) => {
                    tb.set_total_collateral(&csl::BigNum::from(total));
                }
                (None, None) => {}
            }
        }

        // 5. Required signers.
        for k in &self.required_signers {
            tb.add_required_signer(k);
        }

        // 6. Mint.
        if !self.mints.is_empty() {
            tb.set_mint_builder(&self.build_mint()?);
        }

        // 7. Certificates.
        if !self.certs.is_empty() {
            let mut cb = csl::CertificatesBuilder::new();
            for c in &self.certs {
                cb.add(&c.to_csl())?;
            }
            tb.set_certs_builder(&cb);
        }

        // 8. Withdrawals.
        if !self.withdrawals.is_empty() {
            tb.set_withdrawals_builder(&self.build_withdrawals()?);
        }

        // 9. Voting + proposals (Conway).
        if !self.votes.is_empty() {
            tb.set_voting_builder(&self.build_voting()?);
        }
        if !self.proposals.is_empty() {
            tb.set_voting_proposal_builder(&self.build_proposals()?);
        }

        // 10. Validity range.
        if let Some(s) = self.validity_start {
            tb.set_validity_start_interval_bignum(&csl::BigNum::from(s));
        }
        if let Some(t) = self.ttl {
            tb.set_ttl_bignum(&csl::BigNum::from(t));
        }

        // 11. Metadata.
        if !self.metadata.is_empty() || !self.auxiliary_scripts.is_empty() {
            let mut aux = csl::AuxiliaryData::new();
            if !self.metadata.is_empty() {
                let mut gm = csl::GeneralTransactionMetadata::new();
                for (label, datum) in &self.metadata {
                    gm.insert(&csl::BigNum::from(*label), datum);
                }
                aux.set_metadata(&gm);
            }
            if !self.auxiliary_scripts.is_empty() {
                let mut ns = csl::NativeScripts::new();
                for s in &self.auxiliary_scripts {
                    ns.add(s);
                }
                aux.set_native_scripts(&ns);
            }
            tb.set_auxiliary_data(&aux);
        }

        // 12. Script data hash. Must include ALL cost-model languages that
        //     actually appear in this tx; passing the full set is safe.
        if has_plutus {
            tb.calc_script_data_hash(&self.params.cost_models)?;
        }

        // 13. Balance to change address.
        tb.add_change_if_needed(change)?;

        // 14. Finalize body + witness set (no vkey witnesses).
        Ok(tb.build_tx()?)
    }

    // -------------------------------------------------------------- helpers

    fn has_plutus_witnesses(&self) -> bool {
        self.inputs.iter().any(|i| i.is_plutus())
            || self
                .mints
                .iter()
                .any(|m| matches!(m.witness, MintWitness::Plutus { .. }))
            || self
                .withdrawals
                .iter()
                .any(|w| matches!(w.witness, Some(WithdrawalWitness::Plutus { .. })))
    }

    fn build_inputs(&self) -> Result<csl::TxInputsBuilder> {
        let mut ib = csl::TxInputsBuilder::new();
        for input in &self.inputs {
            let utxo = input.utxo();
            let value = utxo.output()?.amount();
            match input {
                SpendingInput::Pubkey(_) => {
                    ib.add_regular_input(&utxo.address, &utxo.input(), &value)?;
                }
                SpendingInput::Plutus {
                    script,
                    datum,
                    redeemer,
                    ex_units,
                    ..
                } => {
                    let script_src = script.to_plutus_script_source();
                    let redeemer = csl::Redeemer::new(
                        &csl::RedeemerTag::new_spend(),
                        // The real index is fixed by CSL when the tx is
                        // built — inputs get sorted. 0 is a placeholder.
                        &csl::BigNum::from(0u64),
                        redeemer,
                        ex_units,
                    );
                    let witness = match datum {
                        DatumSpec::Inline => csl::PlutusWitness::new_with_ref_without_datum(
                            &script_src,
                            &redeemer,
                        ),
                        DatumSpec::Witness(d) => csl::PlutusWitness::new_with_ref(
                            &script_src,
                            &csl::DatumSource::new(d),
                            &redeemer,
                        ),
                        DatumSpec::Reference(ref_utxo) => csl::PlutusWitness::new_with_ref(
                            &script_src,
                            &csl::DatumSource::new_ref_input(&ref_utxo.input()),
                            &redeemer,
                        ),
                    };
                    ib.add_plutus_script_input(&witness, &utxo.input(), &value);
                }
                SpendingInput::NativeScript { script, .. } => {
                    let info = csl::NativeScriptSource::new(script);
                    ib.add_native_script_input(&info, &utxo.input(), &value);
                }
            }
        }
        Ok(ib)
    }

    fn build_mint(&self) -> Result<csl::MintBuilder> {
        let mut mb = csl::MintBuilder::new();
        for action in &self.mints {
            for (name, qty) in &action.assets {
                let name_csl = name.to_csl()?;
                let amount = signed_to_csl_int(*qty)?;
                match &action.witness {
                    MintWitness::Native(script) => {
                        let witness = csl::MintWitness::new_native_script(
                            &csl::NativeScriptSource::new(script),
                        );
                        mb.add_asset(&witness, &name_csl, &amount)?;
                    }
                    MintWitness::Plutus {
                        script,
                        redeemer,
                        ex_units,
                    } => {
                        let redeemer = csl::Redeemer::new(
                            &csl::RedeemerTag::new_mint(),
                            &csl::BigNum::from(0u64), // CSL fixes the index
                            redeemer,
                            ex_units,
                        );
                        let witness = csl::MintWitness::new_plutus_script(
                            &script.to_plutus_script_source(),
                            &redeemer,
                        );
                        mb.add_asset(&witness, &name_csl, &amount)?;
                    }
                }
            }
        }
        Ok(mb)
    }

    fn build_withdrawals(&self) -> Result<csl::WithdrawalsBuilder> {
        let mut wb = csl::WithdrawalsBuilder::new();
        for w in &self.withdrawals {
            let amount = csl::BigNum::from(w.amount);
            match &w.witness {
                None => wb.add(&w.address, &amount)?,
                Some(WithdrawalWitness::Native(script)) => {
                    let info = csl::NativeScriptSource::new(script);
                    wb.add_with_native_script(&w.address, &amount, &info)?;
                }
                Some(WithdrawalWitness::Plutus {
                    script,
                    redeemer,
                    ex_units,
                }) => {
                    let r = csl::Redeemer::new(
                        &csl::RedeemerTag::new_reward(),
                        &csl::BigNum::from(0u64),
                        redeemer,
                        ex_units,
                    );
                    let pw = csl::PlutusWitness::new_with_ref_without_datum(
                        &script.to_plutus_script_source(),
                        &r,
                    );
                    wb.add_with_plutus_witness(&w.address, &amount, &pw)?;
                }
            }
        }
        Ok(wb)
    }

    fn build_voting(&self) -> Result<csl::VotingBuilder> {
        let mut vb = csl::VotingBuilder::new();
        for v in &self.votes {
            vb.add(&v.voter.to_csl(), &v.action, &v.to_csl_procedure())?;
        }
        Ok(vb)
    }

    fn build_proposals(&self) -> Result<csl::VotingProposalBuilder> {
        let mut pb = csl::VotingProposalBuilder::new();
        for p in &self.proposals {
            pb.add(&p.to_csl())?;
        }
        Ok(pb)
    }
}

/// CSL `Int` from a signed i128, for use in `MintBuilder::add_asset`.
fn signed_to_csl_int(qty: i128) -> Result<csl::Int> {
    if qty >= 0 {
        Ok(csl::Int::new(&csl::BigNum::from(qty as u64)))
    } else {
        Ok(csl::Int::new_negative(&csl::BigNum::from((-qty) as u64)))
    }
}

// Surface PolicyId from the value module for callers that prefer the
// builder path.
pub use crate::value::{AssetName, Assets, PolicyId};
