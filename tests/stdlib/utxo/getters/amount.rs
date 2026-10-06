use anyhow::Context;
use rand::Rng;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::run_with_inputs_outputs;
use crate::common::utxo_helper::ConfidentialAmount;
use crate::common::utxo_helper::create_utxo_for_inputs;

use simplicityhl_std::artifacts::tests::utxo::getters::amount::AmountProgram as TestAmountProgram;
use simplicityhl_std::artifacts::tests::utxo::getters::amount::derived_amount::{
    AmountArguments as TestAmountArguments, AmountWitness as TestAmountWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Explicit,
    Confidential,
    ExplicitInput,
    ConfidentialInput,
    ExplicitOutput,
    // Currently, the getters happy path for the confidential amount of an output index cannot be tested
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

fn program() -> TestAmountProgram {
    TestAmountProgram::new(&TestAmountArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestAmountWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestAmountWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected: 0,
            expected_parity: 0,
            expected_conf: [0; 32],
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
    fn expect(mut self, expected: u64) -> Self {
        self.witness.expected = expected;
        self
    }

    /// `expected_parity`: value the arm should produce.
    fn expected_parity(mut self, expected_parity: u8) -> Self {
        self.witness.expected_parity = expected_parity;
        self
    }

    /// `expected_confidential`: the arm's confidential result.
    fn confidential_amount(mut self, expected_confidential: [u8; 32]) -> Self {
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

fn create_utxo_for_inputs_wrapper(
    context: &simplex::TestContext,
    index: u32,
    is_explicit_input: bool,
    is_input: bool,
    expected_amount: u64,
) -> anyhow::Result<(Option<ConfidentialAmount>, Vec<Txid>)> {
    let (result, txids) = create_utxo_for_inputs(
        context,
        index,
        is_explicit_input,
        is_input,
        expected_amount,
        program(),
    )?;

    let confidential_amount = match is_explicit_input {
        true => None,
        false => {
            let (_, conf_amount) =
                result.context("confidential UTXO creation returned no commitment")?;

            Some(conf_amount)
        }
    };

    Ok((confidential_amount, txids))
}

#[simplex::test]
fn get_explicit_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_amount_for_random_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32; // not a big value to not to slow down tests
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_amount_for_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_amount_for_random_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_amount_for_confidential_input_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_explicit_amount_for_confidential_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_confidential_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 3;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (conf_amount, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let expected = conf_amount.unwrap();

    case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(expected.parity_bit)
        .confidential_amount(expected.amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_confidential_amount_for_explicit_input_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 4;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_confidential_amount_for_explicit_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(Confidential)
        .index(index)
        .index_type(IndexType::Output)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_explicit_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(ExplicitInput)
        .index(index)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_input_amount_for_confidential_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(ExplicitInput)
        .index(index)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_confidential_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (conf_amount, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let expected = conf_amount.unwrap();

    case(ConfidentialInput)
        .index(index)
        .expected_parity(expected.parity_bit)
        .confidential_amount(expected.amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_confidential_input_amount_for_explicit_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(ConfidentialInput)
        .index(index)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_explicit_output_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(ExplicitOutput)
        .index(index)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_output_amount_for_confidential_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(ExplicitOutput)
        .index(index)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_confidential_output_amount_for_explicit_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(ConfidentialOutput)
        .index(index)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_explicit_current_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(CurrentExplicit)
        .index(index)
        .expect(expected_amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_explicit_current_amount_for_confidentional_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(CurrentExplicit)
        .index(index)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_confidential_current_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (conf_amount, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let expected = conf_amount.unwrap();

    case(CurrentConfidential)
        .index(index)
        .expected_parity(expected.parity_bit)
        .confidential_amount(expected.amount)
        .run(&context, txids)
}

#[simplex::test]
fn get_confidential_current_amount_for_explicit_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    let (_, txids) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(CurrentConfidential)
        .index(index)
        .expecting(&context, Expect::PrunedBranch, txids)
}
