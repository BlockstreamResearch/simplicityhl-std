use rand::Rng;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::run_with_inputs_outputs;
use crate::common::utxo_helper::create_utxos_and_return_script_hash;

use simplicityhl_std::artifacts::tests::utxo::getters::script_hash::ScriptHashProgram as TestScriptHashProgram;
use simplicityhl_std::artifacts::tests::utxo::getters::script_hash::derived_script_hash::{
    ScriptHashArguments as TestScriptHashArguments, ScriptHashWitness as TestScriptHashWitness,
};

use simplicityhl_std::artifacts::tests::mock::MockProgram as TestMockProgram;
use simplicityhl_std::artifacts::tests::mock::derived_mock::MockArguments as TestMockArguments;

use FunctionToTest::*;

enum FunctionToTest {
    Script,
    InputScript,
    OutputScript,
}
#[derive(PartialEq, Eq)]
enum IndexType {
    Input,
    Output,
}

fn is_input(o: IndexType) -> bool {
    o == IndexType::Input
}

fn program() -> TestScriptHashProgram {
    TestScriptHashProgram::new(&TestScriptHashArguments {})
}

fn mock() -> TestMockProgram {
    TestMockProgram::new(&TestMockArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestScriptHashWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestScriptHashWitness {
            function_index: function as u8,
            index: 0,
            is_input_index: false,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// `index`: the input/output the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.index = index;
        self
    }

    /// `is_input`: whether the arm inspects an input.
    fn index_type(mut self, index_type: IndexType) -> Self {
        self.witness.is_input_index = is_input(index_type);
        self
    }

    /// `expected`: value the arm should produce.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = expected;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext, txids: Vec<Txid>) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok, txids)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(
        self,
        context: &simplex::TestContext,
        expect: Expect,
        txids: Vec<Txid>,
    ) -> anyhow::Result<()> {
        run_with_inputs_outputs(
            context,
            program(),
            self.witness,
            expect,
            txids,
            Some(mock().as_ref().clone()),
        )
    }
}

fn create_utxos_wrapper(
    context: &simplex::TestContext,
    index: u32,
    is_input: bool,
) -> anyhow::Result<([u8; 32], Vec<Txid>)> {
    let last_script = &mock().as_ref().get_script_pubkey(context.get_network());
    let (hash, txids) =
        create_utxos_and_return_script_hash(context, index, is_input, last_script, program())?;

    Ok((hash, txids))
}

#[simplex::test]
fn get_script_hash_for_random_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32; // not a big value to not to slow down tests
    let (expected_hash, txids) = create_utxos_wrapper(&context, index, is_input(IndexType::Input))?;

    case(Script)
        .index(index)
        .index_type(IndexType::Input)
        .expect(expected_hash)
        .run(&context, txids)
}

#[simplex::test]
fn get_script_hash_for_random_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=20) as u32; // not a big value to not to slow down tests
    let (expected_hash, txids) =
        create_utxos_wrapper(&context, index, is_input(IndexType::Output))?;

    case(Script)
        .index(index)
        .index_type(IndexType::Output)
        .expect(expected_hash)
        .run(&context, txids)
}

#[simplex::test]
fn get_script_hash_for_missing_input_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(&context, 0, is_input(IndexType::Input))?;

    case(Script)
        .index(u32::MAX)
        .index_type(IndexType::Input)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_script_hash_for_missing_output_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(&context, 0, is_input(IndexType::Output))?;

    case(Script)
        .index(u32::MAX)
        .index_type(IndexType::Output)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_input_script_hash(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=3) as u32;
    let (expected_hash, txids) = create_utxos_wrapper(&context, index, is_input(IndexType::Input))?;

    case(InputScript)
        .index(index)
        .expect(expected_hash)
        .run(&context, txids)
}

#[simplex::test]
fn get_input_script_hash_missing_input_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(&context, 0, is_input(IndexType::Input))?;

    case(InputScript)
        .index(u32::MAX)
        .expecting(&context, Expect::PrunedBranch, txids)
}

#[simplex::test]
fn get_output_script_hash(context: simplex::TestContext) -> anyhow::Result<()> {
    let index = rand::thread_rng().gen_range(0..=3) as u32;

    let (expected_hash, txids) =
        create_utxos_wrapper(&context, index, is_input(IndexType::Output))?;

    case(OutputScript)
        .index(index)
        .expect(expected_hash)
        .run(&context, txids)
}

#[simplex::test]
fn get_output_script_hash_missing_output_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let (_, txids) = create_utxos_wrapper(&context, 0, is_input(IndexType::Output))?;

    case(OutputScript)
        .index(u32::MAX)
        .expecting(&context, Expect::PrunedBranch, txids)
}
