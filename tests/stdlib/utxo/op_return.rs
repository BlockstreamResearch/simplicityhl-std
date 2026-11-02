use rand::Rng;
use secp256k1_zkp::hashes::Hash;

use simplex::either::Either;
use simplex::simplicityhl::elements::Script;

use crate::common::core::{Expect, run, run_with_op_return};

use simplicityhl_std::artifacts::tests::utxo::op_return::OpReturnProgram as OpReturnTestProgram;
use simplicityhl_std::artifacts::tests::utxo::op_return::derived_op_return::{
    OpReturnArguments as OpReturnTestArguments, OpReturnWitness as OpReturnTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    GetEntryOption,
    PushedDataFromEntry,
    GetPushedData,
    OpXMinus1FromEntry,
    GetOpXMinus1,
    NegateOrReservedFromEntry,
    GetNegateOrReserved,
    KindOfEntry,
    IsOutputOpReturn,
    HasEntry,
    AssertOutputIsOpReturn,
    AssertHasEntry,
}

type Entry = Either<(u8, [u8; 32]), Either<u8, u8>>;

const DEFAULT_DATA: &[u8; 2] = &[0, 1];
const OP_PUSHNUM_NEG1: &[u8] = &[0x01, 0x4f];
const OP_RESERVED: &[u8] = &[0x01, 0x50];

fn program() -> OpReturnTestProgram {
    OpReturnTestProgram::new(&OpReturnTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads and whether the
/// spend carries an `OP_RETURN` output.
struct Case {
    witness: OpReturnTestWitness,
    data: Option<&'static [u8]>,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: OpReturnTestWitness {
            function_index: function as u8,
            output_index: 0,
            entry_index: 0,
            entry: Either::Right(Either::Right(0)),
            expected_entry: None,
            expected_value: None,
            expected_bool: false,
            expected_hash: [0; 32],
        },
        data: None,
    }
}

impl Case {
    /// `output_index`: the output the arm inspects.
    fn output_index(mut self, output_index: u32) -> Self {
        self.witness.output_index = output_index;
        self
    }

    /// `entry_index`: the entry index the arm inspects.
    fn entry_index(mut self, entry_index: u32) -> Self {
        self.witness.entry_index = entry_index;
        self
    }

    /// `entry`:the entry passed to the arm.
    fn entry(mut self, entry: Entry) -> Self {
        self.witness.entry = entry;
        self
    }

    /// `expected_entry`: the entry the arm expects.
    fn expected_entry(mut self, expected_entry: Option<Entry>) -> Self {
        self.witness.expected_entry = expected_entry;
        self
    }

    /// `expected_value`: the value the arm expects.
    fn expected_value(mut self, expected_value: Option<u8>) -> Self {
        self.witness.expected_value = expected_value;
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn expected_bool(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
        self
    }

    /// `expected_hash`: the hash of the pushed data the arm inspects.
    fn expected_hash(mut self, expected_hash: [u8; 32]) -> Self {
        self.witness.expected_hash = expected_hash;
        self
    }

    /// Add an `OP_RETURN` output carrying `data` to the spend.
    /// First byte is a build indicator (0 means single data push, 1 means raw opcode push)
    fn op_return(mut self, data: &'static [u8]) -> Self {
        self.data = Some(data);
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        match self.data {
            Some(data) => run_with_op_return(context, program(), self.witness, expect, data),
            None => run(context, program(), self.witness, expect),
        }
    }
}

pub fn hash_of_pushed_data(data: &[u8]) -> [u8; 32] {
    Script::from(data.to_vec()).wscript_hash().to_byte_array()
}

#[simplex::test]
fn get_entry_option(context: simplex::TestContext) -> anyhow::Result<()> {
    /// 0 = push data (0x12, 0x23), 1 = OP_1, 2 = OP_1NEGATE, 3 = OP_RESERVED.
    const MIXED_ENTRIES: &[u8] = &[1, 0x02, 0x12, 0x23, 0x51, 0x4f, 0x50];
    const PUSHED_DATA: &[u8] = &[0x12, 0x23];

    let cases: [(u32, Entry); 4] = [
        (0, Either::Left((0, hash_of_pushed_data(PUSHED_DATA)))),
        (1, Either::Right(Either::Right(0))),
        (2, Either::Right(Either::Left(0))),
        (3, Either::Right(Either::Left(1))),
    ];

    for (entry_index, entry) in cases {
        case(GetEntryOption)
            .entry_index(entry_index)
            .expected_entry(Some(entry))
            .op_return(MIXED_ENTRIES)
            .run(&context)
            .unwrap_or_else(|_| panic!("entry index = {entry_index}"));
    }

    Ok(())
}

#[simplex::test]
fn get_entry_option_nonexistent_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = u32::MAX;
    let entry_index = 0;
    let expected_entry: Option<Entry> = None;

    case(GetEntryOption)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_entry(expected_entry)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn get_entry_option_not_op_return_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;
    let expected_entry: Option<Entry> = None;

    case(GetEntryOption)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_entry(expected_entry)
        .run(&context)
}

#[simplex::test]
fn get_entry_option_out_of_range_entry(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = u32::MAX;
    let expected_entry: Option<Entry> = None;

    case(GetEntryOption)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_entry(expected_entry)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn pushed_data_from_entry(context: simplex::TestContext) -> anyhow::Result<()> {
    let indicator = 0_u8;
    let hash = hash_of_pushed_data(&DEFAULT_DATA[1..]);

    let entry: Entry = Either::Left((indicator, hash));

    case(PushedDataFromEntry)
        .entry(entry)
        .expected_value(Some(indicator))
        .expected_hash(hash)
        .run(&context)
}

#[simplex::test]
fn pushed_data_from_entry_op_x_returns_none(context: simplex::TestContext) -> anyhow::Result<()> {
    case(PushedDataFromEntry)
        .entry(Either::Right(Either::Right(0)))
        .expected_value(None)
        .run(&context)
}

#[simplex::test]
fn get_pushdata0(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    let indicator = 0_u8;
    let hash = hash_of_pushed_data(&DEFAULT_DATA[1..]);

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(Some(indicator))
        .expected_hash(hash)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn get_pushdata1(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    const DATA: [u8; 77] = [0; 77]; // a 76-byte push, which needs OP_PUSHDATA1

    let indicator = 1_u8;
    let hash = hash_of_pushed_data(&DATA[1..]);

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(Some(indicator))
        .expected_hash(hash)
        .op_return(&DATA)
        .run(&context)
}

#[simplex::test]
fn get_pushdata2(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    const DATA: [u8; 7] = [1, 0x4d, 0x03, 0x00, 0xde, 0xad, 0xbe]; // the last 3 bytes are the pushed data

    let indicator = 2_u8;
    let hash = hash_of_pushed_data(&DATA[4..]);

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(Some(indicator))
        .expected_hash(hash)
        .op_return(&DATA)
        .run(&context)
}

#[simplex::test]
fn get_pushdata4(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    const DATA: [u8; 9] = [1, 0x4e, 0x03, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe]; // the last 3 bytes are the pushed data

    let indicator = 3_u8;
    let hash = hash_of_pushed_data(&DATA[6..]);

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(Some(indicator))
        .expected_hash(hash)
        .op_return(&DATA)
        .run(&context)
}

#[simplex::test]
fn get_pushed_data_nonexistent_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = u32::MAX;
    let entry_index = 0;

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn get_pushed_data_not_op_return_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .run(&context)
}

#[simplex::test]
fn get_pushed_data_out_of_range_entry(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = u32::MAX;

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn get_pushed_data_neg1_returns_none(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(GetPushedData)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(OP_PUSHNUM_NEG1)
        .run(&context)
}

#[simplex::test]
fn op_x_minus_1_from_entry(context: simplex::TestContext) -> anyhow::Result<()> {
    for i in 0..16 {
        let entry: Entry = Either::Right(Either::Right(i));

        case(OpXMinus1FromEntry)
            .entry(entry)
            .expected_value(Some(i))
            .run(&context)
            .unwrap_or_else(|_| panic!("entry index = {i}"));
    }

    Ok(())
}

#[simplex::test]
fn op_x_minus_1_from_entry_returns_none(context: simplex::TestContext) -> anyhow::Result<()> {
    let entry: Entry = Either::Right(Either::Left(1)); // OP_RESERVED

    case(OpXMinus1FromEntry)
        .entry(entry)
        .expected_value(None)
        .run(&context)
}

#[simplex::test]
fn get_op_x_minus_1(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;

    // the first byte is the build indicator
    const OP_1_TO_16: &[u8] = &[
        0x01, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e,
        0x5f, 0x60,
    ];

    for i in 0..16 {
        case(GetOpXMinus1)
            .output_index(output_index)
            .entry_index(i)
            .expected_value(Some(i as u8))
            .op_return(OP_1_TO_16)
            .run(&context)
            .unwrap_or_else(|err| panic!("{err}, entry index = {i}"));
    }

    Ok(())
}

#[simplex::test]
fn get_op_x_minus_1_nonexistent_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = u32::MAX;
    let entry_index = 0;

    case(GetOpXMinus1)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn get_op_x_minus_1_not_op_return_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(GetOpXMinus1)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .run(&context)
}

#[simplex::test]
fn get_op_x_minus_1_out_of_range_entry(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = u32::MAX;
    const OP_1: &[u8] = &[0x01, 0x51];

    case(GetOpXMinus1)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(OP_1)
        .run(&context)
}

#[simplex::test]
fn get_op_x_minus_1_returns_none(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(GetOpXMinus1)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(OP_RESERVED)
        .run(&context)
}

#[simplex::test]
fn negate_or_reserved_from_entry_negate(context: simplex::TestContext) -> anyhow::Result<()> {
    let indicator = 0_u8;

    let entry: Entry = Either::Right(Either::Left(indicator));

    case(NegateOrReservedFromEntry)
        .entry(entry)
        .expected_value(Some(indicator))
        .run(&context)
}

#[simplex::test]
fn negate_or_reserved_from_entry_reserved(context: simplex::TestContext) -> anyhow::Result<()> {
    let indicator = 1_u8;

    let entry: Entry = Either::Right(Either::Left(indicator));

    case(NegateOrReservedFromEntry)
        .entry(entry)
        .expected_value(Some(indicator))
        .run(&context)
}

#[simplex::test]
fn negate_or_reserved_from_entry_returns_none(context: simplex::TestContext) -> anyhow::Result<()> {
    let entry: Entry = Either::Left((0, hash_of_pushed_data(&[1])));

    case(NegateOrReservedFromEntry)
        .entry(entry)
        .expected_value(None)
        .run(&context)
}

#[simplex::test]
fn get_negate_or_reserved_negate(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;
    let indicator = 0_u8;

    case(GetNegateOrReserved)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(Some(indicator))
        .op_return(OP_PUSHNUM_NEG1)
        .run(&context)
}

#[simplex::test]
fn get_negate_or_reserved_reserved(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;
    let indicator = 1_u8;

    case(GetNegateOrReserved)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(Some(indicator))
        .op_return(OP_RESERVED)
        .run(&context)
}

#[simplex::test]
fn get_negate_or_reserved_nonexistent_output(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = u32::MAX;
    let entry_index = 0;

    case(GetNegateOrReserved)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn get_negate_or_reserved_not_op_return_output(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(GetNegateOrReserved)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .run(&context)
}

#[simplex::test]
fn get_negate_or_reserved_out_of_range_entry(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = u32::MAX;

    case(GetNegateOrReserved)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(OP_RESERVED)
        .run(&context)
}

#[simplex::test]
fn get_negate_or_reserved_returns_none(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(GetNegateOrReserved)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_value(None)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn kind_of_entry_pushed_data(context: simplex::TestContext) -> anyhow::Result<()> {
    let entry: Entry = Either::Left((0, hash_of_pushed_data(&[1])));

    case(KindOfEntry)
        .entry(entry)
        .expected_value(Some(0))
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn kind_of_entry_op_x_minus_1(context: simplex::TestContext) -> anyhow::Result<()> {
    let op_x_minus_1 = rand::thread_rng().gen_range(0..16);
    let entry: Entry = Either::Right(Either::Right(op_x_minus_1));

    case(KindOfEntry)
        .entry(entry)
        .expected_value(Some(1))
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn kind_of_entry_negate_or_reserved_negate(context: simplex::TestContext) -> anyhow::Result<()> {
    let indicator = 0_u8;
    let entry: Entry = Either::Right(Either::Left(indicator));

    case(KindOfEntry)
        .entry(entry)
        .expected_value(Some(2))
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn kind_of_entry_negate_or_reserved_reserved(context: simplex::TestContext) -> anyhow::Result<()> {
    let indicator = 1_u8;
    let entry: Entry = Either::Right(Either::Left(indicator));

    case(KindOfEntry)
        .entry(entry)
        .expected_value(Some(2))
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_return_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(IsOutputOpReturn)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_bool(true)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_returns_nonexistent_output_false(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let output_index = u32::MAX;

    case(IsOutputOpReturn)
        .output_index(output_index)
        .expected_bool(false)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn is_output_op_return_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;

    case(IsOutputOpReturn)
        .output_index(output_index)
        .expected_bool(false)
        .run(&context)
}

#[simplex::test]
fn has_entry_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = 0;

    case(HasEntry)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_bool(true)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn has_entry_nonexistent_output_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = u32::MAX;

    case(HasEntry)
        .output_index(output_index)
        .expected_bool(false)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn has_entry_out_of_range_entry_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = u32::MAX;

    case(HasEntry)
        .output_index(output_index)
        .entry_index(entry_index)
        .expected_bool(false)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn has_entry_output_not_op_return_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;

    case(HasEntry)
        .output_index(output_index)
        .expected_bool(false)
        .run(&context)
}

#[simplex::test]
fn assert_output_is_op_return_pass(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;

    case(AssertOutputIsOpReturn)
        .output_index(output_index)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn assert_output_is_op_returns_nonexistent_output_fails(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let output_index = u32::MAX;

    case(AssertOutputIsOpReturn)
        .output_index(output_index)
        .op_return(DEFAULT_DATA)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_output_is_op_return_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;

    case(AssertOutputIsOpReturn)
        .output_index(output_index)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_has_entry_pass(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;

    case(AssertHasEntry)
        .output_index(output_index)
        .op_return(DEFAULT_DATA)
        .run(&context)
}

#[simplex::test]
fn assert_has_entry_nonexistent_output_fails(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = u32::MAX;

    case(AssertHasEntry)
        .output_index(output_index)
        .op_return(DEFAULT_DATA)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_has_entry_output_not_op_return_fails(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let output_index = 0;

    case(AssertHasEntry)
        .output_index(output_index)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_has_entry_out_of_range_entry_fails(context: simplex::TestContext) -> anyhow::Result<()> {
    let output_index = 0;
    let entry_index = u32::MAX;

    case(AssertHasEntry)
        .output_index(output_index)
        .entry_index(entry_index)
        .op_return(DEFAULT_DATA)
        .expecting(&context, Expect::AssertFailed)
}
