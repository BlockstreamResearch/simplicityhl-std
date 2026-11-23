use simplex::program::{Program, ProgramError, ProgramTrait};
use simplex::provider::SimplicityNetwork;
use simplex::simplicityhl::elements::{OutPoint, Script, Txid};
use simplex::simplicityhl::simplicity::bit_machine::ExecutionError;
use simplex::simplicityhl::{WitnessValues, elements};
use simplex::transaction::{FinalTransaction, PartialInput, PartialOutput, ProgramInput, UTXO};

use crate::common::core::Expect;

const AMOUNT: u64 = 1_000;
const OP_PUSHNUM_1: elements::opcodes::All = elements::opcodes::all::OP_PUSHNUM_1;

#[derive(Clone, Debug, Default)]
pub struct UnitTx {
    pub leading_outputs: Vec<PartialOutput>,
}

pub fn run(
    program: &Program,
    witness: impl Into<WitnessValues>,
    tx: UnitTx,
) -> Result<(), ProgramError> {
    let network = SimplicityNetwork::default_regtest();
    let witness = witness.into();

    let (pst, _) = unit_transaction(program, witness.clone(), tx, network)
        .extract_pst()
        .expect("the unit transaction is explicit");

    program.execute(&pst, &witness, 0, &network).map(|_| ())
}

/// A case that fails before the program runs (e.g. a malformed witness) panics instead of
/// matching `expect`.
#[track_caller]
pub fn check(program: &Program, witness: impl Into<WitnessValues>, expect: Expect) {
    check_in(program, witness, UnitTx::default(), expect);
}

/// A case that fails before the program runs (e.g. a malformed witness) panics instead of
/// matching `expect`. Uses custom `UnitTx` for pattern matching.
#[track_caller]
pub fn check_in(program: &Program, witness: impl Into<WitnessValues>, tx: UnitTx, expect: Expect) {
    match outcome(run(program, witness, tx)) {
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
    tx: UnitTx,
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

    for output in tx.leading_outputs {
        ft.add_output(output);
    }

    ft.add_output(PartialOutput::new(
        Script::from(vec![OP_PUSHNUM_1.into_u8()]),
        AMOUNT,
        asset,
    ));

    ft
}
