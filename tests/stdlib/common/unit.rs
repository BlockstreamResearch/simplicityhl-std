use simplex::program::{Program, ProgramError, ProgramTrait};
use simplex::provider::SimplicityNetwork;
use simplex::simplicityhl::WitnessValues;
use simplex::simplicityhl::elements::{OutPoint, Script, Txid};
use simplex::simplicityhl::simplicity::bit_machine::ExecutionError;
use simplex::transaction::{FinalTransaction, PartialInput, PartialOutput, ProgramInput, UTXO};

use crate::common::core::Expect;

const AMOUNT: u64 = 1_000;

pub fn run(program: &Program, witness: impl Into<WitnessValues>) -> Result<(), ProgramError> {
    let network = SimplicityNetwork::default_regtest();
    let witness = witness.into();

    let (pst, _) = unit_transaction(program, witness.clone(), network)
        .extract_pst()
        .expect("the unit transaction is explicit");

    program.execute(&pst, &witness, 0, &network).map(|_| ())
}

/// A case that fails before the program runs (e.g. a malformed witness) panics instead of
/// matching `expect`.
#[track_caller]
pub fn check(program: &Program, witness: impl Into<WitnessValues>, expect: Expect) {
    match outcome(run(program, witness)) {
        Ok(actual) => assert_eq!(actual, expect, "unexpected program outcome"),
        Err(error) => panic!("the case failed before the program ran: {error}"),
    }
}

fn outcome(result: Result<(), ProgramError>) -> Result<Expect, ProgramError> {
    let execution = match result {
        Ok(()) => return Ok(Expect::Ok),
        Err(ProgramError::Pruning(execution) | ProgramError::Execution(execution)) => execution,
        Err(other) => return Err(other),
    };

    match execution {
        ExecutionError::JetFailed(_) => Ok(Expect::AssertFailed),
        ExecutionError::ReachedPrunedBranch(_) => Ok(Expect::PrunedBranch),
        other => Err(ProgramError::Execution(other)),
    }
}

fn unit_transaction(
    program: &Program,
    witness: WitnessValues,
    network: SimplicityNetwork,
) -> FinalTransaction {
    let asset = network.policy_asset();
    let utxo = UTXO::explicit(
        OutPoint::new(Txid::from_byte_array([0xf0; 32]), 0),
        program.get_script_pubkey(&network),
        AMOUNT,
        asset,
    );

    let mut ft = FinalTransaction::new();
    ft.add_program_input(
        PartialInput::new(utxo),
        ProgramInput::new(Box::new(program.clone()), witness),
        None,
    );
    ft.add_output(PartialOutput::new(Script::from(vec![0x51]), AMOUNT, asset));

    ft
}
