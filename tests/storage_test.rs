mod common;

use simplex::simplicityhl::elements::{Script, Sequence};
use simplex::transaction::PartialOutput;

use common::core::{assert_error_msg, construct_final_tx, fund, run, run_with_outputs, Expect};

use simplicityhl_std::artifacts::op_return_test::OpReturnTestProgram;
use simplicityhl_std::artifacts::op_return_test::derived_op_return_test::OpReturnTestArguments;
use simplicityhl_std::artifacts::storage_test::StorageTestProgram;
use simplicityhl_std::artifacts::storage_test::derived_storage_test::{
    StorageTestArguments, StorageTestWitness,
};

// Dispatch indices — must match the `if_test_this_function(N, ..)` arms in
// simf/storage_test.simf.
enum FunctionToTest {
    Load,
    Store,
    Transition, // load + store together, in one spend
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

// Two distinct 32-byte state values, used throughout to distinguish "the state that's
// really committed" from "the state a witness falsely claims."
const STATE_A: [u8; 32] = [0xAA; 32];
const STATE_B: [u8; 32] = [0xBB; 32];

fn program() -> StorageTestProgram {
    StorageTestProgram::new(StorageTestArguments {})
}

fn build_load_witness(state_data: [u8; 32]) -> StorageTestWitness {
    StorageTestWitness {
        function_index: op(FunctionToTest::Load),
        state_data,
        new_state: [0; 32],
        index: 0,
    }
}

fn build_store_witness(new_state: [u8; 32], index: u32) -> StorageTestWitness {
    StorageTestWitness {
        function_index: op(FunctionToTest::Store),
        state_data: [0; 32],
        new_state,
        index,
    }
}

fn build_transition_witness(state_data: [u8; 32], new_state: [u8; 32], index: u32) -> StorageTestWitness {
    StorageTestWitness {
        function_index: op(FunctionToTest::Transition),
        state_data,
        new_state,
        index,
    }
}

mod storage_tests {
    use super::*;

    // ---------- load ----------

    #[simplex::test]
    fn load_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
        // Fund a UTXO that commits to STATE_A, then load exactly that.
        let mut funded = program().with_storage_capacity(1);
        funded.set_storage_at(0, STATE_A);

        run(&context, funded, build_load_witness(STATE_A), Expect::Ok)
    }

    #[simplex::test]
    fn load_wrong_state_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (1) The UTXO commits to STATE_A, but the witness claims STATE_B.
        let mut funded = program().with_storage_capacity(1);
        funded.set_storage_at(0, STATE_A);

        run(
            &context,
            funded,
            build_load_witness(STATE_B),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn load_no_commitment_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (2) The UTXO is a bare instance of this contract, with no state leaf at all --
        // no claimed state can ever match, since `own_script_hash_with_state` always
        // computes a 2-leaf tree, never a bare single-leaf one.
        run(
            &context,
            program(),
            build_load_witness(STATE_A),
            Expect::AssertFailed,
        )
    }

    // ---------- store ----------

    #[simplex::test]
    fn store_happy_path_index_0(context: simplex::TestContext) -> anyhow::Result<()> {
        let mut target = program().with_storage_capacity(1);
        target.set_storage_at(0, STATE_A);
        let target_script = target.as_ref().get_script_pubkey(context.get_network());

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 0),
            vec![(target_script, 50)],
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn store_happy_path_nonzero_index(context: simplex::TestContext) -> anyhow::Result<()> {
        // Same as above, but the real target is output index 1, behind a dummy filler
        // output at index 0 -- exercises that `index` is a genuine parameter, not just
        // always 0.
        let dummy_script = program().as_ref().get_script_pubkey(context.get_network());

        let mut target = program().with_storage_capacity(1);
        target.set_storage_at(0, STATE_A);
        let target_script = target.as_ref().get_script_pubkey(context.get_network());

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 1),
            vec![(dummy_script, 50), (target_script, 50)],
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn store_wrong_state_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (3) The output actually commits to STATE_A, but the witness claims STATE_B.
        let mut target = program().with_storage_capacity(1);
        target.set_storage_at(0, STATE_A);
        let target_script = target.as_ref().get_script_pubkey(context.get_network());

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_B, 0),
            vec![(target_script, 50)],
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn store_no_commitment_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (4) The output is a bare instance of this contract (no state leaf at all).
        let bare_script = program().as_ref().get_script_pubkey(context.get_network());

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 0),
            vec![(bare_script, 50)],
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn store_nonexistent_index_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (5) The claimed output index is far beyond the transaction's real output
        // count -> `jet::output_script_hash` returns `None` -> `unwrap` hits the pruned
        // branch.
        let mut target = program().with_storage_capacity(1);
        target.set_storage_at(0, STATE_A);
        let target_script = target.as_ref().get_script_pubkey(context.get_network());

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 99),
            vec![(target_script, 50)],
            Expect::PrunedBranch,
        )
    }

    #[simplex::test]
    fn store_wrong_cmr_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (6) The output commits to the *right* state value, but paired with a
        // *different* contract's CMR (here, `OpReturnTestProgram`, reused purely as a
        // conveniently different already-compiled program) -- proving `store` verifies
        // which contract as well as which state, not state alone.
        let mut other = OpReturnTestProgram::new(OpReturnTestArguments {}).with_storage_capacity(1);
        other.set_storage_at(0, STATE_A);
        let other_script = other.as_ref().get_script_pubkey(context.get_network());

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 0),
            vec![(other_script, 50)],
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn store_wrong_taproot_leaf_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (7) The output *is* a genuine Taproot output -- just not a Simplicity one. It's a
        // real BIP-341 P2TR output whose sole leaf is an ordinary Bitcoin Script (OP_TRUE),
        // combined with the same NUMS internal key our own contracts use, at the standard
        // Elements Tapscript leaf version (0xc4, not Bitcoin's 0xc0). This scriptPubkey was
        // computed once, out of band, with a throwaway generator built on the same taproot
        // APIs `Program` itself uses (`TaprootBuilder`, `finalize`, `Address::p2tr`), then
        // hardcoded here -- so the test doesn't need to reconstruct real BIP-341 tweaking
        // machinery just to exercise this one case.
        //
        // `jet::output_script_hash` hashes the raw scriptPubKey bytes, whatever they are.
        // So the hash here is `Some(...)`, just the wrong one, and `store`'s `assert!`
        // fails on the mismatch -- unlike `store_nonexistent_index_fails`, this never reaches
        // the pruned-branch case.

        let alternate_leaf_script = Script::from(vec![
            0x51, 0x20, 0xe4, 0xd6, 0x4b, 0x74, 0x54, 0x4b, 0xe8, 0x37, 0x4e, 0x44, 0x98, 0x19,
            0xb5, 0x1c, 0xde, 0xe2, 0x9f, 0xec, 0x36, 0x3a, 0x71, 0xa8, 0x5a, 0x42, 0xb0, 0x9d,
            0xbe, 0xb7, 0x92, 0xfc, 0x53, 0xf8,
        ]);

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 0),
            vec![(alternate_leaf_script, 50)],
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn store_non_taproot_output_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // (8) The output isn't a Taproot output at all -- a plain P2WPKH (SegWit v0)
        // scriptPubkey, `OP_0 <20-byte-hash>`. The scriptPubKey is still wrong for our
        // store() purposes, as this is another way in which an output can be "not us".
        let p2wpkh_script = Script::from(vec![
            0x00, 0x14, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
            0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44,
        ]);

        run_with_outputs(
            &context,
            program(),
            build_store_witness(STATE_A, 0),
            vec![(p2wpkh_script, 50)],
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn state_chain_store_load_store_load(context: simplex::TestContext) -> anyhow::Result<()> {
        // Exercises a genuine multi-transaction state chain: genesis (store only) ->
        // transition (load + store, consuming the prior commitment and creating the
        // next one) -> final consumption (load only). Each step spends the *real*
        // output the previous step created, looked up by script -- safe here because
        // every state value is unique, so there's no ambiguity about which UTXO to grab.
        const STATE_1: [u8; 32] = [0x11; 32];
        const STATE_2: [u8; 32] = [0x22; 32];

        let mut state1_program = program().with_storage_capacity(1);
        state1_program.set_storage_at(0, STATE_1);
        let state1_script = state1_program.as_ref().get_script_pubkey(context.get_network());

        let mut state2_program = program().with_storage_capacity(1);
        state2_program.set_storage_at(0, STATE_2);
        let state2_script = state2_program.as_ref().get_script_pubkey(context.get_network());

        let asset = context.get_network().policy_asset();

        // Step 1 (genesis): spend a bare instance of the covenant, storing STATE_1 into
        // output 0.
        let genesis_script = fund(&context, &program())?;
        let ft1 = construct_final_tx(
            &context,
            &program(),
            &genesis_script,
            build_store_witness(STATE_1, 0),
            Sequence::default(),
            vec![PartialOutput::new(state1_script.clone(), 50, asset)],
        )?;
        context.get_default_signer().broadcast(&ft1)?.wait()?;

        // Step 2 (transition): spend the STATE_1-committing output, loading STATE_1 and
        // storing STATE_2 into output 0.
        let ft2 = construct_final_tx(
            &context,
            &state1_program,
            &state1_script,
            build_transition_witness(STATE_1, STATE_2, 0),
            Sequence::default(),
            vec![PartialOutput::new(state2_script.clone(), 50, asset)],
        )?;
        context.get_default_signer().broadcast(&ft2)?.wait()?;

        // Step 3 (final consumption): spend the STATE_2-committing output, loading
        // STATE_2, with no further re-commitment.
        let ft3 = construct_final_tx(
            &context,
            &state2_program,
            &state2_script,
            build_load_witness(STATE_2),
            Sequence::default(),
            vec![],
        )?;
        context.get_default_signer().broadcast(&ft3)?;

        Ok(())
    }

    #[simplex::test]
    fn state_chain_second_hop_wrong_state_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // Same shape as `state_chain_store_load_store_load`, but the second hop claims
        // the wrong prior state -- demonstrating that a real, otherwise-legitimate chain
        // genuinely halts if someone tries to continue it with a state that doesn't
        // match what was actually committed, rather than the covenant just trusting
        // whatever the witness happens to assert.
        const STATE_1: [u8; 32] = [0x11; 32];
        const WRONG_STATE_1: [u8; 32] = [0x99; 32];
        const STATE_2: [u8; 32] = [0x22; 32];

        let mut state1_program = program().with_storage_capacity(1);
        state1_program.set_storage_at(0, STATE_1);
        let state1_script = state1_program.as_ref().get_script_pubkey(context.get_network());

        let mut state2_program = program().with_storage_capacity(1);
        state2_program.set_storage_at(0, STATE_2);
        let state2_script = state2_program.as_ref().get_script_pubkey(context.get_network());

        let asset = context.get_network().policy_asset();

        // Step 1 (genesis): spend a bare instance of the covenant, storing STATE_1 into
        // output 0. This step succeeds -- the chain is genuinely underway.
        let genesis_script = fund(&context, &program())?;
        let ft1 = construct_final_tx(
            &context,
            &program(),
            &genesis_script,
            build_store_witness(STATE_1, 0),
            Sequence::default(),
            vec![PartialOutput::new(state1_script.clone(), 50, asset)],
        )?;
        context.get_default_signer().broadcast(&ft1)?.wait()?;

        // Step 2 (transition, with a WRONG claimed prior state): spend the
        // STATE_1-committing output, but claim WRONG_STATE_1 instead of STATE_1.
        // `load`'s assert fails before `store` -- or broadcast -- ever comes into play.
        let ft2 = construct_final_tx(
            &context,
            &state1_program,
            &state1_script,
            build_transition_witness(WRONG_STATE_1, STATE_2, 0),
            Sequence::default(),
            vec![PartialOutput::new(state2_script.clone(), 50, asset)],
        )?;
        let result2 = context
            .get_default_signer()
            .broadcast(&ft2)
            .map(|receipt| receipt.to_string())
            .map_err(anyhow::Error::from);

        assert_error_msg(result2, Expect::AssertFailed)
    }
}
