use move_binary_format::CompiledModule;

use crate::{feedback::OracleFeedback, minimizer::SequentialMinimizer, tracer::TxnTrace};

use super::{vm_state::MoveVMState, types::{MoveAddress, MoveSlotTy, MoveOutput, MoveFuzzState, MoveLoc}, input::{MoveFunctionInput, ConciseMoveInput}};


pub struct MoveMinimizer;

type MoveOracleFeedback<'a> = OracleFeedback<
    'a, MoveVMState, MoveAddress, CompiledModule, MoveFunctionInput, MoveLoc,
    MoveSlotTy, MoveOutput, MoveFunctionInput, MoveFuzzState, ConciseMoveInput,
>;

impl<E: libafl::executors::HasObservers>
    SequentialMinimizer<MoveFuzzState, E, MoveLoc, MoveAddress, ConciseMoveInput, MoveOracleFeedback<'_>>
    for MoveMinimizer
{
    fn minimize(
        &mut self,
        state: &mut MoveFuzzState,
        exec: &mut E,
        input: &TxnTrace<MoveLoc, MoveAddress, ConciseMoveInput>,
        objective: &mut MoveOracleFeedback<'_>,
        corpus_id: usize,
    ) -> Vec<ConciseMoveInput> {
        todo!()
    }
}
