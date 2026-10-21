use simplex::fuzz::proptest::strategy::Strategy;
use simplex::fuzz::{Expect, FuzzProgramTrait, Scenario};
use simplex::simplicityhl::{Arguments, WitnessValues};
use simplex::{FuzzMode, TestContext};

/// Keep the existing reference-generated arguments and witness in a one-input scenario.
pub fn run<P: FuzzProgramTrait>(
    context: &TestContext<FuzzMode>,
    strategy: impl Strategy<Value = (Arguments, WitnessValues)> + 'static,
    name: &'static str,
    expect: Expect,
) {

    let mut scenario = Scenario::default();
    scenario
        .add::<P>(name)
        .spends(0)
        .custom_strategy(strategy.prop_map(Into::into));

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
