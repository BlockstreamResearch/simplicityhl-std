use simplex::either::Either;
use simplex::simplicityhl::elements::{AssetId, Script};
use simplex::transaction::PartialOutput;

use simplicityhl_std::artifacts::tests::utxo::op_return::OpReturnProgram as OpReturnTestProgram;
use simplicityhl_std::artifacts::tests::utxo::op_return::derived_op_return::{
    OpReturnArguments as OpReturnTestArguments, OpReturnWitness as OpReturnTestWitness,
};

use crate::common::case::DispatchCase;
use crate::common::core::create_op_return_script;

pub type OpReturnEntry = Either<PushedData, Either<u8, u8>>;

pub type PushedData = (u8, [u8; 32]);

#[derive(Clone, Copy)]
enum FunctionToTest {
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

pub fn op_return_output(data: &[u8]) -> PartialOutput {
    PartialOutput::new(create_op_return_script(data), 0, AssetId::default())
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
    data: Option<&'static [u8]>,
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
            data: None,
        }
    }

    pub fn op_return(mut self, data: &'static [u8]) -> Self {
        self.data = Some(data);
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

    const PROGRAM_NAME: &'static str = "utxo::op_return";

    fn arguments() -> OpReturnTestArguments {
        OpReturnTestArguments {}
    }

    fn into_witness(self) -> OpReturnTestWitness {
        self.witness
    }

    fn leading_outputs(&self) -> Vec<PartialOutput> {
        self.data.map(op_return_output).into_iter().collect()
    }
}
