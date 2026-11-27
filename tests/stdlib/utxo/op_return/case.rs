use simplex::either::Either;
use simplex::simplicityhl::elements::opcodes::{self, All};
use simplex::simplicityhl::elements::script::Builder;
use simplex::simplicityhl::elements::{AssetId, Script};
use simplex::transaction::{FinalTransaction, PartialInput, PartialOutput, UTXO};

use simplicityhl_std::artifacts::tests::utxo::op_return::OpReturnProgram as OpReturnTestProgram;
use simplicityhl_std::artifacts::tests::utxo::op_return::derived_op_return::{
    OpReturnArguments as OpReturnTestArguments, OpReturnWitness as OpReturnTestWitness,
};

use crate::common::case::DispatchCase;

pub type OpReturnEntry = Either<PushedData, Either<u8, u8>>;

pub type PushedData = (u8, [u8; 32]);

pub const PROGRAM_NAME: &str = "utxo::op_return";

const OP_RETURN: u8 = 0x6a;
const OP_PUSHDATA1: u8 = 0x4c;
const OP_PUSHDATA2: u8 = 0x4d;
const OP_PUSHDATA4: u8 = 0x4e;
const OP_1NEGATE: u8 = 0x4f;
const OP_RESERVED: u8 = 0x50;
const OP_1: u8 = 0x51;
const OP_16: u8 = 0x60;

/// Outputs of [`reference_tx`].
pub mod reference {
    /// `OP_RETURN <0x01>`, a direct push.
    pub const PUSHDATA0: u32 = 0;
    /// A plain output, not `OP_RETURN`.
    pub const NOT_OP_RETURN: u32 = 1;
    /// `OP_RETURN <0x12 0x23> OP_1 OP_1NEGATE OP_RESERVED`.
    pub const MIXED: u32 = 2;
    /// A bare `OP_RETURN` with no entries.
    pub const EMPTY: u32 = 3;
    /// `OP_RETURN OP_PUSHDATA1 <76 zero bytes>`.
    pub const PUSHDATA1: u32 = 4;
    /// `OP_RETURN OP_1 .. OP_16`.
    pub const OP_1_TO_16: u32 = 5;
    /// `OP_RETURN OP_CHECKSIG`, starts with `OP_RETURN` but is not null data.
    pub const MALFORMED: u32 = 6;
    /// An empty script, like a fee output.
    pub const FEE: u32 = 7;
    /// The number of outputs.
    pub const OUTPUTS: u32 = 8;
}

#[derive(Clone, Copy)]
pub enum FunctionToTest {
    GetEntryOption = 0,
    PushedDataFromEntry = 1,
    GetPushedData = 2,
    OpXMinus1FromEntry = 3,
    GetOpXMinus1 = 4,
    NegateOrReservedFromEntry = 5,
    GetNegateOrReserved = 6,
    KindOfEntry = 7,
    IsOutputOpReturn = 8,
    HasEntry = 9,
    AssertOutputIsOpReturn = 10,
    AssertHasEntry = 11,
}

pub fn pushed_data(indicator: u8, hash: [u8; 32]) -> OpReturnEntry {
    Either::Left((indicator, hash))
}

pub fn negate_or_reserved(indicator: u8) -> OpReturnEntry {
    Either::Right(Either::Left(indicator))
}

pub fn op_x_minus_1(value: u8) -> OpReturnEntry {
    Either::Right(Either::Right(value))
}

pub fn hash_of_pushed_data(data: &[u8]) -> [u8; 32] {
    Script::from(data.to_vec()).wscript_hash().to_byte_array()
}

/// Create an `OP_RETURN` script from bytes, where the first byte indicates whether
/// the rest is an immediate push (`0x00`) or raw opcodes.
pub fn create_op_return_script(data: &[u8]) -> Script {
    let mut builder = Builder::new().push_opcode(opcodes::all::OP_RETURN);

    if data[0] == 0x00 {
        builder = builder.push_slice(&data[1..]);
    } else {
        for i in &data[1..] {
            builder = builder.push_opcode(All::from(*i));
        }
    }

    builder.into_script()
}

pub fn op_return_output(data: &[u8]) -> PartialOutput {
    script_output(create_op_return_script(data))
}

/// An output that is not `OP_RETURN`.
pub fn plain_output() -> PartialOutput {
    script_output(Script::from(vec![OP_1]))
}

/// A bare `OP_RETURN`: null data without entries.
pub fn empty_op_return_output() -> PartialOutput {
    script_output(Script::from(vec![OP_RETURN]))
}

fn script_output(script: Script) -> PartialOutput {
    PartialOutput::new(script, 0, AssetId::default())
}

/// A transaction with a placeholder input 0 for the program and `outputs`.
pub fn tx_with_outputs(outputs: impl IntoIterator<Item = PartialOutput>) -> FinalTransaction {
    let mut tx = FinalTransaction::new();
    tx.add_input(PartialInput::new(UTXO::default()), None);

    for output in outputs {
        tx.add_output(output);
    }

    tx
}

/// One transaction with every kind of output the `OP_RETURN` functions distinguish,
/// laid out as [`reference`] describes. Unit and fuzz cases share it.
pub fn reference_tx() -> FinalTransaction {
    let mut op_1_to_16 = vec![1];
    op_1_to_16.extend(OP_1..=OP_16);

    tx_with_outputs([
        op_return_output(&[0, 1]),
        plain_output(),
        op_return_output(&[1, 0x02, 0x12, 0x23, OP_1, OP_1NEGATE, OP_RESERVED]),
        empty_op_return_output(),
        op_return_output(&[0; 77]),
        op_return_output(&op_1_to_16),
        script_output(Script::from(vec![OP_RETURN, 0xac])),
        script_output(Script::new()),
    ])
}

/// What `jet::output_null_datum(output_index, entry_index)` returns in `tx`: `None` when
/// the output does not exist or is not null data, `Some(None)` when the entry does not exist.
pub fn output_null_datum(
    tx: &FinalTransaction,
    output_index: u32,
    entry_index: u32,
) -> Option<Option<OpReturnEntry>> {
    let output = tx.outputs().get(usize::try_from(output_index).ok()?)?;
    let entries = null_data_entries(output.script_pubkey.as_bytes())?;

    Some(
        usize::try_from(entry_index)
            .ok()
            .and_then(|index| entries.get(index).copied()),
    )
}

/// The entries of a null-data script, `None` when `script` is not null data.
/// Mirrors `parseNullData` of the Simplicity C implementation.
pub fn null_data_entries(script: &[u8]) -> Option<Vec<OpReturnEntry>> {
    let (&first, mut rest) = script.split_first()?;

    if first != OP_RETURN {
        return None;
    }

    let mut entries = Vec::new();

    while let Some((&code, tail)) = rest.split_first() {
        rest = tail;

        let entry = match code {
            OP_1..=OP_16 => op_x_minus_1(code - OP_1),
            OP_1NEGATE | OP_RESERVED => negate_or_reserved(code - OP_1NEGATE),
            ..OP_1NEGATE => {
                let (indicator, length_bytes) = match code {
                    OP_PUSHDATA1 => (1, 1),
                    OP_PUSHDATA2 => (2, 2),
                    OP_PUSHDATA4 => (3, 4),
                    _ => (0, 0),
                };

                let length = if indicator == 0 {
                    usize::from(code)
                } else {
                    let (length, tail) = rest.split_at_checked(length_bytes)?;
                    rest = tail;

                    length
                        .iter()
                        .rev()
                        .fold(0, |length, byte| length << 8 | usize::from(*byte))
                };

                let (data, tail) = rest.split_at_checked(length)?;
                rest = tail;

                pushed_data(indicator, hash_of_pushed_data(data))
            }
            _ => return None,
        };

        entries.push(entry);
    }

    Some(entries)
}

pub fn get_entry_option(
    output_index: u32,
    entry_index: u32,
    expected: Option<OpReturnEntry>,
) -> Case {
    Case::new(FunctionToTest::GetEntryOption)
        .at(output_index, entry_index)
        .expect_entry(expected)
}

pub fn pushed_data_from_entry(entry: OpReturnEntry, expected: Option<PushedData>) -> Case {
    Case::new(FunctionToTest::PushedDataFromEntry)
        .entry(entry)
        .expect_pushed_data(expected)
}

pub fn get_pushed_data(output_index: u32, entry_index: u32, expected: Option<PushedData>) -> Case {
    Case::new(FunctionToTest::GetPushedData)
        .at(output_index, entry_index)
        .expect_pushed_data(expected)
}

pub fn op_x_minus_1_from_entry(entry: OpReturnEntry, expected: Option<u8>) -> Case {
    Case::new(FunctionToTest::OpXMinus1FromEntry)
        .entry(entry)
        .expect_value(expected)
}

pub fn get_op_x_minus_1(output_index: u32, entry_index: u32, expected: Option<u8>) -> Case {
    Case::new(FunctionToTest::GetOpXMinus1)
        .at(output_index, entry_index)
        .expect_value(expected)
}

pub fn negate_or_reserved_from_entry(entry: OpReturnEntry, expected: Option<u8>) -> Case {
    Case::new(FunctionToTest::NegateOrReservedFromEntry)
        .entry(entry)
        .expect_value(expected)
}

pub fn get_negate_or_reserved(output_index: u32, entry_index: u32, expected: Option<u8>) -> Case {
    Case::new(FunctionToTest::GetNegateOrReserved)
        .at(output_index, entry_index)
        .expect_value(expected)
}

pub fn kind_of_entry(entry: OpReturnEntry, expected: u8) -> Case {
    Case::new(FunctionToTest::KindOfEntry)
        .entry(entry)
        .expect_value(Some(expected))
}

pub fn is_output_op_return(output_index: u32, expected: bool) -> Case {
    Case::new(FunctionToTest::IsOutputOpReturn)
        .at(output_index, 0)
        .expect_bool(expected)
}

pub fn has_entry(output_index: u32, entry_index: u32, expected: bool) -> Case {
    Case::new(FunctionToTest::HasEntry)
        .at(output_index, entry_index)
        .expect_bool(expected)
}

pub fn assert_output_is_op_return(output_index: u32) -> Case {
    Case::new(FunctionToTest::AssertOutputIsOpReturn).at(output_index, 0)
}

pub fn assert_has_entry(output_index: u32, entry_index: u32) -> Case {
    Case::new(FunctionToTest::AssertHasEntry).at(output_index, entry_index)
}

#[derive(Clone, Debug)]
pub struct Case {
    witness: OpReturnTestWitness,
    tx: Option<FinalTransaction>,
}

impl Case {
    fn new(function: FunctionToTest) -> Self {
        Self {
            witness: OpReturnTestWitness {
                function_index: function as u8,
                output_index: 0,
                entry_index: 0,
                entry: op_x_minus_1(0),
                expected_entry: None,
                expected_value: None,
                expected_bool: false,
                expected_hash: [0; 32],
            },
            tx: None,
        }
    }

    /// Runs the case in a transaction with an `OP_RETURN` output built from `data`,
    /// followed by a plain output.
    pub fn op_return(self, data: &[u8]) -> Self {
        self.outputs([op_return_output(data), plain_output()])
    }

    /// Runs the case in a transaction with `outputs`.
    pub fn outputs(self, outputs: impl IntoIterator<Item = PartialOutput>) -> Self {
        self.in_tx(tx_with_outputs(outputs))
    }

    /// Runs the case in [`reference_tx`].
    pub fn in_reference_tx(self) -> Self {
        self.in_tx(reference_tx())
    }

    fn in_tx(mut self, tx: FinalTransaction) -> Self {
        self.tx = Some(tx);
        self
    }

    fn at(mut self, output_index: u32, entry_index: u32) -> Self {
        self.witness.output_index = output_index;
        self.witness.entry_index = entry_index;
        self
    }

    fn entry(mut self, entry: OpReturnEntry) -> Self {
        self.witness.entry = entry;
        self
    }

    fn expect_entry(mut self, expected_entry: Option<OpReturnEntry>) -> Self {
        self.witness.expected_entry = expected_entry;
        self
    }

    fn expect_value(mut self, expected_value: Option<u8>) -> Self {
        self.witness.expected_value = expected_value;
        self
    }

    fn expect_pushed_data(mut self, expected: Option<PushedData>) -> Self {
        let (indicator, hash) = expected.unzip();

        self.witness.expected_value = indicator;
        self.witness.expected_hash = hash.unwrap_or_default();
        self
    }

    fn expect_bool(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
        self
    }
}

impl DispatchCase for Case {
    type Program = OpReturnTestProgram;

    const PROGRAM_NAME: &'static str = PROGRAM_NAME;

    fn arguments() -> OpReturnTestArguments {
        OpReturnTestArguments {}
    }

    fn into_witness(self) -> OpReturnTestWitness {
        self.witness
    }

    fn transaction(&self) -> Option<FinalTransaction> {
        self.tx.clone()
    }
}
