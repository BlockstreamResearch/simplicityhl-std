use simplex::simplicityhl::elements::AssetEntropy;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::run_with_issuance;
use crate::common::utxo_helper::create_funding_utxo_for_issue;
use crate::common::utxo_helper::get_asset_id;
use crate::common::utxo_helper::get_entropy;
use crate::common::utxo_helper::get_token_id;
use crate::common::utxo_helper::search_utxo_by_txid;

use simplicityhl_std::artifacts::tests::utxo::getters::issuance::IssuanceProgram as TestIssuanceProgram;
use simplicityhl_std::artifacts::tests::utxo::getters::issuance::derived_issuance::{
    IssuanceArguments as TestIssuanceArguments, IssuanceWitness as TestIssuanceWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    IsIssuance,
    IsReissuance,
    GetIssuanceAssetId,
    GetReissuanceToken,
    GetIssuanceEntropy,
}

const DEFAULT_ISSUANCE_INDEX: u32 = 1;

fn program() -> TestIssuanceProgram {
    TestIssuanceProgram::new(&TestIssuanceArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: TestIssuanceWitness,
    funding_txid_for_issue: Option<Txid>,
    issuance_amount: u64,
    reissuance_amount: u64,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestIssuanceWitness {
            function_index: function as u8,
            input_index: 0,
            expected_bool: false,
            expected_u256: None,
        },
        funding_txid_for_issue: None,
        issuance_amount: 0,
        reissuance_amount: 0,
    }
}

impl Case {
    /// `index`: the input the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.input_index = index;
        self
    }

    /// `funding_txid_for_issue`: the txid of utxo to issue from.
    fn funding_txid_for_issue(mut self, funding_txid_for_issue: Txid) -> Self {
        self.funding_txid_for_issue = Some(funding_txid_for_issue);
        self
    }

    /// `issuance_amount`: the issuance amount to build the transaction.
    fn issuance_amount(mut self, issuance_amount: u64) -> Self {
        self.issuance_amount = issuance_amount;
        self
    }

    /// `reissuance_amount`: the reissuance amount to build the transaction.
    fn reissuance_amount(mut self, reissuance_amount: u64) -> Self {
        self.reissuance_amount = reissuance_amount;
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn expected_bool(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
        self
    }

    /// `expected`: value the arm should produce.
    fn expect(mut self, expected: Option<[u8; 32]>) -> Self {
        self.witness.expected_u256 = expected;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext, contract_hash: [u8; 32]) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok, contract_hash)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(
        self,
        context: &simplex::TestContext,
        expect: Expect,
        contract_hash: [u8; 32],
    ) -> anyhow::Result<()> {
        let funding_txid_for_issue = self.funding_txid_for_issue;
        let issuance_amount = self.issuance_amount;
        let reissuance_amount = self.reissuance_amount;

        run_with_issuance(
            context,
            program(),
            self.witness,
            expect,
            Some((
                funding_txid_for_issue,
                issuance_amount,
                reissuance_amount,
                contract_hash,
            )),
        )
    }
}

fn calculate_entropy(
    context: &simplex::TestContext,
    contract_hash: [u8; 32],
) -> anyhow::Result<(AssetEntropy, Txid)> {
    let funding_txid_for_issue = create_funding_utxo_for_issue(context)?;

    let utxos = context
        .get_default_signer()
        .get_utxos_txid(funding_txid_for_issue)?;
    let funding_utxo = search_utxo_by_txid(&funding_txid_for_issue, &utxos)?;

    let entropy = get_entropy(funding_utxo.outpoint, contract_hash);

    Ok((entropy, funding_txid_for_issue))
}

#[simplex::test]
fn is_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(1)
        .expected_bool(true)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_no_issuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .reissuance_amount(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_only_reissuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .reissuance_amount(10)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_nonexistent_input_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = 2;
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(input_index)
        .issuance_amount(100)
        .reissuance_amount(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_program_input_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(0)
        .issuance_amount(100)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .reissuance_amount(10)
        .expected_bool(true)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_false_for_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(10)
        .reissuance_amount(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_no_issuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .reissuance_amount(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_nonexistent_input_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = 2;
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(input_index)
        .issuance_amount(0)
        .reissuance_amount(100)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 100;
    let reissue_amount = 0;
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let asset_id = get_asset_id(entropy).to_byte_array();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid_for_issue(txid)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(Some(asset_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_from_reissuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 0;
    let reissue_amount = 100;
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let asset_id = get_asset_id(entropy).to_byte_array();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid_for_issue(txid)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(Some(asset_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_no_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 0;
    let reissue_amount = 0;
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_nonexistent_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = 2;
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceAssetId)
        .index(input_index)
        .issuance_amount(100)
        .reissuance_amount(0)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 0;
    let reissue_amount = 100;
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let token_id = get_token_id(entropy).to_byte_array();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid_for_issue(txid)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(Some(token_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_from_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 100;
    let reissue_amount = 0;
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let token_id = get_token_id(entropy).to_byte_array();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid_for_issue(txid)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(Some(token_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_no_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 0;
    let reissue_amount = 0;
    let contract_hash: [u8; 32] = rand::random();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_nonexistent_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = 2;
    let contract_hash: [u8; 32] = rand::random();

    case(GetReissuanceToken)
        .index(input_index)
        .issuance_amount(0)
        .reissuance_amount(100)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();
    let issue_amount = 10;
    let reissue_amount = 0;

    let (expected_entropy, txid) = calculate_entropy(&context, contract_hash)?;

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid_for_issue(txid)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(Some(expected_entropy.to_byte_array()))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_reissuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();
    let issue_amount = 0;
    let reissue_amount = 100;

    let (expected_entropy, txid) = calculate_entropy(&context, contract_hash)?;

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid_for_issue(txid)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(Some(expected_entropy.to_byte_array()))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_no_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let issue_amount = 0;
    let reissue_amount = 0;
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(issue_amount)
        .reissuance_amount(reissue_amount)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_nonexistent_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = 2;
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceEntropy)
        .index(input_index)
        .issuance_amount(100)
        .reissuance_amount(0)
        .expect(None)
        .run(&context, contract_hash)
}
