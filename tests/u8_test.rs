use simplex::simplicityhl::elements::Script;

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::u8_mock::U8MockProgram;
use simplicityhl_std::artifacts::mock::u8_mock::derived_u8_mock::{U8MockArguments, U8MockWitness};

use rand::Rng;

mod helper;
use crate::helper::{IfTestOverflow, cast_to_bool};

enum FunctionToTest {
    CheckedAdd8,
    SafeAdd8,
    CheckedSub8,
    SafeSub8,
    CheckedMul8,
    SafeMul8,
    CheckedDiv8,
    SafeDiv8,
}

fn get_script(context: &simplex::TestContext) -> (U8MockProgram, Script) {
    let arguments = U8MockArguments {};

    let logical_operations_program = U8MockProgram::new(arguments);

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
    if_test_overflow: IfTestOverflow,
    first_arg: u8,
    second_arg: u8,
    result: u8,
) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (logical_operations_program, logical_operations_script) = get_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&logical_operations_script)?;

    let mut ft = FinalTransaction::new();

    let witness = U8MockWitness {
        function_index: function_index as u8,
        if_test_overflow: cast_to_bool(if_test_overflow as u8),
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
fn u8_test_checked_add_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX / 2);
    let second_arg = rand::thread_rng().gen_range(0..=u8::MAX / 2);
    let result = first_arg + second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedAdd8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_checked_add_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u8::MAX;
    let second_arg = rand::thread_rng().gen_range(1..=u8::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedAdd8,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_add_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX / 2);
    let second_arg = rand::thread_rng().gen_range(0..=u8::MAX / 2);
    let result = first_arg + second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeAdd8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_add_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u8::MAX;
    let second_arg = rand::thread_rng().gen_range(1..=u8::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeAdd8,
        IfTestOverflow::Overflow,
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
fn u8_test_checked_sub_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX);
    let second_arg = rand::thread_rng().gen_range(0..=first_arg);
    let result = first_arg - second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedSub8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_checked_sub_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX - 1);
    let second_arg = rand::thread_rng().gen_range(first_arg + 1..=u8::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedSub8,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_sub_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX);
    let second_arg = rand::thread_rng().gen_range(0..=first_arg);
    let result = first_arg - second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeSub8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_sub_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX - 1);
    let second_arg = rand::thread_rng().gen_range(first_arg + 1..=u8::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeSub8,
        IfTestOverflow::Overflow,
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
fn u8_test_checked_mul_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..2_u8.pow(8/2));
    let second_arg = rand::thread_rng().gen_range(0..2_u8.pow(8/2));
    let result = first_arg * second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedMul8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_checked_mul_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u8::MAX;
    let second_arg = rand::thread_rng().gen_range(2..=u8::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedMul8,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_mul_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..2_u8.pow(8/2));
    let second_arg = rand::thread_rng().gen_range(0..2_u8.pow(8/2));
    let result = first_arg * second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeMul8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_mul_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u8::MAX;
    let second_arg = rand::thread_rng().gen_range(2..=u8::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeMul8,
        IfTestOverflow::Overflow,
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
fn u8_test_checked_div_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX);
    let second_arg = rand::thread_rng().gen_range(1..=u8::MAX);
    let result = first_arg / second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedDiv8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_checked_div_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX);
    let second_arg = 0;
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedDiv8,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_div_8_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX);
    let second_arg = rand::thread_rng().gen_range(1..=u8::MAX);
    let result = first_arg / second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeDiv8,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u8_test_safe_div_8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u8::MAX);
    let second_arg = 0;
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeDiv8,
        IfTestOverflow::Overflow,
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
