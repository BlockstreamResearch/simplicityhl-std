use simplex::simplicityhl::elements::Txid;

use rand::Rng;

use crate::common::core::Expect;
use crate::common::core::run_with_inputs_outputs;
use crate::common::utxo_helper::ConfidentialAsset;
use crate::common::utxo_helper::{
    create_utxos_and_return_conf_assets, create_utxos_with_new_asset, random_expected_amount,
};

use simplicityhl_std::artifacts::tests::utxo::getters::asset_id_with_amount::AssetIdWithAmountProgram as TestAssetIdWithAmountProgram;
use simplicityhl_std::artifacts::tests::utxo::getters::asset_id_with_amount::derived_asset_id_with_amount::{
    AssetIdWithAmountArguments as TestAssetIdWithAmountArguments, AssetIdWithAmountWitness as TestAssetIdWithAmountWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Explicit,
    Confidential,
    ExplicitInput,
    ConfidentialInput,
    ExplicitOutput,
    // Currently, the getters happy path for the confidential amount and asset_id of an output index cannot be tested
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
        run_with_inputs_outputs(context, program(), self.witness, expect, txids, None)
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
    create_utxos_and_return_conf_assets(
        context,
        index,
        is_explicit,
        is_input,
        expected_amount,
        program(),
    )
}

#[simplex::test]
fn get_explicit_asset_id_with_amount_for_input(
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
fn get_explicit_asset_id_with_amount_for_random_input(
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
fn get_explicit_asset_id_with_amount_for_output(
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
fn get_explicit_asset_id_with_amount_for_random_output(
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
fn get_explicit_asset_id_with_amount_for_confidential_output_fail(
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
    // Output `index` is blinded because input `index` is confidential, and its commitment
    // is only made when the spend is blinded, so check the input instead.
    let passing = case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expected_parity(asset.parity_bit, amount.parity_bit)
        .confidentials(asset.asset_id, amount.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_confidential_asset_id_with_amount_for_input(
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
fn get_explicit_input_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
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
fn get_confidential_input_asset_id_with_amount(
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
fn get_explicit_output_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
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
fn get_explicit_output_asset_id_with_amount_for_confidential_fail(
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
    // Output `index` is blinded because input `index` is confidential, and its commitment
    // is only made when the spend is blinded, so check the input instead.
    let passing = case(ConfidentialInput)
        .index(index)
        .expected_parity(asset.parity_bit, amount.parity_bit)
        .confidentials(asset.asset_id, amount.amount);

    fail_and_pass_cases(&context, failing, passing, txids)
}

#[simplex::test]
fn get_explicit_current_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
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
fn get_confidential_current_asset_id_with_amount(
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
fn get_explicit_input_asset_id_with_amount_for_missing_input_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(
        &context,
        0,
        true,
        is_input(IndexType::Input),
        random_expected_amount(),
    )?;

    case(ExplicitInput)
        .index(u32::MAX)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_explicit_output_asset_id_with_amount_for_missing_output_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(
        &context,
        0,
        true,
        is_input(IndexType::Input),
        random_expected_amount(),
    )?;

    case(ExplicitOutput)
        .index(u32::MAX)
        .expecting(&context, Expect::PrunedBranch, txids)
}

mod fuzz {
    use simplex::simplicityhl::elements::confidential::{Asset, Value};
    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::proptest::prelude::*;
    use simplex::fuzz::{Expect, Mutations, Scenario};
    use simplex::{FuzzMode, TestContext};

    use crate::common::utxo_helper::decode_point;
    use crate::common::utxo_helper::fuzz::{
        MAX_UTXO_INDEX, create_confidential_transaction, create_explicit_transaction,
    };

    use super::*;

    const SPENDING_INPUT_INDEX: usize = 0;

    // index 0 - explicit input/output

    // Represents tests: [get_explicit_asset_id_with_amount_for_input (successful), get_explicit_asset_id_with_amount_for_random_input (successful)]
    // Fuzzes matching explicit assets and amounts at input indices.
    #[simplex::fuzz]
    fn get_explicit_asset_id_with_amount_for_input_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected_asset = case.input_asset(index);
                witness.expected_amount = case.input_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [get_explicit_asset_id_with_amount_for_output (successful), get_explicit_asset_id_with_amount_for_random_output (successful)]
    // Fuzzes matching explicit assets and amounts at output indices.
    #[simplex::fuzz]
    fn get_explicit_asset_id_with_amount_for_output_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected_asset = case.output_asset(index);
                witness.expected_amount = case.output_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [get_explicit_asset_id_with_amount_for_confidential_output_fail]
    // Fuzzes rejection of confidential assets and amounts by the explicit getter at output indices.
    #[simplex::fuzz]
    fn get_explicit_asset_id_with_amount_for_confidential_output_fail(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
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

    // Represents tests: [get_confidential_asset_id_with_amount_for_input (failing)]
    // Fuzzes rejection of confidential assets and amounts by the explicit getter at input indices.
    #[simplex::fuzz]
    fn get_confidential_asset_id_with_amount_for_input_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
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

    // index 1 - confidential input/output

    // Represents tests: [get_explicit_asset_id_with_amount_for_input (failing)]
    // Fuzzes rejection of explicit assets and amounts by the confidential getter at input indices.
    #[simplex::fuzz]
    fn get_explicit_asset_id_with_amount_for_input_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [get_explicit_asset_id_with_amount_for_output (failing)]
    // Fuzzes rejection of explicit assets and amounts by the confidential getter at output indices.
    #[simplex::fuzz]
    fn get_explicit_asset_id_with_amount_for_output_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [get_confidential_asset_id_with_amount_for_input (successful)]
    // Fuzzes matching confidential assets and amounts at input indices.
    #[simplex::fuzz]
    fn get_confidential_asset_id_with_amount_for_input_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;

                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;

                let Asset::Confidential(asset) = output.asset else {
                    panic!("expected a confidential input asset");
                };
                let Value::Confidential(amount) = output.value else {
                    panic!("expected a confidential input amount");
                };

                let (asset_parity, asset_data) = decode_point(asset.serialize());
                let (amount_parity, amount_data) = decode_point(amount.serialize());
                witness.expected_asset_parity = asset_parity;
                witness.expected_conf_asset = asset_data;
                witness.expected_amount_parity = amount_parity;
                witness.expected_conf_amount = amount_data;
            });

        context.engine(scenario).spec(|_| Expect::Accept).run();
        Ok(())
    }

    // index 2 - explicit input

    // Represents tests: [get_explicit_input_asset_id_with_amount (successful)]
    // Fuzzes matching explicit assets and amounts at input indices.
    #[simplex::fuzz]
    fn get_explicit_input_asset_id_with_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected_asset = case.input_asset(index);
                witness.expected_amount = case.input_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [get_confidential_input_asset_id_with_amount (failing)]
    // Fuzzes rejection of confidential assets and amounts by the explicit getter at input indices.
    #[simplex::fuzz]
    fn get_confidential_input_asset_id_with_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
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

    // Represents tests: [get_explicit_input_asset_id_with_amount_for_missing_input_fail]
    // Fuzzes transaction layouts and checks rejection of a missing input index.
    #[simplex::fuzz]
    fn get_explicit_input_asset_id_with_amount_for_missing_input_fail(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const MISSING_INDEX: u32 = u32::MAX;

        let strategy = Just(MISSING_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy);

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 3 - confidential input

    // Represents tests: [get_explicit_input_asset_id_with_amount (failing)]
    // Fuzzes rejection of explicit assets and amounts by the confidential getter at input indices.
    #[simplex::fuzz]
    fn get_explicit_input_asset_id_with_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [get_confidential_input_asset_id_with_amount (successful)]
    // Fuzzes matching confidential assets and amounts at input indices.
    #[simplex::fuzz]
    fn get_confidential_input_asset_id_with_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(witness.index as usize);
                witness.index = index as u32;

                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;

                let Asset::Confidential(asset) = output.asset else {
                    panic!("expected a confidential input asset");
                };
                let Value::Confidential(amount) = output.value else {
                    panic!("expected a confidential input amount");
                };

                let (asset_parity, asset_data) = decode_point(asset.serialize());
                let (amount_parity, amount_data) = decode_point(amount.serialize());
                witness.expected_asset_parity = asset_parity;
                witness.expected_conf_asset = asset_data;
                witness.expected_amount_parity = amount_parity;
                witness.expected_conf_amount = amount_data;
            });

        context.engine(scenario).spec(|_| Expect::Accept).run();
        Ok(())
    }

    // index 4 - explicit output

    // Represents tests: [get_explicit_output_asset_id_with_amount (successful)]
    // Fuzzes matching explicit assets and amounts at output indices.
    #[simplex::fuzz]
    fn get_explicit_output_asset_id_with_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
                witness.expected_asset = case.output_asset(index);
                witness.expected_amount = case.output_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [get_explicit_output_asset_id_with_amount_for_confidential_fail]
    // Fuzzes rejection of confidential assets and amounts by the explicit getter at output indices.
    #[simplex::fuzz]
    fn get_explicit_output_asset_id_with_amount_for_confidential_fail(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
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

    // Represents tests: [get_explicit_output_asset_id_with_amount_for_missing_output_fail]
    // Fuzzes transaction layouts and checks rejection of a missing output index.
    #[simplex::fuzz]
    fn get_explicit_output_asset_id_with_amount_for_missing_output_fail(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const MISSING_INDEX: u32 = u32::MAX;

        let strategy = Just(MISSING_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy);

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 5 - confidential output

    // Represents tests: [get_explicit_output_asset_id_with_amount (failing)]
    // Fuzzes rejection of explicit assets and amounts by the confidential getter at output indices.
    #[simplex::fuzz]
    fn get_explicit_output_asset_id_with_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = (0u32..=MAX_UTXO_INDEX).prop_map(|index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.output_index(witness.index as usize);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // index 6 - explicit current input

    // Represents tests: [get_explicit_current_asset_id_with_amount (successful)]
    // Fuzzes matching explicit assets and amounts at the spending input.
    #[simplex::fuzz]
    fn get_explicit_current_asset_id_with_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
                witness.expected_asset = case.input_asset(index);
                witness.expected_amount = case.input_amount(index);
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Accept)
            .run();
        Ok(())
    }

    // Represents tests: [get_confidential_current_asset_id_with_amount (failing)]
    // Fuzzes rejection of confidential assets and amounts by the explicit getter at the spending input.
    #[simplex::fuzz]
    fn get_confidential_current_asset_id_with_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
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

    // Represents tests: [get_explicit_current_asset_id_with_amount (failing)]
    // Fuzzes rejection of explicit assets and amounts by the confidential getter at the spending input.
    #[simplex::fuzz]
    fn get_explicit_current_asset_id_with_amount_failing(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;
            });

        context
            .engine(scenario)
            .mutations(Mutations::none().layout())
            .spec(|_| Expect::Reject)
            .allow_unspendable()
            .run();
        Ok(())
    }

    // Represents tests: [get_confidential_current_asset_id_with_amount (successful)]
    // Fuzzes matching confidential assets and amounts at the spending input.
    #[simplex::fuzz]
    fn get_confidential_current_asset_id_with_amount_successful(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        const UNUSED_WITNESS_INDEX: u32 = 0;

        let strategy = Just(UNUSED_WITNESS_INDEX).prop_map(|unused_index| {
            let arguments: Arguments = TestAssetIdWithAmountArguments {}.into();
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
            .add::<TestAssetIdWithAmountProgram>("utxo getter")
            .spends(SPENDING_INPUT_INDEX)
            .custom_strategy(strategy)
            .bind_witness(|witness, case| {
                let index = case.input_index(SPENDING_INPUT_INDEX);
                witness.index = index as u32;

                let output = &case.transaction().inputs()[index]
                    .partial_input
                    .witness_utxo;

                let Asset::Confidential(asset) = output.asset else {
                    panic!("expected a confidential input asset");
                };
                let Value::Confidential(amount) = output.value else {
                    panic!("expected a confidential input amount");
                };

                let (asset_parity, asset_data) = decode_point(asset.serialize());
                let (amount_parity, amount_data) = decode_point(amount.serialize());
                witness.expected_asset_parity = asset_parity;
                witness.expected_conf_asset = asset_data;
                witness.expected_amount_parity = amount_parity;
                witness.expected_conf_amount = amount_data;
            });

        context.engine(scenario).spec(|_| Expect::Accept).run();
        Ok(())
    }
}
