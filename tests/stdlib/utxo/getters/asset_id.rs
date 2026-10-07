use anyhow::Context;
use rand::Rng;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::run_with_inputs_outputs;
use crate::common::utxo_helper::ConfidentialAssetId;
use crate::common::utxo_helper::DEFAULT_SEND_AMOUNT;
use crate::common::utxo_helper::{create_utxos, create_utxos_with_new_asset};

use simplicityhl_std::artifacts::tests::utxo::getters::asset_id::AssetIdProgram as TestAssetIdProgram;
use simplicityhl_std::artifacts::tests::utxo::getters::asset_id::derived_asset_id::{
    AssetIdArguments as TestAssetIdArguments, AssetIdWitness as TestAssetIdWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Explicit,
    Confidential,
    ExplicitInput,
    ConfidentialInput,
    ExplicitOutput,
    // Currently, the getters happy path for the confidential asset_id of an output index cannot be tested
    ConfidentialOutput,
    CurrentExplicit,
    CurrentConfidential,
}

#[derive(PartialEq, Eq)]
enum IndexType {
    Input,
    Output,
}

fn is_input(o: IndexType) -> bool {
    o == IndexType::Input
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> TestAssetIdProgram {
    TestAssetIdProgram::new(&TestAssetIdArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestAssetIdWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestAssetIdWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected: DEFAULT_EXPECTED,
            expected_parity: 0,
            expected_conf: DEFAULT_EXPECTED,
        },
    }
}

impl Case {
    /// `index`: the input/output the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.index = index;
        self
    }

    /// `is_input`: whether the arm inspects an input.
    fn index_type(mut self, index_type: IndexType) -> Self {
        self.witness.is_input_index = is_input(index_type);
        self
    }

    /// `expected`: value the arm should produce.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = expected;
        self
    }

    /// `expected_parity`: value the arm should produce.
    fn expected_parity(mut self, expected_parity: u8) -> Self {
        self.witness.expected_parity = expected_parity;
        self
    }

    /// `expected_confidential`: the arm's confidential result.
    fn confidential_asset_id(mut self, expected_confidential: [u8; 32]) -> Self {
        self.witness.expected_conf = expected_confidential;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext, txids: Vec<Txid>) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok, txids)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(
        self,
        context: &simplex::TestContext,
        expect: Expect,
        txids: Vec<Txid>,
    ) -> anyhow::Result<()> {
        run_with_inputs_outputs(context, program(), self.witness, expect, txids)
    }
}

/// Missing input or output also returns `Expect::PrunedBranch` error,
/// so this approach ensures that the program has the expected input or output.
/// A failed spend is not broadcasted, so both cases can use the same UTXOs.
fn fail_and_pass_cases(
    context: &simplex::TestContext,
    failing: Case,
    passing: Case,
    txids: Vec<Txid>,
) -> anyhow::Result<()> {
    failing.expecting(context, Expect::PrunedBranch, txids.clone())?;
    passing.run(context, txids)
}

fn create_utxos_wrapper(
    context: &simplex::TestContext,
    index: u32,
    is_explicit: bool,
) -> anyhow::Result<(Option<ConfidentialAssetId>, Vec<Txid>)> {
    let (result, txids) = create_utxos(
        context,
        index,
        is_explicit,
        true, // is_input is always true, because this parameter affects only tested amount
        DEFAULT_SEND_AMOUNT,
        program(),
    )?;

    let confidential_asset = match is_explicit {
        true => None,
        false => {
            let (conf_asset, _) =
                result.context("confidential UTXO creation returned no commitment")?;

            Some(conf_asset)
        }
    };

    Ok((confidential_asset, txids))
}

#[simplex::test]
fn get_explicit_asset_id_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    let failing = case(Confidential).index(index).index_type(IndexType::Input);
    let passing = case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_asset_id_for_random_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32; // not a big value to not to slow down tests
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_asset_id)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_asset_id_for_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    let failing = case(Confidential).index(index);
    let passing = case(Explicit).index(index).expect(expected_asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_asset_id_for_random_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32;
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_asset_id)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_asset_id_for_confidential_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(&context, index, is_explicit)?;
    let conf = conf.unwrap();

    let failing = case(Explicit).index(index).index_type(IndexType::Output);
    // Output `index` is blinded because input `index` is confidential, and its commitment
    // is only made when the spend is blinded, so check the input instead.
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(conf.parity_bit)
        .confidential_asset_id(conf.asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_confidential_asset_id_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let is_explicit = false;

    let (conf_asset, txids) = create_utxos_wrapper(&context, index, is_explicit)?;
    let expected = conf_asset.unwrap();

    let failing = case(Explicit).index(index).index_type(IndexType::Input);
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(expected.parity_bit)
        .confidential_asset_id(expected.asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_input_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    let failing = case(ConfidentialInput).index(index);
    let passing = case(ExplicitInput).index(index).expect(expected_asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_confidential_input_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let is_explicit = false;

    let (conf_asset, txids) = create_utxos_wrapper(&context, index, is_explicit)?;
    let expected = conf_asset.unwrap();

    let failing = case(ExplicitInput).index(index);
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(expected.parity_bit)
        .confidential_asset_id(expected.asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_output_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    let failing = case(ConfidentialOutput).index(index);
    let passing = case(ExplicitOutput).index(index).expect(expected_asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_output_asset_id_for_confidential_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(&context, index, is_explicit)?;
    let conf = conf.unwrap();

    let failing = case(ExplicitOutput).index(index);
    // Output `index` is blinded because input `index` is confidential, and its commitment
    // is only made when the spend is blinded, so check the input instead.
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(conf.parity_bit)
        .confidential_asset_id(conf.asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_current_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let (expected_asset_id, txids) =
        create_utxos_with_new_asset(&context, index, true, DEFAULT_SEND_AMOUNT, program())?;

    let failing = case(CurrentConfidential).index(index);
    let passing = case(CurrentExplicit).index(index).expect(expected_asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_confidential_current_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let is_explicit = false;

    let (conf_asset, txids) = create_utxos_wrapper(&context, index, is_explicit)?;
    let expected = conf_asset.unwrap();

    let failing = case(CurrentExplicit).index(index);
    let passing = case(CurrentConfidential)
        .index(index)
        .expected_parity(expected.parity_bit)
        .confidential_asset_id(expected.asset_id);

    fail_and_pass_cases(&context, failing, passing, txids)
}

// The signer adds its own inputs and outputs for the fee and change, so `u32::MAX` is the index
// that is certain not to exist.
#[simplex::test]
fn get_explicit_input_asset_id_for_missing_input_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(&context, 0, true)?;

    case(ExplicitInput)
        .index(u32::MAX)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_explicit_output_asset_id_for_missing_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(&context, 0, true)?;

    case(ExplicitOutput)
        .index(u32::MAX)
        .expecting(&context, Expect::PrunedBranch, txids)
}
