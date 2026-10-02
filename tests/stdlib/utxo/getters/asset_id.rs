use crate::common::core::Expect;
use crate::common::core::run_w_inputs_outputs;
use crate::common::utxo_helper::create_utxo_for_inputs;
use crate::common::utxo_helper::from_hex_to_u256_array;

use primitive_types::U256;
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

const DEFAULT_SEND_AMOUNT: u64 = 50;
const DEFAULT_EXPECTED: [u8; 32] = [0; 32];
const DEFAULT_ASSET_ID: &str = "25b251070e29ca19043cf33ccd7324e2ddab03ecc4ae0b5e77c4fc0e5cf6c95a";

fn program() -> TestAssetIdProgram {
    TestAssetIdProgram::new(&TestAssetIdArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestAssetIdWitness,
    index: u32,
    explicit: bool,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestAssetIdWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected: DEFAULT_EXPECTED,
            second_expected: DEFAULT_EXPECTED,
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
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = expected;
        self
    }

    /// `second_expected`: the arm's second result.
    fn confidential_asset_id(mut self, second_expected: [u8; 32]) -> Self {
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
        let index = self.index;

        run_w_inputs_outputs(
            context,
            program(),
            self.witness,
            expect,
            index,
            self.explicit,
        )
    }
}

fn create_utxo_for_inputs_wrapper(
    context: &simplex::TestContext,
    index: u32,
    is_explicit_input: bool,
) -> anyhow::Result<Option<(u64, [u8; 32])>> {
    let result = create_utxo_for_inputs(
        context,
        index,
        is_explicit_input,
        DEFAULT_SEND_AMOUNT,
        program(),
    )?;

    if is_explicit_input {
        Ok(None)
    } else {
        let (conf_asset, _) = result.unwrap();

        Ok(Some((conf_asset.parity_bit, conf_asset.asset_id)))
    }
}

#[simplex::test]
fn get_explicit_asset_id_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_asset_id)
        .run(&context)
}

#[simplex::test]
fn get_explicit_asset_id_for_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?;

    case(Explicit)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_asset_id)
        .run(&context)
}

#[simplex::test]
fn get_confidential_asset_id_for_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let is_explicit_input = false;

    let (expected_parity_bit, expected_conf_asset_id) =
        create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?.unwrap();

    case(Confidential)
        .index(index)
        .index_type(IndexType::Input)
        .expect(U256::from(expected_parity_bit).to_big_endian())
        .confidential_asset_id(expected_conf_asset_id)
        .explicit(false)
        .run(&context)
}

#[simplex::test]
fn get_explicit_input_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?;

    case(ExplicitInput)
        .index(index)
        .expect(expected_asset_id)
        .run(&context)
}

#[simplex::test]
fn get_confidential_input_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 1;
    let is_explicit_input = false;

    let (expected_parity_bit, expected_conf_asset_id) =
        create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?.unwrap();

    case(ConfidentialInput)
        .index(index)
        .expect(U256::from(expected_parity_bit).to_big_endian())
        .confidential_asset_id(expected_conf_asset_id)
        .run(&context)
}

#[simplex::test]
fn get_explicit_output_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 2;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?;

    case(ExplicitOutput)
        .index(index)
        .expect(expected_asset_id)
        .run(&context)
}

#[simplex::test]
fn get_current_explicit_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let expected_asset_id: [u8; 32] = from_hex_to_u256_array(DEFAULT_ASSET_ID)?;
    let is_explicit_input = true;

    create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?;

    case(CurrentExplicit)
        .index(index)
        .expect(expected_asset_id)
        .run(&context)
}

#[simplex::test]
fn get_current_confidential_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = 0;
    let is_explicit_input = false;

    let (expected_parity_bit, expected_conf_asset_id) =
        create_utxo_for_inputs_wrapper(&context, index, is_explicit_input)?.unwrap();

    case(CurrentConfidential)
        .index(index)
        .expect(U256::from(expected_parity_bit).to_big_endian())
        .confidential_asset_id(expected_conf_asset_id)
        .explicit(is_explicit_input)
        .run(&context)
}
