use simplex::simplicityhl::elements::{Script, Txid};

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::asserts_mock::AssertsMockProgram;
use simplicityhl_std::artifacts::mock::asserts_mock::derived_asserts_mock::{AssertsMockWitness, AssertsMockArguments};

fn get_asserts_test_script(context: &simplex::TestContext) -> (AssertsMockProgram, Script) {
    let arguments = AssertsMockArguments {};

    let asserts_program = AssertsMockProgram::new(arguments);

    let asserts_script = asserts_program.get_script_pubkey(context.get_network());

    (asserts_program, asserts_script)
}

fn fund_script(context: &simplex::TestContext) -> anyhow::Result<Txid> {
    let signer = context.get_default_signer();

    let (_, asserts_script) = get_asserts_test_script(context);

    let txid = signer.send(asserts_script.clone(), 50)?;
    println!("Broadcast: {}", txid);

    Ok(txid)
}

fn spend_script(context: &simplex::TestContext) -> anyhow::Result<Txid> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (asserts_program, asserts_script) = get_asserts_test_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&asserts_script)?;

    let mut ft = FinalTransaction::new();

    let witness = AssertsMockWitness {};

    ft.add_program_input(
        PartialInput::new(asserts_utxos[0].clone()),
        ProgramInput::new(Box::new(asserts_program.as_ref().clone()), Box::new(witness.clone())),
        RequiredSignature::None,
    );

    let txid = signer.broadcast(&ft)?;
    println!("Broadcast: {}", txid);

    Ok(txid)
}

#[simplex::test]
fn asserts_test(context: simplex::TestContext) -> anyhow::Result<()> {
    let provider = context.get_default_provider();

    let txid = fund_script(&context)?;

    provider.wait(&txid)?;
    println!("Confirmed");

    let txid = spend_script(&context)?;

    provider.wait(&txid)?;
    println!("Confirmed");

    Ok(())
}
