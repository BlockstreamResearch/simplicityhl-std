use simplex::simplicityhl::elements::Script;

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::asserts_mock::AssertsMockProgram;
use simplicityhl_std::artifacts::mock::asserts_mock::derived_asserts_mock::{
    AssertsMockArguments, AssertsMockWitness,
};

use rand::Rng;

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

fn spend_script(
    context: &simplex::TestContext,
    function_index: FunctionToTest,
    first_arg_u8: Option<u8>,
    second_arg_u8: Option<u8>,
    first_arg_u16: Option<u16>,
    second_arg_u16: Option<u16>,
    first_arg_u32: Option<u32>,
    second_arg_u32: Option<u32>,
    first_arg_u64: Option<u64>,
    second_arg_u64: Option<u64>,
    first_arg_u128: Option<u128>,
    second_arg_u128: Option<u128>,
    first_arg_u256: Option<[u8; 32]>,
    second_arg_u256: Option<[u8; 32]>,
) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (asserts_program, asserts_script) = get_asserts_test_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&asserts_script)?;

    let mut ft = FinalTransaction::new();

    let witness = AssertsMockWitness {
        function_index: function_index as u8,
        first_arg_u8,
        second_arg_u8,
        first_arg_u16,
        second_arg_u16,
        first_arg_u32,
        second_arg_u32,
        first_arg_u64,
        second_arg_u64,
        first_arg_u128,
        second_arg_u128,
        first_arg_u256,
        second_arg_u256,
    };

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
    let some_u8 = rand::thread_rng().gen_range(0..=u8::MAX);

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq8,
        Some(some_u8),
        Some(some_u8),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let some_u8 = rand::thread_rng().gen_range(1..=u8::MAX);

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq8,
        Some(some_u8),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
    let some_u16: u16 = rand::thread_rng().gen_range(1..=u16::MAX);

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq16,
        Some(0),
        Some(0),
        Some(some_u16),
        Some(some_u16),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let some_u16: u16 = rand::thread_rng().gen_range(1..=u16::MAX);

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq16,
        Some(0),
        Some(0),
        Some(some_u16),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
    let some_u32: u32 = rand::thread_rng().gen_range(1..=u32::MAX);

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq32,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(some_u32),
        Some(some_u32),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let some_u32: u32 = rand::thread_rng().gen_range(1..=u32::MAX);

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq32,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(some_u32),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
    let some_u64: u64 = rand::thread_rng().gen_range(1..=u64::MAX);

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq64,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(some_u64),
        Some(some_u64),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let some_u64: u64 = rand::thread_rng().gen_range(1..=u64::MAX);

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq64,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(some_u64),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
    let some_u8: u8 = rand::thread_rng().gen_range(1..=u8::MAX);

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::AssertEq256,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([some_u8; 32]),
        Some([some_u8; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_eq_256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let some_u8: u8 = rand::thread_rng().gen_range(1..=u8::MAX);

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertEq256,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([some_u8; 32]),
        Some([0; 32]),
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
        None,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone8,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
        Some(0),
        Some(0),
        None,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone16,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        None,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone32,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        None,
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone64,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        None,
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_128_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone128,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
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
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        None,
        Some([0; 32]),
    )?;

    Ok(())
}

#[simplex::test]
fn assert_none_256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::AssertNone256,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some([0; 32]),
        Some([0; 32]),
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(err, "Failed to prune program: Jet failed during execution");

    Ok(())
}
