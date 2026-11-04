use anyhow::Context;
use rand::Rng;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::run_with_inputs_outputs;
use crate::common::utxo_helper::ConfidentialAmount;
use crate::common::utxo_helper::create_utxos_and_return_conf_assets;
use crate::common::utxo_helper::generate_different_amounts;
use crate::common::utxo_helper::generate_random_confidential;
use crate::common::utxo_helper::random_expected_amount;

use simplicityhl_std::artifacts::tests::utxo::asserts::amount::AmountProgram as TestAmountProgram;
use simplicityhl_std::artifacts::tests::utxo::asserts::amount::derived_amount::{
    AmountArguments as TestAmountArguments, AmountWitness as TestAmountWitness,
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

    /// `index_type`: whether the arm inspects an input.
    fn index_type(mut self, index_type: IndexType) -> Self {
        self.witness.is_input_index = is_input(index_type);
        self
    }

    /// `expected`: the value the arm should produce.
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
        run_with_inputs_outputs(context, program(), self.witness, expect, txids, None)
    }
}

/// A missing input or output also returns an `Expect::PrunedBranch` error,
/// so this approach ensures that the program has the expected input or output.
/// A failed spend is not broadcast, so both cases can use the same UTXOs.
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
) -> anyhow::Result<(Option<ConfidentialAmount>, Vec<Txid>)> {
    let (result, txids) = create_utxos_and_return_conf_assets(
        context,
        index,
        is_explicit,
        is_input,
        expected_amount,
        program(),
    )?;

    let confidential_amount = match is_explicit {
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
fn assert_explicit_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let failing = case(Confidential).index(index).index_type(IndexType::Input);
    let passing = case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_explicit_amount_for_random_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32; // Keep the index small to avoid slowing down tests.
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
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
fn assert_false_explicit_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        real_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_amount_for_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    let failing = case(Confidential)
        .index(index)
        .index_type(IndexType::Output);
    let passing = case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_explicit_amount_for_random_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
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
fn assert_false_explicit_amount_for_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 3;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        real_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_amount_for_confidential_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;
    let conf = conf.unwrap();

    let failing = case(Explicit).index(index).index_type(IndexType::Output);
    // The confidential output value is unknown, so check the input instead.
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(conf.parity_bit)
        .confidential_amount(conf.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_confidential_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 3;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;
    let conf = conf.unwrap();

    let failing = case(Explicit).index(index).index_type(IndexType::Input);
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(conf.parity_bit)
        .confidential_amount(conf.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_amount_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let (false_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(Confidential)
        .index(index)
        .expected_parity(false_bit)
        .confidential_amount(false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_false_confidential_amount_for_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let (false_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(Confidential)
        .index(index)
        .expected_parity(false_bit)
        .confidential_amount(false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let failing = case(ConfidentialInput).index(index);
    let passing = case(ExplicitInput).index(index).expect(expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_explicit_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        real_amount,
    )?;

    case(ExplicitInput)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_confidential_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;
    let conf = conf.unwrap();

    let failing = case(ExplicitInput).index(index);
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(conf.parity_bit)
        .confidential_amount(conf.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_input_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let (false_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(ConfidentialInput)
        .index(index)
        .expected_parity(false_bit)
        .confidential_amount(false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_output_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    let failing = case(ConfidentialOutput).index(index);
    let passing = case(ExplicitOutput).index(index).expect(expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_explicit_output_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        real_amount,
    )?;

    case(ExplicitOutput)
        .index(index)
        .expect(expected_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_output_amount_for_confidential_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Output),
        expected_amount,
    )?;
    let conf = conf.unwrap();

    let failing = case(ExplicitOutput).index(index);
    // The confidential output value is unknown, so check the input instead.
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(conf.parity_bit)
        .confidential_amount(conf.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_output_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let (false_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(ConfidentialOutput)
        .index(index)
        .expected_parity(false_bit)
        .confidential_amount(false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_explicit_current_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    let failing = case(CurrentConfidential).index(index);
    let passing = case(CurrentExplicit).index(index).expect(expected_amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_explicit_current_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let (real_amount, expected_amount): (u64, u64) = generate_different_amounts();
    let is_explicit = true;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        real_amount,
    )?;

    case(CurrentExplicit)
        .index(index)
        .expect(expected_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

#[simplex::test]
fn assert_confidential_current_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let is_explicit = false;

    let (conf, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;
    let conf = conf.unwrap();

    let failing = case(CurrentExplicit).index(index);
    let passing = case(CurrentConfidential)
        .index(index)
        .expected_parity(conf.parity_bit)
        .confidential_amount(conf.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn assert_false_confidential_current_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = random_expected_amount();
    let (false_bit, false_amount): (u8, [u8; 32]) = generate_random_confidential();
    let is_explicit = false;

    let (_, txids) = create_utxos_wrapper(
        &context,
        index,
        is_explicit,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(CurrentConfidential)
        .index(index)
        .expected_parity(false_bit)
        .confidential_amount(false_amount)
        .expecting(&context, Expect::AssertFailed, txids)
}

mod fuzz {
    use super::*;

    use crate::common::utxo_helper::decode_point;

    use crate::common::utxo_helper::fuzz::{
        MAX_UTXO_INDEX, create_confidential_transaction, create_explicit_transaction,
    };

    use simplex::fuzz::proptest::prelude::*;
    use simplex::fuzz::{Expect, Mutations, Scenario};
    use simplex::{FuzzMode, TestContext};

    use simplex::simplicityhl::elements::confidential::Value;
    use simplex::simplicityhl::{Arguments, WitnessValues};

    const SPENDING_INPUT_INDEX: usize = 0;

    #[derive(Clone, Copy)]
    enum ExpectedMutation {
        ExplicitAmount,
        ConfidentialAmount,
    }

    impl ExpectedMutation {
        fn mutate_expected(&self, witness: &mut TestAmountWitness) {
            match self {
                ExpectedMutation::ExplicitAmount => {
                    witness.expected = witness.expected.wrapping_add(1);
                }
                ExpectedMutation::ConfidentialAmount => witness.expected_conf[0] ^= 1,
            }
        }
    }

    // index 0 - explicit input/output

    // Represents tests: [assert_explicit_amount_for_input (successful), assert_explicit_amount_for_random_input (successful)]
    // Fuzzes matching explicit amounts at input indices.
    #[simplex::fuzz]
    fn assert_explicit_amount_for_input_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Explicit)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.input_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [assert_false_explicit_amount_for_input]
    // Fuzzes rejection of incorrect explicit amount expectations at input indices.
    #[simplex::fuzz]
    fn assert_false_explicit_amount_for_input(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Explicit)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.input_amount(index);
                ExpectedMutation::ExplicitAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_explicit_amount_for_output (successful), assert_explicit_amount_for_random_output (successful)]
    // Fuzzes matching explicit amounts at output indices.
    #[simplex::fuzz]
    fn assert_explicit_amount_for_output_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Explicit)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.output_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // assert_false_explicit_amount_for_output
    // Fuzzes rejection of incorrect explicit amount expectations at output indices.
    #[simplex::fuzz]
    fn assert_false_explicit_amount_for_output(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Explicit)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.output_amount(index);
                ExpectedMutation::ExplicitAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_confidential_amount_for_input (failing)]
    // Fuzzes rejection of confidential amounts by the explicit assertion at input indices.
    #[simplex::fuzz]
    fn assert_confidential_amount_for_input_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Explicit)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_explicit_amount_for_confidential_output_fail]
    // Fuzzes rejection of confidential amounts by the explicit assertion at output indices.
    #[simplex::fuzz]
    fn assert_explicit_amount_for_confidential_output_fail(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Explicit)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 1 - confidential input/output

    // Represents tests: [assert_explicit_amount_for_input (failing)]
    // Fuzzes rejection of explicit amounts by the confidential assertion at input indices.
    #[simplex::fuzz]
    fn assert_explicit_amount_for_input_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Confidential)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_explicit_amount_for_output (failing)]
    // Fuzzes rejection of explicit amounts by the confidential assertion at output indices.
    #[simplex::fuzz]
    fn assert_explicit_amount_for_output_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Confidential)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_confidential_amount_for_input (successful)]
    // Fuzzes matching confidential amounts at input indices.
    #[simplex::fuzz]
    fn assert_confidential_amount_for_input_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Confidential)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;
                let Value::Confidential(commitment) = output.value else {
                    panic!("expected a confidential input amount");
                };
                let (amount_parity, amount_data) = decode_point(commitment.serialize());
                witness.expected_parity = amount_parity;
                witness.expected_conf = amount_data;
            });

        context.engine(scenario).spec(|_| Expect::Accept).run();
        Ok(())
    }

    // Represents tests: [assert_false_confidential_amount_for_input]
    // Fuzzes rejection of incorrect confidential amount expectations at output indices.
    #[simplex::fuzz]
    fn assert_false_confidential_amount_for_input(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Confidential)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                ExpectedMutation::ConfidentialAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_false_confidential_amount_for_output]
    // Fuzzes rejection of incorrect confidential amount expectations at output indices.
    #[simplex::fuzz]
    fn assert_false_confidential_amount_for_output(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(Confidential)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                ExpectedMutation::ConfidentialAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 2 - explicit input

    // Represents tests: [assert_explicit_input_amount (successful)]
    // Fuzzes matching explicit amounts at input indices.
    #[simplex::fuzz]
    fn assert_explicit_input_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ExplicitInput)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.input_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [assert_false_explicit_input_amount]
    // Fuzzes rejection of incorrect explicit amount expectations at input indices.
    #[simplex::fuzz]
    fn assert_false_explicit_input_amount(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ExplicitInput)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.input_amount(index);
                ExpectedMutation::ExplicitAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_confidential_input_amount (failing)]
    // Fuzzes rejection of confidential amounts by the explicit assertion at input indices.
    #[simplex::fuzz]
    fn assert_confidential_input_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ExplicitInput)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 3 - confidential input

    // Represents tests: [assert_explicit_input_amount (failing)]
    // Fuzzes rejection of explicit amounts by the confidential assertion at input indices.
    #[simplex::fuzz]
    fn assert_explicit_input_amount_failing(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ConfidentialInput)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_confidential_input_amount (successful)]
    // Fuzzes matching confidential amounts at input indices.
    #[simplex::fuzz]
    fn assert_confidential_input_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ConfidentialInput)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;

                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;

                let Value::Confidential(commitment) = output.value else {
                    panic!("expected a confidential input amount");
                };

                let (amount_parity, amount_data) = decode_point(commitment.serialize());
                witness.expected_parity = amount_parity;
                witness.expected_conf = amount_data;
            });

        context.engine(scenario).spec(|_| Expect::Accept).run();
        Ok(())
    }

    // Represents tests: [assert_false_confidential_input_amount]
    // Fuzzes rejection of incorrect confidential amount expectations at input indices.
    #[simplex::fuzz]
    fn assert_false_confidential_input_amount(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ConfidentialInput)
                .index(index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;

                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;

                let Value::Confidential(commitment) = output.value else {
                    panic!("expected a confidential input amount");
                };

                let (amount_parity, amount_data) = decode_point(commitment.serialize());
                witness.expected_parity = amount_parity;
                witness.expected_conf = amount_data;
                ExpectedMutation::ConfidentialAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 4 - explicit output

    // Represents tests: [assert_explicit_output_amount (successful)]
    // Fuzzes matching explicit amounts at output indices.
    #[simplex::fuzz]
    fn assert_explicit_output_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ExplicitOutput)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.output_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [assert_false_explicit_output_amount]
    // Fuzzes rejection of incorrect explicit amount expectations at output indices.
    #[simplex::fuzz]
    fn assert_false_explicit_output_amount(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ExplicitOutput)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected = case.output_amount(index);
                ExpectedMutation::ExplicitAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // assert_explicit_output_amount_for_confidential_fail
    // Fuzzes rejection of confidential amounts by the explicit assertion at output indices.
    #[simplex::fuzz]
    fn assert_explicit_output_amount_for_confidential_fail(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ExplicitOutput)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 5 - confidential output

    // Represents tests: [assert_explicit_output_amount (failing)]
    // Fuzzes rejection of explicit amounts by the confidential assertion at output indices.
    #[simplex::fuzz]
    fn assert_explicit_output_amount_failing(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ConfidentialOutput)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_false_confidential_output_amount]
    // Fuzzes rejection of incorrect confidential amount expectations at output indices.
    #[simplex::fuzz]
    fn assert_false_confidential_output_amount(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(ConfidentialOutput)
                .index(index)
                .index_type(IndexType::Output)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;

                ExpectedMutation::ConfidentialAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 6 - explicit current input

    // Represents tests: [assert_explicit_current_amount (successful)]
    // Fuzzes matching explicit amounts at the spending input.
    #[simplex::fuzz]
    fn assert_explicit_current_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(CurrentExplicit)
                .index(unused_index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
                witness.expected = case.input_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [assert_false_explicit_current_amount]
    // Fuzzes rejection of incorrect explicit amount expectations at the spending input.
    #[simplex::fuzz]
    fn assert_false_explicit_current_amount(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(CurrentExplicit)
                .index(unused_index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
                witness.expected = case.input_amount(index);
                ExpectedMutation::ExplicitAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_confidential_current_amount (failing)]
    // Fuzzes rejection of confidential amounts by the explicit assertion at the spending input.
    #[simplex::fuzz]
    fn assert_confidential_current_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(CurrentExplicit)
                .index(unused_index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 7 - confidential current input

    // Represents tests: [assert_explicit_current_amount (failing)]
    // Fuzzes rejection of explicit amounts by the confidential assertion at the spending input.
    #[simplex::fuzz]
    fn assert_explicit_current_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(CurrentConfidential)
                .index(unused_index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_explicit_transaction();

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().amounts().layout().everywhere())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [assert_confidential_current_amount (successful)]
    // Fuzzes matching confidential amounts at the spending input.
    #[simplex::fuzz]
    fn assert_confidential_current_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(CurrentConfidential)
                .index(unused_index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;
                let Value::Confidential(commitment) = output.value else {
                    panic!("expected a confidential input amount");
                };
                let (amount_parity, amount_data) = decode_point(commitment.serialize());
                witness.expected_parity = amount_parity;
                witness.expected_conf = amount_data;
            });

        context.engine(scenario).spec(|_| Expect::Accept).run();
        Ok(())
    }

    // Represents tests: [assert_false_confidential_current_amount]
    // Fuzzes rejection of incorrect confidential amount expectations at the spending input.
    #[simplex::fuzz]
    fn assert_false_confidential_current_amount(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAmountArguments {}.into();
            let witness: WitnessValues = case(CurrentConfidential)
                .index(unused_index)
                .index_type(IndexType::Input)
                .witness
                .into();
            (arguments, witness).into()
        });

        let transaction = create_confidential_transaction(&context);

        let mut scenario = Scenario::new(transaction);

        scenario
            .add::<TestAmountProgram>("utxo assertion")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;
                let Value::Confidential(commitment) = output.value else {
                    panic!("expected a confidential input amount");
                };
                let (amount_parity, amount_data) = decode_point(commitment.serialize());
                witness.expected_parity = amount_parity;
                witness.expected_conf = amount_data;
                ExpectedMutation::ConfidentialAmount.mutate_expected(witness);
            });

        context
            .engine(scenario)
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }
}
