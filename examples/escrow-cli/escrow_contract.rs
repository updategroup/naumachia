use async_trait::async_trait;
use naumachia::address::PolicyId;
use naumachia::output::OutputId;
use naumachia::{
    address::Address,
    ledger_client::LedgerClient,
    logic::SCLogic,
    logic::{SCLogicError, SCLogicResult},
    output::Output,
    scripts::ScriptError,
    scripts::ScriptResult,
    scripts::{TxContext, ValidatorCode},
    transaction::TxActions,
    values::Values,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const SOME_NETWORK: u8 = 99;

pub struct EscrowValidatorScript;

impl ValidatorCode<EscrowDatum, ()> for EscrowValidatorScript {
    fn execute(&self, datum: EscrowDatum, _redeemer: (), ctx: TxContext) -> ScriptResult<()> {
        signer_is_recipient(&datum, &ctx)?;
        Ok(())
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        Ok(Address::new("escrow validator"))
    }

    fn script_hex(&self) -> ScriptResult<&str> {
        todo!()
    }
}

fn signer_is_recipient(datum: &EscrowDatum, ctx: &TxContext) -> ScriptResult<()> {
    if datum.receiver != ctx.signer {
        Err(ScriptError::FailedToExecute(format!(
            "Signer: {:?} doesn't match receiver: {:?}",
            ctx.signer, datum.receiver
        )))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EscrowContract;

#[allow(dead_code)]
#[derive(Clone)]
pub enum EscrowEndpoint {
    Escrow { amount: u64, receiver: Address },
    Claim { output_id: OutputId },
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EscrowDatum {
    receiver: Address,
}

impl EscrowDatum {
    pub fn receiver(&self) -> &Address {
        &self.receiver
    }
}

#[derive(Debug, Error)]
enum EscrowContractError {
    #[error("Output with ID {0:?} not found.")]
    OutputNotFound(OutputId),
}

#[async_trait]
impl SCLogic for EscrowContract {
    type Endpoint = EscrowEndpoint;
    type Lookup = ();
    type LookupResponse = Vec<Output<Self::Datum>>;
    type Datum = EscrowDatum;
    type Redeemer = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> SCLogicResult<TxActions<EscrowDatum, ()>> {
        match endpoint {
            EscrowEndpoint::Escrow { amount, receiver } => escrow(amount, receiver),
            EscrowEndpoint::Claim { output_id } => claim(&output_id, txo_record).await,
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        let address = EscrowValidatorScript
            .address(SOME_NETWORK)
            .map_err(SCLogicError::ValidatorScript)?;
        let outputs = txo_record
            .outputs_at_address(&address)
            .await
            .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
        Ok(outputs)
    }
}

fn escrow(amount: u64, receiver: Address) -> SCLogicResult<TxActions<EscrowDatum, ()>> {
    let script = EscrowValidatorScript;
    let address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script, SOME_NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let datum = EscrowDatum { receiver };
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let u_tx = TxActions::default().with_script_init(datum, values, address);
    Ok(u_tx)
}

async fn claim<Record: LedgerClient<EscrowDatum, ()>>(
    output_id: &OutputId,
    txo_record: &Record,
) -> SCLogicResult<TxActions<EscrowDatum, ()>> {
    let script = Box::new(EscrowValidatorScript);
    let output = lookup_output(output_id, txo_record).await?;
    let u_tx = TxActions::default().with_script_redeem(output, (), script);
    Ok(u_tx)
}

async fn lookup_output<Record: LedgerClient<EscrowDatum, ()>>(
    id: &OutputId,
    txo_record: &Record,
) -> SCLogicResult<Output<EscrowDatum>> {
    let script_address = EscrowValidatorScript
        .address(SOME_NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let outputs = txo_record
        .outputs_at_address(&script_address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
    outputs
        .iter()
        .find(|o| o.id() == id)
        .cloned()
        .ok_or_else(|| {
            SCLogicError::Lookup(Box::new(EscrowContractError::OutputNotFound(id.clone())))
        })
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use naumachia::{
        ledger_client::{in_memory_ledger::TestBackendsBuilder, LedgerClient},
        smart_contract::{SmartContract, SmartContractTrait},
    };

    #[tokio::test]
    async fn escrow__can_create_instance() {
        let me = Address::new("me");
        let alice = Address::new("alice");
        let start_amount = 100;
        let mut backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(PolicyId::ADA, start_amount)
            .finish_output()
            .build();

        let escrow_amount = 25;
        let call = EscrowEndpoint::Escrow {
            amount: escrow_amount,
            receiver: alice.clone(),
        };
        let script = EscrowValidatorScript;
        let contract = SmartContract::new(&EscrowContract, &backend);
        contract.hit_endpoint(call).await.unwrap();

        let escrow_address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script, 0).unwrap();
        let expected = escrow_amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual);

        let expected = start_amount - escrow_amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual);

        let instance = backend
            .ledger_client
            .outputs_at_address(&script.address(0).unwrap())
            .await
            .unwrap()
            .pop()
            .unwrap();
        // The creator tries to spend escrow but fails because not recipient
        let call = EscrowEndpoint::Claim {
            output_id: instance.id().clone(),
        };

        let contract = SmartContract::new(&EscrowContract, &backend);
        let attempt = contract.hit_endpoint(call.clone()).await;
        assert!(attempt.is_err());

        // The recipient tries to spend and succeeds
        backend.ledger_client.signer = alice.clone();
        let contract = SmartContract::new(&EscrowContract, &backend);
        contract.hit_endpoint(call).await.unwrap();

        let alice_balance = backend
            .ledger_client
            .balance_at_address(&alice, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(alice_balance, escrow_amount);

        let script_balance = backend
            .ledger_client
            .balance_at_address(&escrow_address, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(script_balance, 0);
    }
}
