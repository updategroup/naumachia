use std::error;

use thiserror::Error;

use crate::{
    address::Address,
    output::Output,
    transaction::Transaction,
    Policy,
    error::Error as NauError,
};

pub trait TxORecord<Datum, Redeemer> {
    fn signer(&self) -> &Address;
    fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>>;
    fn balance_at_address(&self, address: &Address, policy: &Policy) -> u64 {
        self.outputs_at_address(address).iter().fold(0, |acc, o| {
            if let Some(val) = o.values().get(policy) {
                acc + val
            } else {
                acc
            }
        })
    }
    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()>; // TODO: Move to other trait
}

#[derive(Debug, Error)]
pub enum TxORecordError {
    #[error("Failed to retrieve outputs at {0:?}: {1:?}.")]
    FailedToRetrieveOutputsAt(Address, Box<dyn error::Error>),
    #[error("Failed to retrieve UTXO with ID {0:?}.")]
    FailedToRetrieveOutputWithId(String),
    #[error("Failed to retrieve redeemer for {0:?}.")]
    FailedToRetrieveRedeemersFor(Address),
    #[error("Failed to retrieve script for {0:?}.")]
    FailedToRetrieveScriptFor(Address),
    #[error("Failed to spend inputs: {0:?}.")]
    FailedToSpendInputs(Box<NauError>),
}

pub type TxORecordResult<T> = Result<T, TxORecordError>;
