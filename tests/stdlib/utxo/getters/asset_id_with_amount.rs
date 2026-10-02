use primitive_types::U256;
use rand::Rng;

use crate::common::core::Expect;
use crate::common::core::run_w_inputs_outputs;
use crate::common::utxo_helper::ConfidentialAmount;
use crate::common::utxo_helper::ConfidentialAssetId;
use crate::common::utxo_helper::create_utxo_for_inputs;
use crate::common::utxo_helper::from_hex_to_u256_array;

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

const DEFAULT_ASSET_ID: &str = "25b251070e29ca19043cf33ccd7324e2ddab03ecc4ae0b5e77c4fc0e5cf6c95a";

fn program() -> TestAssetIdWithAmountProgram {
    TestAssetIdWithAmountProgram::new(&TestAssetIdWithAmountArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestAssetIdWithAmountWitness,
    index: u32,
    explicit: bool,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestAssetIdWithAmountWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected_asset: [0; 32],
            expected_conf_asset: [0; 32],
            expected_amount: 0,
            expected_conf_amount: [0; 32],
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
    fn expect(mut self, expected_asset: [u8; 32], expected_amount: u64) -> Self {
        self.witness.expected_asset = expected_asset;
        self.witness.expected_amount = expected_amount;
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
    is_input: bool,
    expected_amount: u64,
) -> anyhow::Result<Option<(ConfidentialAssetId, ConfidentialAmount)>> {
    create_utxo_for_inputs(
        context,
        index,
        is_explicit_input,
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
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_asset_id, expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_explicit_asset_id_with_amount_for_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_asset_id, expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_confidential_asset_id_with_amount_for_input(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 2;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (conf_asset, conf_amount) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?
    .unwrap();

    let (asset_parity_bit, asset_conf_amount) = (conf_asset.parity_bit, conf_asset.asset_id);
    let (amount_parity_bit, amount_conf_amount) = (conf_amount.parity_bit, conf_amount.amount);

    case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expect(
            U256::from(asset_parity_bit).to_big_endian(),
            amount_parity_bit,
        )
        .confidentials(asset_conf_amount, amount_conf_amount)
        .run(&context)
}

#[simplex::test]
fn get_explicit_input_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;

    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(ExplicitInput)
        .index(index)
        .expect(expected_asset_id, expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_confidential_input_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let is_explicit_input = false;

    let (conf_asset, conf_amount) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?
    .unwrap();

    let (asset_parity_bit, asset_conf_amount) = (conf_asset.parity_bit, conf_asset.asset_id);
    let (amount_parity_bit, amount_conf_amount) = (conf_amount.parity_bit, conf_amount.amount);

    case(ConfidentialInput)
        .index(index)
        .expect(
            U256::from(asset_parity_bit).to_big_endian(),
            amount_parity_bit,
        )
        .confidentials(asset_conf_amount, amount_conf_amount)
        .run(&context)
}

#[simplex::test]
fn get_explicit_output_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_amount: u64 = rand::thread_rng().gen_range(10..=100) as u64;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Output),
        expected_amount,
    )?;

    case(ExplicitOutput)
        .index(index)
        .expect(expected_asset_id, expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_current_explicit_asset_id_with_amount(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?;

    case(CurrentExplicit)
        .index(index)
        .expect(expected_asset_id, expected_amount)
        .run(&context)
}

#[simplex::test]
fn get_current_confidential_asset_id_with_amount(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let index = 0;
    let expected_amount: u64 = rand::thread_rng().gen_range(50..=100) as u64;
    let is_explicit_input = false;

    let (conf_asset, conf_amount) = create_utxo_for_inputs_wrapper(
        &context,
        index,
        is_explicit_input,
        is_input(IndexType::Input),
        expected_amount,
    )?
    .unwrap();

    let (asset_parity_bit, asset_conf_amount) = (conf_asset.parity_bit, conf_asset.asset_id);
    let (amount_parity_bit, amount_conf_amount) = (conf_amount.parity_bit, conf_amount.amount);

    case(CurrentConfidential)
        .index(index)
        .expect(
            U256::from(asset_parity_bit).to_big_endian(),
            amount_parity_bit,
        )
        .confidentials(asset_conf_amount, amount_conf_amount)
        .explicit(is_explicit_input)
        .run(&context)
}
