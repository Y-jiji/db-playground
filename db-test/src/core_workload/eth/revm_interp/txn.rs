use super::host::*;
use super::misc::*;
use revm_interpreter::*;
use revm_primitives::*;
use typing::tx::*;
use typing::constraint::TxCkpt;

pub struct REVMInterpTxnInner {
    id: usize,
    host: ProxyHost,
    interp: Interpreter,
    isinit: bool,
}

pub struct REVMInterpTxn(pub Box<REVMInterpTxnInner>);

unsafe impl Send for REVMInterpTxnInner {}
unsafe impl Sync for REVMInterpTxnInner {}

impl std::fmt::Debug for REVMInterpTxn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{id: {:?}, isinit: {:?}, host: {:?}, stack: {:?}, memory: {:?}, pc: {:?}}}", self.0.id, self.0.isinit, self.0.host, self.0.interp.stack(), self.0.interp.memory(), self.0.interp.instruction_pointer)
    }
}

impl TxCkpt for REVMInterpTxn {
    type Ckpt = ();
    fn make(&mut self) -> Self::Ckpt {
        assert!(self.0.isinit, "{self:?} is not at initial point, but REVMInterpTxn can only make a checkpoint at initial point");
    }
    fn goto(&mut self, (): Self::Ckpt) {
        let REVMInterpTxnInner { host, interp, isinit, .. } = self.0.as_mut();
        host.act = HostState::Execute;
        host.val = None;
        *interp = Interpreter::new(std::mem::replace(&mut interp.contract, Default::default()), u64::MAX, false);
        *isinit = true;
    }
}

impl REVMInterpTxnInner {
    pub fn new(id: usize, bytecode: BytecodeLocked, input: Bytes) 
    -> Self {
        let contract = Contract {
            input, bytecode,
            ..Default::default()
        };
        let interp = Interpreter::new(contract, u64::MAX, false);
        let host = ProxyHost::new(Env::default());
        Self {id, host, interp, isinit: true}
    }
    pub fn continue_sload(&mut self) {
        self.interp.stack.push(U256::MAX)
            .expect("push action should not fail, a pop just occurred before");
        unsafe {
            // wind pc
            self.interp.instruction_pointer = 
            self.interp.instruction_pointer.offset(-1);
        };
        self.interp.instruction_result = InstructionResult::Continue;
        self.interp.step::<_, BerlinSpec>(&mut self.host);
    }
    pub fn continue_sstore(&mut self) {
        self.interp.stack.push(U256::MAX)
            .expect("push action should not fail, a pop just occurred before");
        self.interp.stack.push(U256::MAX)
            .expect("push action should not fail, a pop just occurred before");
        unsafe {
            // wind pc
            self.interp.instruction_pointer = 
            self.interp.instruction_pointer.offset(-1);
        };
        self.interp.instruction_result = InstructionResult::Continue;
        self.interp.step::<_, BerlinSpec>(&mut self.host);
    }
}

impl Tx<EVMU256Tup> for REVMInterpTxn {
    type I = usize;
    #[inline(always)]
    fn id(&self) -> Self::I {
        self.0.id
    }
    type Out = Bytes;
    #[inline(always)]
    fn cl(self) -> Option<Self::Out> {
        use InstructionResult::*;
        match self.0.interp.instruction_result {
            Revert => None,
            Return | Stop => Some(self.0.interp.return_value()),
            _s => panic!("illegal state {_s:?}")
        }
    }
    type Map = EVMU256Map;
    type Prp = EVMU256Prp;
    #[inline(always)]
    fn go(self) -> RWClosure<Self, Self::Prp, Self::Map> {
        use HostState::*;
        use InstructionResult::*;
        match (&self.0.host.act, self.0.interp.instruction_result) {
            (Execute, Continue) => 
                RWClosure::Op(self),
            (Execute, Return) | (Execute, Stop) => 
                RWClosure::Cl(self, End::Ready),
            (Execute, Revert) => 
                RWClosure::Cl(self, End::Abort),
            (Read(index), FatalExternalError) => {
                let prp = EVMU256Prp(*index);
                RWClosure::Rd(self, prp)
            },
            (PeekAndWr(index, _value), FatalExternalError) => {
                let prp = EVMU256Prp(*index);
                RWClosure::Rd(self, prp)
            }
            (AfterPeek(index, value), FatalExternalError) => {
                let map = EVMU256Map(Some((*index, Some(*value))));
                RWClosure::Wr(self, map)
            }
            _s => panic!("illegal state {_s:?}"),
        }
    }
    #[inline(always)]
    fn op(mut self) -> Self {
        self.0.isinit = false;
        self.0.interp.step::<_, BerlinSpec>(&mut self.0.host);
        self
    }
    #[inline(always)]
    fn rd(mut self, EVMU256Map(map): Self::Map) -> Self {
        use HostState::*;
        match self.0.host.act {
            Read(_index) => {
                self.0.host.val = match map {
                    Some((_index, value)) => Some(value.expect("map should be null if there is no such value")),
                    None => None,
                };
                self.0.host.act = AfterRead;
                self.0.as_mut().continue_sload();
                self
            },
            PeekAndWr(index, value) => {
                self.0.host.val = match map {
                    Some((_index, read_val)) => Some(read_val.expect("map should be null if there is no such value")),
                    None => None,
                };
                self.0.host.act = AfterPeek(index, value);
                self
            },
            _s => panic!("illegal state {_s:?}"),
        }
    }
    #[inline(always)]
    fn wr(mut self) -> Self {
        use HostState::*;
        match self.0.host.act {
            AfterPeek(_index, _value) => {
                self.0.host.act = AfterWrite;
                self.0.as_mut().continue_sstore();
                self
            }
            _s => panic!("illegal state {_s:?}"),
        }
    }
}