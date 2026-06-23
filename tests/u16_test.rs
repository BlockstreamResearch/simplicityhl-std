use simplex::simplicityhl::elements::Script;

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::u16_mock::U16MockProgram;
use simplicityhl_std::artifacts::mock::u16_mock::derived_u16_mock::{
    U16MockArguments, U16MockWitness,
};

use rand::Rng;

mod helper;
use crate::helper::{IfTestOverflow, cast_to_bool};

enum FunctionToTest {
    CheckedAdd16,
    SafeAdd16,
    CheckedSub16,
    SafeSub16,
    CheckedMul16,
    SafeMul16,
    CheckedDiv16,
    SafeDiv16,
}

fn get_script(context: &simplex::TestContext) -> (U16MockProgram, Script) {
    let arguments = U16MockArguments {};

    let u16_program = U16MockProgram::new(arguments);

    let u16_script = u16_program.get_script_pubkey(context.get_network());

    (u16_program, u16_script)
}

fn fund_script(context: &simplex::TestContext) -> anyhow::Result<()> {
    let signer = context.get_default_signer();

    let (_, u16_script) = get_script(context);

    let tx_receipt = signer.send(u16_script.clone(), 50)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

fn spend_script(
    context: &simplex::TestContext,
    function_index: FunctionToTest,
    if_test_overflow: IfTestOverflow,
    first_arg: u16,
    second_arg: u16,
    result: u16,
) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (u16_program, u16_script) = get_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&u16_script)?;

    let mut ft = FinalTransaction::new();

    let witness = U16MockWitness {
        function_index: function_index as u8,
        if_test_overflow: cast_to_bool(if_test_overflow as u8),
        first_arg,
        second_arg,
        result,
    };

    ft.add_program_input(
        PartialInput::new(asserts_utxos[0].clone()),
        ProgramInput::new(
            Box::new(u16_program.as_ref().clone()),
            Box::new(witness.clone()),
        ),
        RequiredSignature::None,
    );

    let tx_receipt = signer.broadcast(&ft)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

#[simplex::test]
fn u16_test_checked_add_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX / 2);
    let second_arg = rand::thread_rng().gen_range(0..=u16::MAX / 2);
    let result = first_arg + second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedAdd16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_checked_add_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u16::MAX;
    let second_arg = rand::thread_rng().gen_range(1..=u16::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedAdd16,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_add_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX / 2);
    let second_arg = rand::thread_rng().gen_range(0..=u16::MAX / 2);
    let result = first_arg + second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeAdd16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_add_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u16::MAX;
    let second_arg = rand::thread_rng().gen_range(1..=u16::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeAdd16,
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
fn u16_test_checked_sub_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX);
    let second_arg = rand::thread_rng().gen_range(0..=first_arg);
    let result = first_arg - second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedSub16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_checked_sub_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX - 1);
    let second_arg = rand::thread_rng().gen_range(first_arg + 1..=u16::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedSub16,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_sub_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX);
    let second_arg = rand::thread_rng().gen_range(0..=first_arg);
    let result = first_arg - second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeSub16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_sub_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX - 1);
    let second_arg = rand::thread_rng().gen_range(first_arg + 1..=u16::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeSub16,
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
fn u16_test_checked_mul_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..2_u16.pow(16 / 2));
    let second_arg = rand::thread_rng().gen_range(0..2_u16.pow(16 / 2));
    let result = first_arg * second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedMul16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_checked_mul_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u16::MAX;
    let second_arg = rand::thread_rng().gen_range(2..=u16::MAX);
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedMul16,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_mul_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..2_u16.pow(16 / 2));
    let second_arg = rand::thread_rng().gen_range(0..2_u16.pow(16 / 2));
    let result = first_arg * second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeMul16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_mul_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = u16::MAX;
    let second_arg = rand::thread_rng().gen_range(2..=u16::MAX);
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeMul16,
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
fn u16_test_checked_div_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX);
    let second_arg = rand::thread_rng().gen_range(1..=u16::MAX);
    let result = first_arg / second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedDiv16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_checked_div_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX);
    let second_arg = 0;
    let result = 0;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::CheckedDiv16,
        IfTestOverflow::Overflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_div_16_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX);
    let second_arg = rand::thread_rng().gen_range(1..=u16::MAX);
    let result = first_arg / second_arg;

    fund_script(&context)?;
    spend_script(
        &context,
        FunctionToTest::SafeDiv16,
        IfTestOverflow::NotOverflow,
        first_arg,
        second_arg,
        result,
    )?;

    Ok(())
}

#[simplex::test]
fn u16_test_safe_div_16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let first_arg = rand::thread_rng().gen_range(0..=u16::MAX);
    let second_arg = 0;
    let result = 0;

    fund_script(&context)?;

    let txid_result = spend_script(
        &context,
        FunctionToTest::SafeDiv16,
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
