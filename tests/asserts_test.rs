use simplex::simplicityhl::elements::{Script};

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::asserts_mock::AssertsMockProgram;
use simplicityhl_std::artifacts::mock::asserts_mock::derived_asserts_mock::{AssertsMockWitness, AssertsMockArguments};

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

fn spend_script(context: &simplex::TestContext, flag: u8) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (asserts_program, asserts_script) = get_asserts_test_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&asserts_script)?;

    let mut ft = FinalTransaction::new();

    let witness = AssertsMockWitness {flag: flag};

    ft.add_program_input(
        PartialInput::new(asserts_utxos[0].clone()),
        ProgramInput::new(Box::new(asserts_program.as_ref().clone()), Box::new(witness.clone())),
        RequiredSignature::None,
    );

    let tx_receipt = signer.broadcast(&ft)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

#[simplex::test]
fn asserts_test_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 0;

    fund_script(&context)?;
    spend_script(&context, flag)?;

    Ok(())
}

#[simplex::test]
fn assert_eq8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 1;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_eq16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 2;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_eq32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 3;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_eq64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 4;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_eq256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 5;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_none8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 6;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_none16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 7;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_none32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 8;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}

#[simplex::test]
fn assert_none64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 9;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}


#[simplex::test]
fn assert_none128_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 10;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}


#[simplex::test]
fn assert_none256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    let flag: u8 = 11;

    fund_script(&context)?;

    let txid_result = spend_script(&context, flag);

    assert!(
        txid_result.is_err(),
        "Expected a test to fail but it succeeded"
    );

    Ok(())
}
