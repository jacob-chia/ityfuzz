use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;

use crate::input::VMInputT;
use crate::rand;
use crate::state_input::StagedVMState;
use bytes::Bytes;
use libafl::prelude::ObserversTuple;
use primitive_types::{H160, H256, U256};
use revm::db::BenchmarkDB;
use revm::Return::{Continue, Revert};
use revm::{
    Bytecode, CallInputs, Contract, CreateInputs, Env, Gas, Host, Interpreter, LatestSpec, Return,
    SelfDestructResult, Spec,
};
use serde::{Deserialize, Serialize};

pub const MAP_SIZE: usize = 256;

pub type VMState = HashMap<H160, HashMap<U256, U256>>;

pub static mut jmp_map: [u8; MAP_SIZE] = [0; MAP_SIZE];
use crate::state::{FuzzState, HasHashToAddress};
pub use jmp_map as JMP_MAP;

#[derive(Clone, Debug)]
pub struct FuzzHost {
    env: Env,
    pub data: VMState,
    code: HashMap<H160, Bytecode>,
    hash_to_address: HashMap<[u8; 4], H160>,
}

impl FuzzHost {
    pub fn new() -> Self {
        Self {
            env: Env::default(),
            data: VMState::new(),
            code: HashMap::new(),
            hash_to_address: HashMap::new(),
        }
    }

    pub fn initalize<S>(&mut self, state: &S)
    where
        S: HasHashToAddress,
    {
        self.hash_to_address = state.get_hash_to_address().clone();
    }

    pub fn set_code(&mut self, address: H160, code: Bytecode) {
        self.code.insert(address, code.to_analysed::<LatestSpec>());
    }
}

const ACTIVE_MATCH_EXT_CALL: bool = true;

impl Host for FuzzHost {
    const INSPECT: bool = true;
    type DB = BenchmarkDB;
    fn step(&mut self, interp: &mut Interpreter, is_static: bool) -> Return {
        unsafe {
            // println!("{}", *interp.instruction_pointer);
            match *interp.instruction_pointer {
                0x57 => {
                    let jump_dest = if interp.stack.peek(0).expect("stack underflow").is_zero() {
                        interp.stack.peek(1).expect("stack underflow").as_u64()
                    } else {
                        1
                    };
                    JMP_MAP[(interp.program_counter() ^ (jump_dest as usize)) % MAP_SIZE] =
                        (JMP_MAP[(interp.program_counter() ^ (jump_dest as usize)) % MAP_SIZE] + 1)
                            % 255;
                }
                _ => {}
            }
        }
        return Continue;
    }

    fn step_end(&mut self, interp: &mut Interpreter, is_static: bool, ret: Return) -> Return {
        return Continue;
    }

    fn env(&mut self) -> &mut Env {
        return &mut self.env;
    }

    fn load_account(&mut self, address: H160) -> Option<(bool, bool)> {
        Some((
            true,
            true, // self.data.contains_key(&address) || self.code.contains_key(&address),
        ))
    }

    fn block_hash(&mut self, number: U256) -> Option<H256> {
        println!("blockhash {}", number);

        Some(
            H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        )
    }

    fn balance(&mut self, address: H160) -> Option<(U256, bool)> {
        println!("balance");

        Some((U256::max_value(), true))
    }

    fn code(&mut self, address: H160) -> Option<(Bytecode, bool)> {
        println!("code");
        match self.code.get(&address) {
            Some(code) => Some((code.clone(), true)),
            None => Some((Bytecode::new(), true)),
        }
    }

    fn code_hash(&mut self, address: H160) -> Option<(H256, bool)> {
        Some((
            H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
            true,
        ))
    }

    fn sload(&mut self, address: H160, index: U256) -> Option<(U256, bool)> {
        match self.data.get(&address) {
            Some(account) => Some((account.get(&index).unwrap_or(&U256::zero()).clone(), true)),
            None => Some((U256::zero(), true)),
        }
    }

    fn sstore(
        &mut self,
        address: H160,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        match self.data.get_mut(&address) {
            Some(account) => {
                account.insert(index, value);
            }
            None => {
                let mut account = HashMap::new();
                account.insert(index, value);
                self.data.insert(address, account);
            }
        };
        Some((U256::from(0), U256::from(0), U256::from(0), true))
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes) {}

    fn selfdestruct(&mut self, address: H160, target: H160) -> Option<SelfDestructResult> {
        return Some(SelfDestructResult::default());
    }

    fn create<SPEC: Spec>(
        &mut self,
        inputs: &mut CreateInputs,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        unsafe {
            println!("create");
        }
        return (
            Continue,
            Some(H160::from_str("0x0000000000000000000000000000000000000000").unwrap()),
            Gas::new(0),
            Bytes::new(),
        );
    }

    fn call<SPEC: Spec>(&mut self, input: &mut CallInputs) -> (Return, Gas, Bytes) {
        if ACTIVE_MATCH_EXT_CALL == true {
            let contract_loc = self
                .hash_to_address
                .get(input.input.slice(0..4).to_vec().as_slice())
                .unwrap();
            let mut interp = Interpreter::new::<LatestSpec>(
                Contract::new_with_context::<LatestSpec>(
                    input.input.clone(),
                    self.code.get(contract_loc).unwrap().clone(),
                    &input.context,
                ),
                1e10 as u64,
            );
            let ret = interp.run::<FuzzHost, LatestSpec>(self);
            return (ret, Gas::new(0), interp.return_value());
        }

        // default behavior
        match self.code.get(&input.contract) {
            Some(code) => {
                let mut interp = Interpreter::new::<LatestSpec>(
                    Contract::new_with_context::<LatestSpec>(
                        input.input.clone(),
                        code.clone(),
                        &input.context,
                    ),
                    1e10 as u64,
                );
                let ret = interp.run::<FuzzHost, LatestSpec>(self);
                return (ret, Gas::new(0), interp.return_value());
            }
            None => {
                return (Revert, Gas::new(0), Bytes::new());
            }
        }

        return (Continue, Gas::new(0), Bytes::new());
    }
}

#[derive(Debug, Clone)]
pub struct EVMExecutor<I, S> {
    pub host: FuzzHost,
    deployer: H160,
    phandom: PhantomData<(I, S)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub output: Bytes,
    pub reverted: bool,
    pub new_state: StagedVMState,
}

impl ExecutionResult {
    pub fn empty_result() -> Self {
        Self {
            output: Bytes::new(),
            reverted: false,
            new_state: StagedVMState::new_uninitialized(),
        }
    }
}

impl<I, S> EVMExecutor<I, S> {
    pub fn new(FuzzHost: FuzzHost, deployer: H160) -> Self {
        Self {
            host: FuzzHost,
            deployer,
            phandom: PhantomData,
        }
    }

    pub fn deploy(&mut self, code: Bytecode, constructor_args: Bytes) -> H160 {
        let deployed_address = rand::generate_random_address();
        let deployer = Contract::new::<LatestSpec>(
            constructor_args,
            code,
            deployed_address,
            self.deployer,
            U256::from(0),
        );
        let mut interp = Interpreter::new::<LatestSpec>(deployer, 1e10 as u64);
        let r = interp.run::<FuzzHost, LatestSpec>(&mut self.host);
        assert_eq!(r, Return::Return);
        self.host.set_code(
            deployed_address,
            Bytecode::new_raw(interp.return_value()).to_analysed::<LatestSpec>(),
        );
        deployed_address
    }

    pub fn execute<OT>(
        &mut self,
        contract_address: H160,
        caller: H160,
        state: &VMState,
        data: Bytes,
        observers: &mut OT,
    ) -> ExecutionResult
    where
        OT: ObserversTuple<I, S>,
    {
        self.host.data = state.clone();
        let call = Contract::new::<LatestSpec>(
            data,
            self.host
                .code
                .get(&contract_address)
                .expect("no code")
                .clone(),
            contract_address,
            caller,
            U256::from(0),
        );
        let mut interp = Interpreter::new::<LatestSpec>(call, 1e10 as u64);
        let r = interp.run::<FuzzHost, LatestSpec>(&mut self.host);
        return ExecutionResult {
            output: interp.return_value(),
            reverted: r != Return::Return,
            new_state: StagedVMState::new_with_state(self.host.data.clone()),
        };
    }
}
