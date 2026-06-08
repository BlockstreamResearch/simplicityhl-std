use simplex::simplicityhl::elements::Script;

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::u64_mock::U64MockProgram;
use simplicityhl_std::artifacts::mock::u64_mock::derived_u64_mock::{
    U64MockArguments, U64MockWitness,
};

use rand::Rng;

mod helper;
use crate::helper::{NOT_TEST_OVERFLOW, TEST_OVERFLOW};

enum FunctionToTest {
    CheckedAdd64,
    SafeAdd64,
    CheckedSub64,
    SafeSub64,
    CheckedMul64,
    SafeMul64,
    CheckedDiv64,
    SafeDiv64,
}

fn get_script(context: &simplex::TestContext) -> (U64MockProgram, Script) {
    let arguments = U64MockArguments {};

    let logical_operations_program = U64MockProgram::new(arguments);

    let logical_operations_script =
        logical_operations_program.get_script_pubkey(context.get_network());

    (logical_operations_program, logical_operations_script)
}

fn fund_script(context: &simplex::TestContext) -> anyhow::Result<()> {
    let signer = context.get_default_signer();

    let (_, logical_operations_script) = get_script(context);

    let tx_receipt = signer.send(logical_operations_script.clone(), 50)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

fn spend_script(
    context: &simplex::TestContext,
    function_index: FunctionToTest,
    if_test_overflow: bool,
    first_arg: u64,
    second_arg: u64,
    result: u64,
) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (logical_operations_program, logical_operations_script) = get_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&logical_operations_script)?;

    let mut ft = FinalTransaction::new();

    let witness = U64MockWitness {
        function_index: function_index as u8,
        if_test_overflow,
        first_arg,
        second_arg,
        result,
    };

    ft.add_program_input(
        PartialInput::new(asserts_utxos[0].clone()),
        ProgramInput::new(
            Box::new(logical_operations_program.as_ref().clone()),
            Box::new(witness.clone()),
        ),
        RequiredSignature::None,
    );

    let tx_receipt = signer.broadcast(&ft)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

#[simplex::test]
fn u64_test_checked_add_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX / 2);
    let second_arg = rand::thread_rng().gen_range(0..=u64::MAX / 2);
    let result = first_arg + second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedAdd64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_checked_add_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u64::MAX;
    let second_arg = rand::thread_rng().gen_range(1..=u64::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedAdd64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_add_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX / 2);
    let second_arg = rand::thread_rng().gen_range(0..=u64::MAX / 2);
    let result = first_arg + second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeAdd64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_add_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u64::MAX;
    let second_arg = rand::thread_rng().gen_range(1..=u64::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeAdd64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(
        err,
        "Failed to prune program: Execution reached a pruned branch: 744339c859e7ff6f8d33f9afa73734e1c908684feedc8c4d0a6112d3bf361317"
    );

    Ok(())
}

#[simplex::test]
fn u64_test_checked_sub_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX);
    let second_arg = rand::thread_rng().gen_range(0..=first_arg);
    let result = first_arg - second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedSub64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_checked_sub_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX - 1);
    let second_arg = rand::thread_rng().gen_range(first_arg + 1..=u64::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedSub64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_sub_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX);
    let second_arg = rand::thread_rng().gen_range(0..=first_arg);
    let result = first_arg - second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeSub64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_sub_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX - 1);
    let second_arg = rand::thread_rng().gen_range(first_arg + 1..=u64::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeSub64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(
        err,
        "Failed to prune program: Execution reached a pruned branch: 744339c859e7ff6f8d33f9afa73734e1c908684feedc8c4d0a6112d3bf361317"
    );

    Ok(())
}

#[simplex::test]
fn u64_test_checked_mul_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=2_u64.pow(4));
    let second_arg = rand::thread_rng().gen_range(0..=2_u64.pow(4));
    let result = first_arg * second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedMul64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_checked_mul_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u64::MAX;
    let second_arg = rand::thread_rng().gen_range(2..=u64::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedMul64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_mul_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=2_u64.pow(4));
    let second_arg = rand::thread_rng().gen_range(0..=2_u64.pow(4));
    let result = first_arg * second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeMul64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_mul_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u64::MAX;
    let second_arg = rand::thread_rng().gen_range(2..=u64::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeMul64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(
        err,
        "Failed to prune program: Execution reached a pruned branch: 744339c859e7ff6f8d33f9afa73734e1c908684feedc8c4d0a6112d3bf361317"
    );

    Ok(())
}

#[simplex::test]
fn u64_test_checked_div_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX);
    let second_arg = rand::thread_rng().gen_range(1..=u64::MAX);
    let result = first_arg / second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedDiv64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_checked_div_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX);
    let second_arg = 0;
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedDiv64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_div_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX);
    let second_arg = rand::thread_rng().gen_range(1..=u64::MAX);
    let result = first_arg / second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeDiv64,
        NOT_TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u64_test_safe_div_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u64::MAX);
    let second_arg = 0;
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeDiv64,
        TEST_OVERFLOW,
        first_arg,
        second_arg,
        result,
    );

    assert!(
        txid_result.is_err(),
        "Expected this test to fail but it succeeded"
    );

    let err: String = txid_result.unwrap_err().to_string();
    assert_eq!(
        err,
        "Failed to prune program: Execution reached a pruned branch: 744339c859e7ff6f8d33f9afa73734e1c908684feedc8c4d0a6112d3bf361317"
    );

    Ok(())
}
