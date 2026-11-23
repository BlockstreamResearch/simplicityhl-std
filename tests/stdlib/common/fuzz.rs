use std::fmt::Debug;

use simplex::fuzz::proptest::strategy::Strategy;
use simplex::fuzz::{FuzzProgramValues, Scenario};
use simplex::simplicityhl::Arguments;
use simplex::{FuzzMode, TestContext};

use simplex::fuzz::Expect;

use crate::common::case::DispatchCase;

const DEFAULT_SPENT_INPUT: usize = 0;

pub fn accepts<C, T>(
    context: &TestContext<FuzzMode>,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
) where
    C: DispatchCase,
    T: Debug + 'static,
{
    check(context, inputs, build, Expect::Accept);
}

pub fn rejects<C, T>(
    context: &TestContext<FuzzMode>,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
) where
    C: DispatchCase,
    T: Debug + 'static,
{
    check(context, inputs, build, Expect::Reject);
}

pub fn check<C, T>(
    context: &TestContext<FuzzMode>,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
    expect: Expect,
) where
    C: DispatchCase,
    T: Debug + 'static,
{
    check_scenario(context, Scenario::default(), inputs, build, expect);
}

pub fn check_scenario<C, T>(
    context: &TestContext<FuzzMode>,
    mut scenario: Scenario,
    inputs: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> C + 'static,
    expect: Expect,
) where
    C: DispatchCase,
    T: Debug + 'static,
{
    let arguments: Arguments = C::arguments().into();
    let values = inputs.prop_map(move |input| {
        FuzzProgramValues::from((arguments.clone(), build(input).into_witness().into()))
    });

    scenario
        .add::<C::Program>(C::PROGRAM_NAME)
        .spends(DEFAULT_SPENT_INPUT)
        .custom_strategy(values);

    run_scenario(context, scenario, expect);
}

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
