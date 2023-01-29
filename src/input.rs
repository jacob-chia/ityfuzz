use crate::abi::BoxedABI;
use crate::state::FuzzState;
use crate::state_input::StagedVMState;
use crate::{evm, VMState};
use bytes::Bytes;
use libafl::inputs::Input;
use libafl::prelude::{HasLen, HasMaxSize, HasRand, MutationResult, State};
use libafl::Error;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ST: Should VMInputT be the generic type for both inputs?
pub trait VMInputT: Input {
    fn to_bytes(&self) -> Bytes;
    fn mutate<S>(&mut self, state: &mut S) -> MutationResult
    where
        S: State + HasRand + HasMaxSize;
    fn get_caller_mut(&mut self) -> &mut H160;
    fn get_caller(&self) -> H160;
    fn set_caller(&mut self, caller: H160);
    fn get_contract_mut(&mut self) -> &mut H160;
    fn get_contract(&self) -> H160;
    fn get_state_mut(&mut self) -> &mut VMState;
    fn set_state(&mut self, state: VMState);
    fn get_state(&self) -> &evm::VMState;
    fn set_staged_state(&mut self, state: StagedVMState);
    fn get_staged_state(&self) -> &StagedVMState;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VMInput {
    pub caller: H160,
    pub contract: H160,
    pub data: BoxedABI,
    pub sstate: StagedVMState,
}

impl HasLen for VMInput {
    fn len(&self) -> usize {
        self.data.get_bytes().len()
    }
}

impl std::fmt::Debug for VMInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VMInput")
            .field("caller", &self.caller)
            .field("contract", &self.contract)
            // .field("data", &self.data)
            .field("state", &self.sstate)
            .finish()
    }
}

impl VMInputT for VMInput {
    fn to_bytes(&self) -> Bytes {
        self.data.get_bytes()
    }

    fn mutate<S>(&mut self, state: &mut S) -> MutationResult
    where
        S: State + HasRand + HasMaxSize,
    {
        self.data.mutate(state)
    }

    fn get_caller_mut(&mut self) -> &mut H160 {
        &mut self.caller
    }

    fn get_caller(&self) -> H160 {
        self.caller.clone()
    }

    fn set_caller(&mut self, caller: H160) {
        self.caller = caller;
    }

    fn get_contract_mut(&mut self) -> &mut H160 {
        &mut self.contract
    }

    fn get_contract(&self) -> H160 {
        self.contract.clone()
    }

    fn get_state_mut(&mut self) -> &mut VMState {
        &mut self.sstate.state
    }

    fn set_state(&mut self, state: VMState) {
        self.sstate = self.sstate.with_state(state);
    }

    fn get_state(&self) -> &VMState {
        &self.sstate.state
    }

    fn set_staged_state(&mut self, state: StagedVMState) {
        self.sstate = state;
    }

    fn get_staged_state(&self) -> &StagedVMState {
        &self.sstate
    }
}

impl Input for VMInput {
    fn generate_name(&self, idx: usize) -> String {
        format!("input-{:06}.bin", idx)
    }

    fn wrapped_as_testcase(&mut self) {
        // todo!()
    }
}
