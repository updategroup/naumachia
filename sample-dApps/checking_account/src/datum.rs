use naumachia::scripts::context::PubKeyHash;
use naumachia::scripts::raw_validator_script::plutus_data::{Constr, PlutusData};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CheckingAccountDatums {
    CheckingAccount(CheckingAccount),
    AllowedPuller(AllowedPuller),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CheckingAccount {
    pub owner: PubKeyHash,
    pub spend_token_policy: Vec<u8>,
}

impl From<CheckingAccount> for CheckingAccountDatums {
    fn from(value: CheckingAccount) -> Self {
        CheckingAccountDatums::CheckingAccount(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AllowedPuller {
    pub owner: PubKeyHash,
    pub puller: PubKeyHash,
    pub amount_lovelace: u64,
    pub next_pull: i64,
    pub period: i64,
    pub spending_token: Vec<u8>,
    pub checking_account_nft: Vec<u8>,
}

impl From<AllowedPuller> for CheckingAccountDatums {
    fn from(value: AllowedPuller) -> Self {
        CheckingAccountDatums::AllowedPuller(value)
    }
}

impl From<CheckingAccountDatums> for PlutusData {
    fn from(value: CheckingAccountDatums) -> Self {
        match value {
            CheckingAccountDatums::CheckingAccount(CheckingAccount {
                owner,
                spend_token_policy,
            }) => {
                let owner_data = owner.into();
                let policy_data = PlutusData::BoundedBytes(spend_token_policy);
                PlutusData::Constr(Constr {
                    constr: 0,
                    fields: vec![owner_data, policy_data],
                })
            }
            CheckingAccountDatums::AllowedPuller(AllowedPuller {
                owner,
                puller,
                amount_lovelace,
                next_pull,
                period,
                spending_token,
                checking_account_nft,
            }) => {
                let owner = owner.into();
                let puller = puller.into();
                let amount_lovelace = PlutusData::BigInt((amount_lovelace as i64).into()); // TODO
                let next_pull = PlutusData::BigInt(next_pull.into());
                let period = PlutusData::BigInt(period.into());
                let spending_token = PlutusData::BoundedBytes(spending_token);
                let checking_account_nft = PlutusData::BoundedBytes(checking_account_nft);
                PlutusData::Constr(Constr {
                    constr: 0,
                    fields: vec![
                        owner,
                        puller,
                        amount_lovelace,
                        next_pull,
                        period,
                        spending_token,
                        checking_account_nft,
                    ],
                })
            }
        }
    }
}

impl TryFrom<PlutusData> for CheckingAccountDatums {
    type Error = ();

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        let PlutusData::Constr(constr) = value else {
            return Err(());
        };

        let datum = match constr.fields.len() {
            2 => checking_account_datum(&constr.fields)?,
            7 => allowed_puller(&constr.fields)?,
            _ => return Err(()),
        };

        Ok(datum)
    }
}

fn checking_account_datum(fields: &[PlutusData]) -> Result<CheckingAccountDatums, ()> {
    let PlutusData::BoundedBytes(owner_bytes) = fields.get(0).ok_or(())? else {
        return Err(());
    };
    let owner = PubKeyHash::new(&owner_bytes);
    let PlutusData::BoundedBytes(policy_bytes) = fields.get(1).ok_or(())? else {
        return Err(());
    };
    let spend_token_policy = policy_bytes.to_vec();
    Ok(CheckingAccount {
        owner,
        spend_token_policy,
    }
    .into())
}

fn allowed_puller(fields: &[PlutusData]) -> Result<CheckingAccountDatums, ()> {
    let PlutusData::BoundedBytes(owner_bytes) = fields.get(0).ok_or(())? else {
        return Err(());
    };
    let owner = PubKeyHash::new(&owner_bytes);
    let PlutusData::BoundedBytes(puller_bytes) = fields.get(1).ok_or(())? else {
        return Err(());
    };
    let puller = PubKeyHash::new(&puller_bytes);
    let PlutusData::BigInt(amount_lovelace) = fields.get(2).ok_or(())? else {
        return Err(());
    };
    let amount_lovelace: i64 = amount_lovelace.into();
    let PlutusData::BigInt(next_pull) = fields.get(3).ok_or(())? else {
        return Err(());
    };
    let next_pull: i64 = next_pull.into();
    let PlutusData::BigInt(period) = fields.get(4).ok_or(())? else {
        return Err(());
    };
    let period: i64 = period.into();
    let PlutusData::BoundedBytes(spending_token) = fields.get(5).ok_or(())? else {
        return Err(());
    };
    let spending_token = spending_token.to_vec();
    let PlutusData::BoundedBytes(checking_account_nft) = fields.get(6).ok_or(())? else {
        return Err(());
    };
    let checking_account_nft = checking_account_nft.to_vec();
    let datum = CheckingAccountDatums::AllowedPuller(AllowedPuller {
        owner,
        puller,
        amount_lovelace: amount_lovelace as u64, //TODO
        next_pull,
        period,
        spending_token,
        checking_account_nft,
    });
    Ok(datum)
}
