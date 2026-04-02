use solana_program::entrypoint::ProgramResult;

pub trait InstructionProcessor {
    fn process(self) -> ProgramResult;
}
