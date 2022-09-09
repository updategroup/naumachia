use super::*;
use crate::keys::{my_base_addr, MAINNET, TESTNET};
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    address::{EnterpriseAddress, RewardAddress, StakeCredential},
    builders::input_builder::{InputBuilderResult, SingleInputBuilder},
    builders::output_builder::SingleOutputBuilderResult,
    builders::tx_builder::{
        ChangeSelectionAlgo, TransactionBuilder, TransactionBuilderConfigBuilder,
    },
    builders::witness_builder::PartialPlutusWitness,
    crypto::TransactionHash,
    ledger::alonzo::fees::LinearFee,
    ledger::common::value::{BigInt, BigNum, Int, Value as CMLValue},
    plutus::{
        CostModel, Costmdls, ExUnitPrices, Language, PlutusData, PlutusScript, PlutusV1Script,
    },
    AssetName, Assets, MultiAsset, PolicyID, RequiredSigners, TransactionInput, TransactionOutput,
    UnitInterval,
};
use std::fs::File;
use std::io::Read;

use futures::future::join_all;

#[ignore]
#[tokio::test]
async fn genesis() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let res = bf.genesis().await.unwrap();
    println!("{:?}", res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn protocol_params() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let epoch = 227;
    let res = bf.protocol_params(epoch).await.unwrap();
    dbg!("{:?}", res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn utxos() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    // TODO: Find a good stable address to use
    // let address = "addr_test1wrtlw9csk7vc9peauh9nzpg45zemvj3w9m532e93nwer24gjwycdl";
    // let address = "addr_test1wrsexavz37208qda7mwwu4k7hcpg26cz0ce86f5e9kul3hqzlh22t";
    let address = "addr_test1wp9m8xkpt2tmy7madqldspgzgug8f2p3pwhz589cq75685slenwf4";
    let res = bf.utxos(address).await.unwrap();
    dbg!(&res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn datum() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    // TODO: Find a good stable address to use
    // let datum_hash = "d1cede40100329bfd7edbb1245a4d24de23924f00341886dc5f5bf6d06c65629";
    let datum_hash = "a9fbe52ace8f89e0ae64d88f879e159b97d51f27d8f932c9aa165e5ce5f0f28e";
    let res = bf.datum(datum_hash).await.unwrap();
    println!("{}", serde_json::to_string_pretty(&res).unwrap());
    Ok(())
}

#[ignore]
#[tokio::test]
async fn address_info() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    // let address = "addr1q97dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fsmqn6vh";
    // let address = "addr1qp7dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fs2k6a72";
    let address = "addr_test1wrtlw9csk7vc9peauh9nzpg45zemvj3w9m532e93nwer24gjwycdl";

    let res = bf.address_info(address).await.unwrap();
    dbg!(&res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn account_associated_addresses() {
    let bf = get_test_bf_http_client().unwrap();
    let base_addr = my_base_addr();
    let staking_cred = base_addr.stake_cred();

    let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
        .to_address()
        .to_bech32(None)
        .unwrap();
    let res = bf.assoc_addresses(&reward_addr).await.unwrap();
    dbg!(&res);
}

#[ignore]
#[tokio::test]
async fn account_associated_addresses_total() {
    let bf = get_test_bf_http_client().unwrap();
    let base_addr = my_base_addr();
    let staking_cred = base_addr.stake_cred();

    let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
        .to_address()
        .to_bech32(None)
        .unwrap();
    let res = bf
        .account_associated_addresses_total(&reward_addr)
        .await
        .unwrap();
    dbg!(&res);
}

// Most of these values are made up
fn test_tx_builder() -> TransactionBuilder {
    let coefficient = BigNum::from_str("44").unwrap();
    let constant = BigNum::from_str("155381").unwrap();
    let linear_fee = LinearFee::new(&coefficient, &constant);

    let pool_deposit = BigNum::from_str("500000000").unwrap();
    let key_deposit = BigNum::from_str("2000000").unwrap();

    let coins_per_utxo_byte = BigNum::from_str("4310").unwrap();
    let mem_num = BigNum::from_str("577").unwrap();
    let mem_den = BigNum::from_str("10000").unwrap();
    let mem_price = UnitInterval::new(&mem_num, &mem_den);
    let step_num = BigNum::from_str("721").unwrap();
    let step_den = BigNum::from_str("10000000").unwrap();
    let step_price = UnitInterval::new(&step_num, &step_den);
    let ex_unit_prices = ExUnitPrices::new(&mem_price, &step_price);
    // let mut cost_models = Costmdls();
    // let language = Language::new_plutus_v1();
    // let op_costs = Vec::new();
    // let v1_model = CostModel::new(&language, &op_costs);
    // cost_models.insert(&v1_model);
    let arr = vec![
        197209, 0, 1, 1, 396231, 621, 0, 1, 150000, 1000, 0, 1, 150000, 32, 2477736, 29175, 4,
        29773, 100, 29773, 100, 29773, 100, 29773, 100, 29773, 100, 29773, 100, 100, 100, 29773,
        100, 150000, 32, 150000, 32, 150000, 32, 150000, 1000, 0, 1, 150000, 32, 150000, 1000, 0,
        8, 148000, 425507, 118, 0, 1, 1, 150000, 1000, 0, 8, 150000, 112536, 247, 1, 150000, 10000,
        1, 136542, 1326, 1, 1000, 150000, 1000, 1, 150000, 32, 150000, 32, 150000, 32, 1, 1,
        150000, 1, 150000, 4, 103599, 248, 1, 103599, 248, 1, 145276, 1366, 1, 179690, 497, 1,
        150000, 32, 150000, 32, 150000, 32, 150000, 32, 150000, 32, 150000, 32, 148000, 425507,
        118, 0, 1, 1, 61516, 11218, 0, 1, 150000, 32, 148000, 425507, 118, 0, 1, 1, 148000, 425507,
        118, 0, 1, 1, 2477736, 29175, 4, 0, 82363, 4, 150000, 5000, 0, 1, 150000, 32, 197209, 0, 1,
        1, 150000, 32, 150000, 32, 150000, 32, 150000, 32, 150000, 32, 150000, 32, 150000, 32,
        3345831, 1, 1,
    ];
    let cm = CostModel::new(
        &Language::new_plutus_v1(),
        &arr.iter().map(|&i| Int::from(i)).collect(),
    );
    let mut cost_models = Costmdls::new();
    cost_models.insert(&cm);

    let tx_builder_cfg = TransactionBuilderConfigBuilder::new()
        .fee_algo(&linear_fee)
        .pool_deposit(&pool_deposit)
        .key_deposit(&key_deposit)
        .max_value_size(5000)
        .max_tx_size(16384)
        .coins_per_utxo_byte(&coins_per_utxo_byte)
        .ex_unit_prices(&ex_unit_prices)
        .collateral_percentage(150)
        .max_collateral_inputs(3)
        .costmdls(&cost_models)
        .build()
        .unwrap();
    TransactionBuilder::new(&tx_builder_cfg)
}

fn payment_input(amt: u64, owner_addr: &CMLAddress) -> InputBuilderResult {
    let index = BigNum::from_str("0").unwrap();
    let hash_raw = "8561258e210352fba2ac0488afed67b3427a27ccf1d41ec030c98a8199bc22ec";
    let tx_hash = TransactionHash::from_hex(hash_raw).unwrap();
    let payment_input = TransactionInput::new(
        &tx_hash, // tx hash
        &index,   // index
    );
    let coin = amt.into();
    let value = CMLValue::new(&coin);
    let utxo_info = TransactionOutput::new(&owner_addr, &value);
    let input_builder = SingleInputBuilder::new(&payment_input, &utxo_info);

    input_builder.payment_key().unwrap()
}

fn read_script_from_file(file_path: &str) -> PlutusScriptFile {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn always_succeeds_script() -> PlutusScript {
    let script_file = read_script_from_file("./always-succeeds-spending.plutus");
    let script_hex = script_file.cborHex;
    let script_bytes = hex::decode(&script_hex).unwrap();
    let v1 = PlutusV1Script::from_bytes(script_bytes).unwrap();
    PlutusScript::from_v1(&v1)
}

fn always_succeeds_script_address() -> CMLAddress {
    let script = always_succeeds_script();
    let script_hash = script.hash();
    let stake_cred = StakeCredential::from_scripthash(&script_hash);
    let enterprise_addr = EnterpriseAddress::new(TESTNET, &stake_cred);
    enterprise_addr.to_address()
}

fn script_input(amt: u64) -> InputBuilderResult {
    let hash_raw = "8561258e210352fba2ac0488afed67b3427a27ccf1d41ec030c98a8199bc22ec";
    let index = BigNum::from_str("1").unwrap();
    let tx_hash = TransactionHash::from_hex(hash_raw).unwrap();
    let script_input = TransactionInput::new(
        &tx_hash, // tx hash
        &index,   // index
    );

    let script = always_succeeds_script();
    let script_addr = always_succeeds_script_address();

    let coin = amt.into();
    let value = CMLValue::new(&coin);
    let utxo_info = TransactionOutput::new(&script_addr, &value);
    let input_builder = SingleInputBuilder::new(&script_input, &utxo_info);

    let redeemer_integer = BigInt::from_str("456").unwrap();
    let data = PlutusData::new_integer(&redeemer_integer);
    let partial_witness = PartialPlutusWitness::new(&script, &data);

    let required_signers = RequiredSigners::new();
    let datum_integer = BigInt::from_str("123").unwrap();
    let datum = PlutusData::new_integer(&datum_integer);
    input_builder
        .plutus_script(&partial_witness, &required_signers, &datum)
        .unwrap()
}

fn new_output(amt: u64) -> SingleOutputBuilderResult {
    let output_address = CMLAddress::from_bech32("addr_test1qpu5vlrf4xkxv2qpwngf6cjhtw542ayty80v8dyr49rf5ewvxwdrt70qlcpeeagscasafhffqsxy36t90ldv06wqrk2qum8x5w").unwrap();
    let coin = amt.into();
    let value = CMLValue::new(&coin);
    let output = TransactionOutput::new(&output_address, &value);
    SingleOutputBuilderResult::new(&output)
}

#[ignore]
#[tokio::test]
async fn execution_units() {
    let bf = get_test_bf_http_client().unwrap();
    let base_addr = my_base_addr();

    let mut tx_builder = test_tx_builder();

    let payment_input = payment_input(3000000, &base_addr.to_address());
    tx_builder.add_input(&payment_input);

    let script_input = script_input(2000001);
    tx_builder.add_input(&script_input);

    // Add output
    let single_output = new_output(1000002);
    tx_builder.add_output(&single_output).unwrap();

    // Add change address
    let change_address = CMLAddress::from_bech32(
        "addr_test1gz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerspqgpsqe70et",
    )
    .unwrap();
    // tx_builder.add_change_if_needed(&change_address).unwrap();

    let algo = ChangeSelectionAlgo::Default;
    // let signed_tx_builder = tx_builder.build(algo, &change_address).unwrap();
    let tx_redeemer_builder = tx_builder
        .build_for_evaluation(algo, &change_address)
        .unwrap();
    let transaction = tx_redeemer_builder.draft_tx();
    // let transaction = signed_tx_builder.build_unchecked();

    println!("{}", &transaction.to_json().unwrap());
    let bytes = transaction.to_bytes();

    let res = bf.execution_units(&bytes).await.unwrap();

    dbg!(res);
}

use crate::schemas::Value;
use serde::Deserialize;

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Deserialize, Debug)]
struct PlutusScriptFile {
    r#type: String,
    description: String,
    cborHex: String,
}

impl PlutusScriptFile {
    pub fn cbor_hex(&self) -> &str {
        &self.cborHex
    }
}

#[ignore]
#[tokio::test]
async fn init_always_succeeds_script() {
    let my_base_addr = my_base_addr();
    let my_address = my_base_addr.to_address();
    let script_addr = always_succeeds_script_address().to_bech32(None).unwrap();
    dbg!(&script_addr);

    let bf = get_test_bf_http_client().unwrap();

    let my_utxos = bf
        .utxos(&my_address.to_bech32(None).unwrap())
        .await
        .unwrap();

    // dbg!(&my_utxos);d
    let mut tx_builder = test_tx_builder();

    for utxo in my_utxos.iter() {
        let input = input_from_utxo(&my_address, utxo);
        tx_builder.add_input(&input);
    }

    let algo = ChangeSelectionAlgo::Default;
    let redeemer_builder = tx_builder.build_for_evaluation(algo, &my_address).unwrap();
    let tx = redeemer_builder.draft_tx();
    println!("{}", tx.to_json().unwrap());
}

fn input_from_utxo(my_address: &CMLAddress, utxo: &UTxO) -> InputBuilderResult {
    let index = utxo.output_index().into();
    let hash_raw = utxo.tx_hash();
    let tx_hash = TransactionHash::from_hex(hash_raw).unwrap();
    let payment_input = TransactionInput::new(
        &tx_hash, // tx hash
        &index,   // index
    );
    let value = cmlvalue_from_values(&utxo.amount());
    let utxo_info = TransactionOutput::new(&my_address, &value);
    let input_builder = SingleInputBuilder::new(&payment_input, &utxo_info);

    input_builder.payment_key().unwrap()
}

fn cmlvalue_from_values(values: &[Value]) -> CMLValue {
    let mut cml_value = CMLValue::zero();
    for value in values.iter() {
        if let Value { unit, quantity } = value {
            let add_value = match unit.as_str() {
                "lovelace" => CMLValue::new(&BigNum::from_str(quantity).unwrap()),
                _ => {
                    let policy_id_hex = &unit[..56];
                    let policy_id = PolicyID::from_hex(policy_id_hex).unwrap();
                    let asset_name_hex = &unit[56..];
                    let asset_name_bytes = hex::decode(asset_name_hex).unwrap();
                    let asset_name = AssetName::new(asset_name_bytes.into()).unwrap();
                    let mut assets = Assets::new();
                    assets.insert(&asset_name, &BigNum::from_str(quantity).unwrap());
                    let mut multi_assets = MultiAsset::new();
                    multi_assets.insert(&policy_id, &assets);
                    CMLValue::new_from_assets(&multi_assets)
                }
            };
            cml_value = cml_value.checked_add(&add_value).unwrap();
        }
    }
    dbg!(cml_value)
}
