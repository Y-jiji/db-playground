use revm_interpreter::{Host, Interpreter, InstructionResult, Gas, CreateInputs, CallInputs, SelfDestructResult};
use revm_primitives::{Env, U256, B160, B256, Bytecode, KECCAK_EMPTY, Bytes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostState {
    // if host can stop
    Execute,
    // reading, the read value is stored in ProxyHost val
    Read(U256),
    // read index
    AfterRead,
    // write index and value
    PeekAndWr(U256, U256),
    // write index and value, the peeked value is stored in ProxyHost val
    AfterPeek(U256, U256),
    // after write operation
    AfterWrite,
}

#[derive(Debug, Clone)]
pub struct ProxyHost {
    pub env: Env,
    // action
    pub act: HostState,
    // the read value
    pub val: Option<U256>,
}

impl ProxyHost {
    pub fn new(env: Env) -> Self {
        Self { env, act: HostState::Execute, val: None }
    }
}

impl Host for ProxyHost {
    fn step(&mut self, _interp: &mut Interpreter, _is_static: bool) -> InstructionResult {
        InstructionResult::Continue
    }

    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _is_static: bool,
        _ret: InstructionResult,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    fn env(&mut self) -> &mut Env {
        &mut self.env
    }

    fn load_account(&mut self, _address: B160) -> Option<(bool, bool)> {
        Some((true, true))
    }

    fn block_hash(&mut self, _number: U256) -> Option<B256> {
        Some(B256::zero())
    }

    fn balance(&mut self, _address: B160) -> Option<(U256, bool)> {
        Some((U256::ZERO, false))
    }

    fn code(&mut self, _address: B160) -> Option<(Bytecode, bool)> {
        Some((Bytecode::default(), false))
    }

    fn code_hash(&mut self, __address: B160) -> Option<(B256, bool)> {
        Some((KECCAK_EMPTY, false))
    }

    fn sload(&mut self, _address: B160, index: U256) 
    -> Option<(U256, bool)> {
        use HostState::*;
        match (&self.act, &self.val) {
            (Execute, None) => {
                // index should only be used here
                self.act = Read(index);
                None
            }
            (AfterRead, None) => {
                let out = Some((U256::ZERO, true));
                self.act = Execute;
                self.val = None;
                out
            }
            (AfterRead, Some(val)) => {
                let out = Some((*val, false));
                self.act = Execute;
                self.val = None;
                out
            }
            _s => panic!("illegal state {_s:?} encountered!")
        }
    }

    fn sstore(&mut self, _address: B160, index: U256, value: U256) -> Option<(U256, U256, U256, bool)> {
        use HostState::*;
        match (&self.act, &self.val) {
            (Execute, None) => {
                // index should and value only be used here
                self.act = PeekAndWr(index, value);
                None
            },
            (AfterWrite, None) => {
                self.act = Execute;
                self.val = None;
                Some((U256::ZERO, U256::ZERO, value, true))
            },
            (AfterWrite, Some(present)) => {
                let out = Some((U256::ZERO, *present, value, false));
                self.act = Execute;
                self.val = None;
                return out
            }
            _s => panic!("illegal state {_s:?} encountered!")
        }
    }

    fn log(&mut self, _address: B160, _topics: Vec<B256>, _data: Bytes) {
        panic!("Logging, i.e. emitting event is not supported for this host");
    }

    fn selfdestruct(&mut self, _address: B160, _target: B160) -> Option<SelfDestructResult> {
        panic!("Selfdestruct is not supported for this host")
    }

    fn create(
        &mut self,
        _inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<B160>, Gas, Bytes) {
        panic!("Create is not supported for this host")
    }

    fn call(&mut self, _input: &mut CallInputs) -> (InstructionResult, Gas, Bytes) {
        panic!("Call is not supported for this host")
    }
}

