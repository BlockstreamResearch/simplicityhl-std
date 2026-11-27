use simplicityhl_std::artifacts::tests::asserts::AssertsProgram as AssertsTestProgram;
use simplicityhl_std::artifacts::tests::asserts::derived_asserts::{
    AssertsArguments as AssertsTestArguments, AssertsWitness as AssertsTestWitness,
};

use crate::common::case::DispatchCase;
use crate::common::numerics::U256;

#[derive(Clone, Copy)]
enum FunctionToTest {
    AssertEq1 = 0,
    AssertEq8 = 1,
    AssertEq16 = 2,
    AssertEq32 = 3,
    AssertEq64 = 4,
    AssertEq128 = 5,
    AssertEq256 = 6,
    AssertEqBool = 7,
    AssertNone1 = 8,
    AssertNone8 = 9,
    AssertNone16 = 10,
    AssertNone32 = 11,
    AssertNone64 = 12,
    AssertNone128 = 13,
    AssertNone256 = 14,
}

pub fn assert_eq_1(a: u8, b: u8) -> Case {
    Case::new(FunctionToTest::AssertEq1).u1(Some(a), Some(b))
}

pub fn assert_eq_8(a: u8, b: u8) -> Case {
    Case::new(FunctionToTest::AssertEq8).u8(Some(a), Some(b))
}

pub fn assert_eq_16(a: u16, b: u16) -> Case {
    Case::new(FunctionToTest::AssertEq16).u16(Some(a), Some(b))
}

pub fn assert_eq_32(a: u32, b: u32) -> Case {
    Case::new(FunctionToTest::AssertEq32).u32(Some(a), Some(b))
}

pub fn assert_eq_64(a: u64, b: u64) -> Case {
    Case::new(FunctionToTest::AssertEq64).u64(Some(a), Some(b))
}

pub fn assert_eq_128(a: u128, b: u128) -> Case {
    Case::new(FunctionToTest::AssertEq128).u128(Some(a), Some(b))
}

pub fn assert_eq_256(a: U256, b: U256) -> Case {
    Case::new(FunctionToTest::AssertEq256).u256(Some(a), Some(b))
}

pub fn assert_eq_bool(a: bool, b: bool) -> Case {
    Case::new(FunctionToTest::AssertEqBool).u1(Some(u8::from(a)), Some(u8::from(b)))
}

pub fn assert_none_1(value: Option<u8>) -> Case {
    Case::new(FunctionToTest::AssertNone1).u1(value, Some(0))
}

pub fn assert_none_8(value: Option<u8>) -> Case {
    Case::new(FunctionToTest::AssertNone8).u8(value, Some(0))
}

pub fn assert_none_16(value: Option<u16>) -> Case {
    Case::new(FunctionToTest::AssertNone16).u16(value, Some(0))
}

pub fn assert_none_32(value: Option<u32>) -> Case {
    Case::new(FunctionToTest::AssertNone32).u32(value, Some(0))
}

pub fn assert_none_64(value: Option<u64>) -> Case {
    Case::new(FunctionToTest::AssertNone64).u64(value, Some(0))
}

pub fn assert_none_128(value: Option<u128>) -> Case {
    Case::new(FunctionToTest::AssertNone128).u128(value, Some(0))
}

pub fn assert_none_256(value: Option<U256>) -> Case {
    Case::new(FunctionToTest::AssertNone256).u256(value, Some([0; 32]))
}

#[derive(Clone, Debug)]
pub struct Case {
    witness: AssertsTestWitness,
}

impl Case {
    fn new(function: FunctionToTest) -> Self {
        Self {
            witness: AssertsTestWitness {
                function_index: function as u8,
                first_arg_u1: Some(0),
                second_arg_u1: Some(0),
                first_arg_u8: Some(0),
                second_arg_u8: Some(0),
                first_arg_u16: Some(0),
                second_arg_u16: Some(0),
                first_arg_u32: Some(0),
                second_arg_u32: Some(0),
                first_arg_u64: Some(0),
                second_arg_u64: Some(0),
                first_arg_u128: Some(0),
                second_arg_u128: Some(0),
                first_arg_u256: Some([0; 32]),
                second_arg_u256: Some([0; 32]),
            },
        }
    }

    fn u1(mut self, first: Option<u8>, second: Option<u8>) -> Self {
        self.witness.first_arg_u1 = first;
        self.witness.second_arg_u1 = second;
        self
    }

    fn u8(mut self, first: Option<u8>, second: Option<u8>) -> Self {
        self.witness.first_arg_u8 = first;
        self.witness.second_arg_u8 = second;
        self
    }

    fn u16(mut self, first: Option<u16>, second: Option<u16>) -> Self {
        self.witness.first_arg_u16 = first;
        self.witness.second_arg_u16 = second;
        self
    }

    fn u32(mut self, first: Option<u32>, second: Option<u32>) -> Self {
        self.witness.first_arg_u32 = first;
        self.witness.second_arg_u32 = second;
        self
    }

    fn u64(mut self, first: Option<u64>, second: Option<u64>) -> Self {
        self.witness.first_arg_u64 = first;
        self.witness.second_arg_u64 = second;
        self
    }

    fn u128(mut self, first: Option<u128>, second: Option<u128>) -> Self {
        self.witness.first_arg_u128 = first;
        self.witness.second_arg_u128 = second;
        self
    }

    fn u256(mut self, first: Option<U256>, second: Option<U256>) -> Self {
        self.witness.first_arg_u256 = first;
        self.witness.second_arg_u256 = second;
        self
    }
}

impl DispatchCase for Case {
    type Program = AssertsTestProgram;

    const PROGRAM_NAME: &'static str = "asserts";

    fn arguments() -> AssertsTestArguments {
        AssertsTestArguments {}
    }

    fn into_witness(self) -> AssertsTestWitness {
        self.witness
    }
}
