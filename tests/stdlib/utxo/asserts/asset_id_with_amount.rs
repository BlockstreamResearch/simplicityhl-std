use rand::Rng;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::run_with_inputs_outputs;
use crate::common::utxo_helper::ConfidentialAsset;
use crate::common::utxo_helper::create_utxos;
use crate::common::utxo_helper::create_utxos_with_new_asset;
use crate::common::utxo_helper::generate_different_amounts;
use crate::common::utxo_helper::generate_random_confidential;
use crate::common::utxo_helper::random_expected_amount;

use simplicityhl_std::artifacts::tests::utxo::asserts::asset_id_with_amount::AssetIdWithAmountProgram as TestAssetIdWithAmountProgram;
use simplicityhl_std::artifacts::tests::utxo::asserts::asset_id_with_amount::derived_asset_id_with_amount::{
    AssetIdWithAmountArguments as TestAssetIdWithAmountArguments, AssetIdWithAmountWitness as TestAssetIdWithAmountWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Explicit,
    Confidential,
    ExplicitInput,
    ConfidentialInput,
    ExplicitOutput,
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

fn program() -> TestAssetIdWithAmountProgram {
    TestAssetIdWithAmountProgram::new(&TestAssetIdWithAmountArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestAssetIdWithAmountWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestAssetIdWithAmountWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected_asset: [0; 32],
            expected_asset_parity: 0,
            expected_conf_asset: [0; 32],
            expected_amount: 0,
            expected_amount_parity: 0,
            expected_conf_amount: [0; 32],
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
    fn expect(mut self, expected_asset: [u8; 32], expected_amount: u64) -> Self {
        self.witness.expected_asset = expected_asset;
        self.witness.expected_amount = expected_amount;
        self
    }

    /// `expected_parity`: value the arm should produce.
    fn expected_parity(mut self, expected_asset_parity: u8, expected_amount_parity: u8) -> Self {
        self.witness.expected_asset_parity = expected_asset_parity;
        self.witness.expected_amount_parity = expected_amount_parity;
        self
    }

    /// `expected_conf_asset` and `expected_conf_amount`.
    fn confidentials(
        mut self,
        expected_conf_asset: [u8; 32],
        expected_conf_amount: [u8; 32],
    ) -> Self {
        self.witness.expected_conf_asset = expected_conf_asset;
        self.witness.expected_conf_amount = expected_conf_amount;
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
    is_input: bool,
    expected_amount: u64,
) -> anyhow::Result<(Option<ConfidentialAsset>, Vec<Txid>)> {
    create_utxos(
        context,
        index,
        is_explicit,
        is_input,
        expected_amount,
        program(),
    )
}

#[simplex::test]
fn assert_explicit_asset_id_with_amount_for_input(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    let failing = case(Confidential).index(index).index_type(IndexType::Input);
    let passing = case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_asset_id, expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_explicit_asset_id_with_amount_for_random_input(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32;
    let expected_amount = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_asset_id, expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn assert_false_explicit_asset_id_with_amount_for_input(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();

    let (_, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(
            context.get_network().policy_asset().into_inner().0,
            real_amount,
        )
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_asset_id_with_amount_for_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Output),
        expected_amount,
        program(),
    )?;

    let failing = case(Confidential)
        .index(index)
        .index_type(IndexType::Output);
    let passing = case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_asset_id, expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_explicit_asset_id_with_amount_for_random_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32;
    let expected_amount = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Output),
        expected_amount,
        program(),
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_asset_id, expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn assert_false_explicit_asset_id_with_amount_for_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();

    let (_, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Output),
        expected_amount,
        program(),
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(
            context.get_network().policy_asset().into_inner().0,
            real_amount,
        )
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_asset_id_with_amount_for_confidential_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;
    let (asset, amount) = conf.unwrap();

    let failing = case(Explicit).index(index).index_type(IndexType::Output);
    // Cannot know confidential value in the output, so check the input instead
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(asset.parity_bit, amount.parity_bit)
        .confidentials(asset.asset_id, amount.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_confidential_asset_id_with_amount_for_input(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (expected, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;
    let (asset, amount) = expected.unwrap();

    let failing = case(Explicit).index(index).index_type(IndexType::Input);
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(asset.parity_bit, amount.parity_bit)
        .confidentials(asset.asset_id, amount.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_asset_id_with_amount_for_input(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        random_expected_amount(),
    )?;

    let (false_asset_bit, false_asset): (u8, [u8; 32]) = generate_random_confidential();
    let (false_amount_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();

    case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(false_asset_bit, false_amount_bit)
        .confidentials(false_asset, false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_false_confidential_asset_id_with_amount_for_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        random_expected_amount(),
    )?;

    let (false_asset_bit, false_asset): (u8, [u8; 32]) = generate_random_confidential();
    let (false_amount_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();

    case(Confidential)
        .index(index)
        .index_type(IndexType::Output)
        .expected_parity(false_asset_bit, false_amount_bit)
        .confidentials(false_asset, false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_input_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    let failing = case(ConfidentialInput).index(index);
    let passing = case(ExplicitInput)
        .index(index)
        .expect(expected_asset_id, expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_explicit_input_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();

    let (_, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    case(ExplicitInput)
        .index(index)
        .expect(
            context.get_network().policy_asset().into_inner().0,
            real_amount,
        )
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_confidential_input_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (expected, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let (conf_asset, conf_amount) = expected.unwrap();

    let (asset_parity_bit, asset_conf_amount) = (conf_asset.parity_bit, conf_asset.asset_id);
    let (amount_parity_bit, amount_conf_amount) = (conf_amount.parity_bit, conf_amount.amount);

    let failing = case(ExplicitInput).index(index);
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(asset_parity_bit, amount_parity_bit)
        .confidentials(asset_conf_amount, amount_conf_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_input_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        random_expected_amount(),
    )?;

    let (false_asset_bit, false_asset): (u8, [u8; 32]) = generate_random_confidential();
    let (false_amount_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();

    case(ConfidentialInput)
        .index(index)
        .expected_parity(false_asset_bit, false_amount_bit)
        .confidentials(false_asset, false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_output_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Output),
        expected_amount,
        program(),
    )?;

    let failing = case(ConfidentialOutput).index(index);
    let passing = case(ExplicitOutput)
        .index(index)
        .expect(expected_asset_id, expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_explicit_output_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();

    let (_, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Output),
        expected_amount,
        program(),
    )?;

    case(ExplicitOutput)
        .index(index)
        .expect(
            context.get_network().policy_asset().into_inner().0,
            real_amount,
        )
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_false_confidential_output_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        random_expected_amount(),
    )?;

    let (false_asset_bit, false_asset): (u8, [u8; 32]) = generate_random_confidential();
    let (false_amount_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();

    case(ConfidentialOutput)
        .index(index)
        .expected_parity(false_asset_bit, false_amount_bit)
        .confidentials(false_asset, false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_output_asset_id_with_amount_for_confidential_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;
    let (asset, amount) = conf.unwrap();

    let failing = case(ExplicitOutput).index(index);
    // Cannot know confidential value in the output, so check the input instead
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(asset.parity_bit, amount.parity_bit)
        .confidentials(asset.asset_id, amount.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_explicit_current_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount = random_expected_amount();
    let (expected_asset_id, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    let failing = case(CurrentConfidential).index(index);
    let passing = case(CurrentExplicit)
        .index(index)
        .expect(expected_asset_id, expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_explicit_current_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();

    let (_, txids) = create_utxos_with_new_asset(
        &context,
        index,
        is_input(IndexType::Input),
        expected_amount,
        program(),
    )?;

    case(CurrentExplicit)
        .index(index)
        .expect(
            context.get_network().policy_asset().into_inner().0,
            real_amount,
        )
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_confidential_current_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (expected, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let (conf_asset, conf_amount) = expected.unwrap();

    let (asset_parity_bit, asset_conf_amount) = (conf_asset.parity_bit, conf_asset.asset_id);
    let (amount_parity_bit, amount_conf_amount) = (conf_amount.parity_bit, conf_amount.amount);

    let failing = case(CurrentExplicit).index(index);
    let passing = case(CurrentConfidential)
        .index(index)
        .expected_parity(asset_parity_bit, amount_parity_bit)
        .confidentials(asset_conf_amount, amount_conf_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_current_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        random_expected_amount(),
    )?;

    let (false_asset_bit, false_asset): (u8, [u8; 32]) = generate_random_confidential();
    let (false_amount_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();

    case(CurrentConfidential)
        .index(index)
        .expected_parity(false_asset_bit, false_amount_bit)
        .confidentials(false_asset, false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}
