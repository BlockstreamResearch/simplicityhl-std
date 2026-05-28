use simplex::simplicityhl::elements::{Script};

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::u8_mock::U8MockProgram;
use simplicityhl_std::artifacts::mock::u8_mock::derived_u8_mock::{U8MockArguments, U8MockWitness};

fn get_script(context: &simplex::TestContext) -> (U8MockProgram, Script) {
    let arguments = U8MockArguments {};

    let logical_operations_program = U8MockProgram::new(arguments);

    let logical_operations_script = logical_operations_program.get_script_pubkey(context.get_network());

    (logical_operations_program, logical_operations_script)
}

fn fund_script(context: &simplex::TestContext) -> anyhow::Result<()> {
    let signer = context.get_default_signer();

    let (_, logical_operations_script) = get_script(context);

    let tx_receipt = signer.send(logical_operations_script.clone(), 50)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(()) 
}

fn spend_script(context: &simplex::TestContext) -> anyhow::Result<()> {
    let signer = context.get_default_signer();
    let provider = context.get_default_provider();

    let (logical_operations_program, logical_operations_script) = get_script(context);

    let asserts_utxos = provider.fetch_scripthash_utxos(&logical_operations_script)?;

    let mut ft = FinalTransaction::new();

    let witness = U8MockWitness {};

    ft.add_program_input(
        PartialInput::new(asserts_utxos[0].clone()),
        ProgramInput::new(Box::new(logical_operations_program.as_ref().clone()), Box::new(witness.clone())),
        RequiredSignature::None,
    );

    let tx_receipt = signer.broadcast(&ft)?;
    println!("Broadcast: {}", tx_receipt);

    Ok(())
}

#[simplex::test]
fn u8_test(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(&context)?;

    Ok(())
}
