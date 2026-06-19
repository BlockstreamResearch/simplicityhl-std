use simplex::simplicityhl::elements::Script;

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::asserts_mock::AssertsMockProgram;
use simplicityhl_std::artifacts::asserts_mock::derived_asserts_mock::{
    AssertsMockArguments, AssertsMockWitness,
};

mod helper;
use crate::helper::{cast_to_bool, generate_uints_in_one_range};

enum FunctionToTest {
    AssertEq8,
    AssertEq16,
    AssertEq32,
    AssertEq64,
    AssertEq256,
    AssertNone8,
    AssertNone16,
    AssertNone32,
    AssertNone64,
    AssertNone128,
    AssertNone256,
}

const DEFAULT_SOME_U8: Option<u8> = Some(0);
const DEFAULT_SOME_U16: Option<u16> = Some(0);
const DEFAULT_SOME_U32: Option<u32> = Some(0);
const DEFAULT_SOME_U64: Option<u64> = Some(0);
const DEFAULT_SOME_U128: Option<u128> = Some(0);
const DEFAULT_SOME_U256: Option<[u8; 32]> = Some([0; 32]);

pub enum IfTestSameValues {
    DifferentValues,
    SameValues,
}
pub enum IfTestNoneValue {
    NotNoneValue,
    NoneValue,
}

fn get_asserts_test_script(context: &simplex::TestContext) -> (AssertsMockProgram, Script) {
    let arguments = AssertsMockArguments {};

    let asserts_program = AssertsMockProgram::new(arguments);

    let asserts_script = asserts_program.get_script_pubkey(context.get_network());

    (asserts_program, asserts_script)
}

fn fund_script(context: &simplex::TestContext) -> anyhow::Result<()> {
    let signer = context.get_default_signer();

    let (_, asserts_script) = get_asserts_test_script(context);

    let tx_receipt = signer.send(asserts_script.clone(), 50)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

fn generate_test_witness(
    function_index: FunctionToTest,
    if_same_values: bool,
    if_none_value: bool,
) -> AssertsMockWitness {
    let mut witness: AssertsMockWitness = AssertsMockWitness {
        function_index: 0,
        first_arg_u8: DEFAULT_SOME_U8,
        second_arg_u8: DEFAULT_SOME_U8,
        first_arg_u16: DEFAULT_SOME_U16,
        second_arg_u16: DEFAULT_SOME_U16,
        first_arg_u32: DEFAULT_SOME_U32,
        second_arg_u32: DEFAULT_SOME_U32,
        first_arg_u64: DEFAULT_SOME_U64,
        second_arg_u64: DEFAULT_SOME_U64,
        first_arg_u128: DEFAULT_SOME_U128,
        second_arg_u128: DEFAULT_SOME_U128,
        first_arg_u256: DEFAULT_SOME_U256,
        second_arg_u256: DEFAULT_SOME_U256,
    };

    match function_index {
        FunctionToTest::AssertEq8 => {
            let (first_arg, second_arg) =
                generate_uints_in_one_range(if_same_values, 0, u8::MAX as u128);

            (witness.first_arg_u8, witness.second_arg_u8) =
                (Some(first_arg as u8), Some(second_arg as u8));
        }
        FunctionToTest::AssertEq16 => {
            let (first_arg, second_arg) =
                generate_uints_in_one_range(if_same_values, 0, u16::MAX as u128);

            (witness.first_arg_u16, witness.second_arg_u16) =
                (Some(first_arg as u16), Some(second_arg as u16));
        }
        FunctionToTest::AssertEq32 => {
            let (first_arg, second_arg) =
                generate_uints_in_one_range(if_same_values, 0, u32::MAX as u128);

            (witness.first_arg_u32, witness.second_arg_u32) =
                (Some(first_arg as u32), Some(second_arg as u32));
        }
        FunctionToTest::AssertEq64 => {
            let (first_arg, second_arg) =
                generate_uints_in_one_range(if_same_values, 0, u64::MAX as u128);

            (witness.first_arg_u64, witness.second_arg_u64) =
                (Some(first_arg as u64), Some(second_arg as u64));
        }
        FunctionToTest::AssertEq256 => {
            let (first_arg, second_arg) =
                generate_uints_in_one_range(if_same_values, 0, u8::MAX as u128);

            (witness.first_arg_u256, witness.second_arg_u256) =
                (Some([first_arg as u8; 32]), Some([second_arg as u8; 32]));
        }
        FunctionToTest::AssertNone8 => {
            if if_none_value {
                witness.first_arg_u8 = None;
            };
        }
        FunctionToTest::AssertNone16 => {
            if if_none_value {
                witness.first_arg_u16 = None;
            };
        }
        FunctionToTest::AssertNone32 => {
            if if_none_value {
                witness.first_arg_u32 = None;
            };
        }
        FunctionToTest::AssertNone64 => {
            if if_none_value {
                witness.first_arg_u64 = None;
            };
        }
        FunctionToTest::AssertNone128 => {
            if if_none_value {
                witness.first_arg_u128 = None;
            };
        }
        FunctionToTest::AssertNone256 => {
            if if_none_value {
                witness.first_arg_u256 = None;
            };
        }
    }

    witness.function_index = function_index as u8;
    witness
}

fn spend_script(
    context: &simplex::TestContext,
    function_index: FunctionToTest,
    if_same_values: IfTestSameValues,
    if_none_value: IfTestNoneValue,
) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (asserts_program, asserts_script) = get_asserts_test_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&asserts_script)?;

    let mut ft = FinalTransaction::new();

    let witness: AssertsMockWitness = generate_test_witness(
        function_index,
        cast_to_bool(if_same_values as u8),
        cast_to_bool(if_none_value as u8),
    );

    ft.add_program_input(
        PartialInput::new(asserts_utxos[0].clone()),
        ProgramInput::new(
            Box::new(asserts_program.as_ref().clone()),
            Box::new(witness.clone()),
        ),
        RequiredSignature::None,
    );

    let tx_receipt = signer.broadcast(&ft)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

#[simplex::test]
fn assert_eq_8_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq8,
        IfTestSameValues::SameValues,
        IfTestNoneValue::NotNoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq8,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_eq_16_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq16,
        IfTestSameValues::SameValues,
        IfTestNoneValue::NotNoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq16,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_eq_32_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq32,
        IfTestSameValues::SameValues,
        IfTestNoneValue::NotNoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq32,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_eq_64_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq64,
        IfTestSameValues::SameValues,
        IfTestNoneValue::NotNoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq64,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_eq_256_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq256,
        IfTestSameValues::SameValues,
        IfTestNoneValue::NotNoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq256,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_none_8_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertNone8,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone8,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_none_16_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    spend_script(
        &context,
        FunctionToTest::AssertNone16,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone16,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_none_32_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    spend_script(
        &context,
        FunctionToTest::AssertNone32,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone32,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_none_64_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    spend_script(
        &context,
        FunctionToTest::AssertNone64,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone64,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_none_128_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    spend_script(
        &context,
        FunctionToTest::AssertNone128,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_128_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone128,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}

#[simplex::test]
fn assert_none_256_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    spend_script(
        &context,
        FunctionToTest::AssertNone256,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NoneValue,
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone256,
        IfTestSameValues::DifferentValues,
        IfTestNoneValue::NotNoneValue,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}
