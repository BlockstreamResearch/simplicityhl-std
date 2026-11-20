use simplex::program::{Program, ProgramSchema};
use simplex::simplicityhl::Arguments;

use simplex::fuzz::FuzzProgramTrait;

use crate::common::core::Expect;
use crate::common::unit;

pub trait DispatchCase: Sized + 'static {
    type Program: FuzzProgramTrait;

    const PROGRAM_NAME: &'static str = "program";

    fn arguments() -> <Self::Program as ProgramSchema>::Arguments;

    fn into_witness(self) -> <Self::Program as ProgramSchema>::Witness;

    fn program() -> Program {
        let arguments: Arguments = Self::arguments().into();

        Self::Program::build_program(&arguments)
            .unwrap_or_else(|error| panic!("{} does not compile: {error}", Self::PROGRAM_NAME))
    }

    #[track_caller]
    fn accepts(self) {
        unit::check(&Self::program(), self.into_witness(), Expect::Ok);
    }

    #[track_caller]
    fn rejects(self, reason: Expect) {
        assert_ne!(
            reason,
            Expect::Ok,
            "use `accepts` for cases that must succeed"
        );

        unit::check(&Self::program(), self.into_witness(), reason);
    }
}
