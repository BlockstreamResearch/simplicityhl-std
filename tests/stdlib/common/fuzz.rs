use std::fmt::Debug;

use simplex::program::ProgramSchema;
use simplex::simplicityhl::Arguments;

use simplex::fuzz::proptest::strategy::Strategy;
use simplex::fuzz::{FuzzProgramTrait, FuzzProgramValues, Scenario};
use simplex::{FuzzMode, TestContext};

pub use simplex::fuzz::Expect;

const DEFAULT_SPENT_INPUT: usize = 0;

/// One call into a dispatch test contract (`function_index` + operands + expected values).
///
/// Implemented once per test module for its `Case` builder, so fuzz tests only
/// describe how generated inputs become a case.
pub trait DispatchFuzzCase: 'static {
    type Program: FuzzProgramTrait;

    /// Contract name used in fuzz scenarios and diagnostics.
    const PROGRAM_NAME: &'static str;

    fn arguments() -> <Self::Program as ProgramSchema>::Arguments;

    fn into_witness(self) -> <Self::Program as ProgramSchema>::Witness;
}

/// Fuzz `inputs` through `build`: every generated case must be accepted.
pub fn accepts<C, T>(
    context: &TestContext<FuzzMode>,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
) where
    C: DispatchFuzzCase,
    T: Debug + 'static,
{
    check(context, inputs, build, Expect::Accept);
}

/// Fuzz `inputs` through `build`: every generated case must be rejected.
pub fn rejects<C, T>(
    context: &TestContext<FuzzMode>,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
) where
    C: DispatchFuzzCase,
    T: Debug + 'static,
{
    check(context, inputs, build, Expect::Reject);
}

/// Fuzz `inputs` through `build` and expect `expect` for every generated case.
pub fn check<C, T>(
    context: &TestContext<FuzzMode>,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
    expect: Expect,
) where
    C: DispatchFuzzCase,
    T: Debug + 'static,
{
    let arguments: Arguments = C::arguments().into();
    let values = inputs.prop_map(move |input| {
        FuzzProgramValues::from((arguments.clone(), build(input).into_witness().into()))
    });

    let mut scenario = Scenario::default();
    scenario
        .add::<C::Program>(C::PROGRAM_NAME)
        .spends(DEFAULT_SPENT_INPUT)
        .custom_strategy(values);

    run_scenario(context, scenario, expect);
}

/// Run a hand-built scenario, for tests that need more than one program or custom bindings.
pub fn run_scenario(context: &TestContext<FuzzMode>, scenario: Scenario, expect: Expect) {
    let engine = context.engine(scenario).spec(move |_| expect);

    // Negative scenarios deliberately generate no accepting witness.
    let engine = if expect == Expect::Reject {
        engine.allow_unspendable()
    } else {
        engine
    };

    engine.run();
}
