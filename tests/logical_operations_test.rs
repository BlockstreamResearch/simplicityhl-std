use simplex::simplicityhl::elements::{Script};

use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

use simplicityhl_std::artifacts::mock::logical_operations_mock::LogicalOperationsMockProgram;
use simplicityhl_std::artifacts::mock::logical_operations_mock::derived_logical_operations_mock::{LogicalOperationsMockArguments, LogicalOperationsMockWitness};

fn get_script(context: &simplex::TestContext) -> (LogicalOperationsMockProgram, Script) {
    let arguments = LogicalOperationsMockArguments {};

    let logical_operations_program = LogicalOperationsMockProgram::new(arguments);

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

    let witness = LogicalOperationsMockWitness {};

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
fn logical_operations_test(context: simplex::TestContext) -> anyhow::Result<()> {
    fund_script(&context)?;
    spend_script(&context)?;

    Ok(())
}
