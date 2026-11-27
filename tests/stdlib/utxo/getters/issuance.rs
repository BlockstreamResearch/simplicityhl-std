use simplex::simplicityhl::elements::AssetEntropy;
use simplex::simplicityhl::elements::Txid;

use crate::common::core::Expect;
use crate::common::core::IssuanceParams;
use crate::common::core::IssuanceSetup;
use crate::common::core::ReissuanceParams;
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
    KindOfIssuance,
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
    funding_txid: Option<Txid>,
    issuance_amount: u64,
    token_amount: u64,
    amount_to_reissue: u64,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: TestIssuanceWitness {
            function_index: function as u8,
            input_index: 0,
            expected_kind: None,
            expected_bool: false,
            expected_u256: None,
        },
        funding_txid: None,
        issuance_amount: 0,
        token_amount: 0,
        amount_to_reissue: 0,
    }
}

impl Case {
    /// `index`: the input the arm inspects.
    fn index(mut self, index: u32) -> Self {
        self.witness.input_index = index;
        self
    }

    /// `funding_txid`: the txid of utxo to issue from.
    fn funding_txid(mut self, funding_txid: Txid) -> Self {
        self.funding_txid = Some(funding_txid);
        self
    }

    /// `issuance_amount`: the issuance amount to build the transaction.
    fn issuance_amount(mut self, issuance_amount: u64) -> Self {
        self.issuance_amount = issuance_amount;
        self
    }

    /// `token_amount`: amount of reissuance tokens minted by the initial issuance.
    fn token_amount(mut self, token_amount: u64) -> Self {
        self.token_amount = token_amount;
        self
    }

    /// `amount_to_reissue`: reissued by spending the reissuance token.
    fn amount_to_reissue(mut self, amount_to_reissue: u64) -> Self {
        self.amount_to_reissue = amount_to_reissue;
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn expected_bool(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
        self
    }

    /// `expected_kind`: the boolean option the arm should report.
    fn expected_kind(mut self, expected_kind: Option<bool>) -> Self {
        self.witness.expected_kind = expected_kind;
        self
    }

    /// `expect: the optional 256-bit value (asset ID, token ID or entropy) the arm should return.
    fn expect(mut self, expected_u256: Option<[u8; 32]>) -> Self {
        self.witness.expected_u256 = expected_u256;
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
        let issuance_amount = self.issuance_amount;
        let token_amount = self.token_amount;
        let amount_to_reissue = self.amount_to_reissue;

        anyhow::ensure!(
            amount_to_reissue == 0 || token_amount > 0,
            "Reissuance needs a token to issue from"
        );

        let issuance_params = IssuanceParams {
            funding_txid: self.funding_txid,
            issuance_amount,
            token_amount,
            contract_hash,
        };

        let setup_param = if amount_to_reissue > 0 {
            IssuanceSetup::Reissue(ReissuanceParams {
                reissue_amount: amount_to_reissue,
                initial_issuance: issuance_params,
            })
        } else if issuance_amount > 0 || token_amount > 0 {
            IssuanceSetup::New(issuance_params)
        } else {
            IssuanceSetup::None
        };

        run_with_issuance(context, program(), self.witness, expect, setup_param)
    }
}

fn calculate_entropy(
    context: &simplex::TestContext,
    contract_hash: [u8; 32],
) -> anyhow::Result<(AssetEntropy, Txid)> {
    let funding_txid = create_funding_utxo_for_issue(context)?;

    let utxos = context.get_default_signer().get_utxos_txid(funding_txid)?;
    let funding_utxo = search_utxo_by_txid(&funding_txid, &utxos)?;

    let entropy = get_entropy(funding_utxo.outpoint, contract_hash);

    Ok((entropy, funding_txid))
}

#[simplex::test]
fn kind_of_issuance_new_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(KindOfIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(1)
        .expected_kind(Some(false))
        .run(&context, contract_hash)
}

#[simplex::test]
fn kind_of_issuance_reissuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(KindOfIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(1)
        .token_amount(1)
        .amount_to_reissue(10)
        .expected_kind(Some(true))
        .run(&context, contract_hash)
}

#[simplex::test]
fn kind_of_issuance_issuance_with_token(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(KindOfIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(10)
        .token_amount(100)
        .amount_to_reissue(0)
        .expected_kind(Some(false))
        .run(&context, contract_hash)
}

#[simplex::test]
fn kind_of_issuance_no_issuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(KindOfIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(0)
        .expected_kind(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn kind_of_issuance_nonexistent_input_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = u32::MAX;
    let contract_hash: [u8; 32] = rand::random();

    case(KindOfIssuance)
        .index(input_index)
        .issuance_amount(100)
        .token_amount(0)
        .expected_kind(None)
        .run(&context, contract_hash)
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
fn is_issuance_with_minting_token(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(10)
        .token_amount(100)
        .amount_to_reissue(0)
        .expected_bool(true)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_no_issuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_only_reissuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(10)
        .amount_to_reissue(100)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_issuance_nonexistent_input_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = u32::MAX;
    let contract_hash: [u8; 32] = rand::random();

    case(IsIssuance)
        .index(input_index)
        .issuance_amount(100)
        .token_amount(0)
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
        .token_amount(10)
        .amount_to_reissue(100)
        .expected_bool(true)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_new_issuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(10)
        .token_amount(0)
        .amount_to_reissue(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_new_issuance_with_token_false(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(10)
        .token_amount(1)
        .amount_to_reissue(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_no_issuance_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(0)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn is_reissuance_nonexistent_input_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = u32::MAX;
    let contract_hash: [u8; 32] = rand::random();

    case(IsReissuance)
        .index(input_index)
        .issuance_amount(10)
        .token_amount(1)
        .amount_to_reissue(100)
        .expected_bool(false)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let asset_id = get_asset_id(entropy).to_byte_array();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(100)
        .token_amount(0)
        .expect(Some(asset_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_with_token(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let asset_id = get_asset_id(entropy).to_byte_array();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .token_amount(10)
        .expect(Some(asset_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_from_reissuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let asset_id = get_asset_id(entropy).to_byte_array();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .token_amount(10)
        .amount_to_reissue(100)
        .expect(Some(asset_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_no_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceAssetId)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(0)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_asset_id_nonexistent_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = u32::MAX;
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceAssetId)
        .index(input_index)
        .issuance_amount(100)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let token_id = get_token_id(entropy).to_byte_array();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(1)
        .token_amount(10)
        .amount_to_reissue(100)
        .expect(Some(token_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_from_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let token_id = get_token_id(entropy).to_byte_array();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(100)
        .amount_to_reissue(0)
        .expect(Some(token_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_from_issuance_with_token(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (entropy, txid) = calculate_entropy(&context, contract_hash)?;
    let token_id = get_token_id(entropy).to_byte_array();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(100)
        .token_amount(10)
        .amount_to_reissue(0)
        .expect(Some(token_id))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_no_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(GetReissuanceToken)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(0)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_reissuance_token_nonexistent_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = u32::MAX;
    let contract_hash: [u8; 32] = rand::random();

    case(GetReissuanceToken)
        .index(input_index)
        .issuance_amount(0)
        .token_amount(100)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_new_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (expected_entropy, txid) = calculate_entropy(&context, contract_hash)?;

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(10)
        .token_amount(0)
        .amount_to_reissue(0)
        .expect(Some(expected_entropy.to_byte_array()))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_new_issuance_with_token(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (expected_entropy, txid) = calculate_entropy(&context, contract_hash)?;

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(10)
        .token_amount(100)
        .amount_to_reissue(0)
        .expect(Some(expected_entropy.to_byte_array()))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_reissuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    let (expected_entropy, txid) = calculate_entropy(&context, contract_hash)?;

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .funding_txid(txid)
        .issuance_amount(0)
        .token_amount(100)
        .amount_to_reissue(1000)
        .expect(Some(expected_entropy.to_byte_array()))
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_no_issuance(context: simplex::TestContext) -> anyhow::Result<()> {
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceEntropy)
        .index(DEFAULT_ISSUANCE_INDEX)
        .issuance_amount(0)
        .token_amount(0)
        .expect(None)
        .run(&context, contract_hash)
}

#[simplex::test]
fn get_issuance_entropy_nonexistent_input(context: simplex::TestContext) -> anyhow::Result<()> {
    let input_index = u32::MAX;
    let contract_hash: [u8; 32] = rand::random();

    case(GetIssuanceEntropy)
        .index(input_index)
        .issuance_amount(100)
        .token_amount(0)
        .expect(None)
        .run(&context, contract_hash)
}
