use rand::Rng;

use crate::common::core::Expect;
use crate::common::core::run_w_inputs_outputs;
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
    // Currently, the getters for the confidential amount and asset_id of an output index cannot be tested
    #[allow(dead_code)]
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
    index: u32,
    explicit: bool,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestAmountWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected: 0,
            second_expected: [0; 32],
        },
        index: 0,
        explicit: true,
    }
}

impl Case {
    /// `index`: the input/output the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.index = index;
        self.index = index;
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

    /// `second_expected`: the arm's second result.
    fn confidential_amount(mut self, second_expected: [u8; 32]) -> Self {
        self.witness.second_expected = second_expected;
        self
    }

    fn explicit(mut self, explicit: bool) -> Self {
        self.explicit = explicit;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        run_w_inputs_outputs(
            context,
            program(),
            self.witness,
            expect,
            self.index,
            self.explicit,
        )
    }
}

fn create_utxo_for_inputs_wrapper(
    context: &simplex::TestContext,
    index: u32,
    is_explicit_input: bool,
    expected_amount: u64,
) -> anyhow::Result<Option<(u64, [u8; 32])>> {
    let result = create_utxo_for_inputs(
        context,
        index,
        is_explicit_input,
        expected_amount,
        program(),
    )?;

    if is_explicit_input {
        Ok(None)
    } else {
        let (_, conf_amount) = result.unwrap();

        Ok(Some((conf_amount.parity_bit, conf_amount.amount)))
    }
}

#[simplex::test]
fn get_explicit_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_explicit_amount_for_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_confidential_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (expected_parity_bit, expected_conf_amount) =
        create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?
            .unwrap();

    case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_parity_bit)
        .confidential_amount(expected_conf_amount)
        .run(&context)
}

#[simplex::test]
fn get_explicit_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?;

    case(ExplicitInput)
        .index(index)
        .expect(expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_confidential_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (expected_parity_bit, expected_conf_amount) =
        create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?
            .unwrap();

    case(ConfidentialInput)
        .index(index)
        .expect(expected_parity_bit)
        .confidential_amount(expected_conf_amount)
        .run(&context)
}

#[simplex::test]
fn get_explicit_output_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?;

    case(ExplicitOutput)
        .index(index)
        .expect(expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_current_explicit_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?;

    case(CurrentExplicit)
        .index(index)
        .expect(expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_current_confidential_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (expected_parity_bit, expected_conf_amount) =
        create_utxo_for_inputs_wrapper(&context, index, is_explicit_input, expected_amount)?
            .unwrap();

    case(CurrentConfidential)
        .index(index)
        .expect(expected_parity_bit)
        .confidential_amount(expected_conf_amount)
        .explicit(is_explicit_input)
        .run(&context)
}
