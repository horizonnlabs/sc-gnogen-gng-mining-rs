multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc::codec::{EncodeErrorHandler, TopEncodeMulti, TopEncodeMultiOutput};

use super::config;

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 15_000_000;

pub enum LoopOp {
    Continue,
    Break,
}

#[derive(TypeAbi)]
pub enum OperationCompletionStatus {
    Completed,
    InterruptedBeforeOutOfGas,
}

impl OperationCompletionStatus {
    pub fn output_bytes(&self) -> &'static [u8] {
        match self {
            OperationCompletionStatus::Completed => b"completed",
            OperationCompletionStatus::InterruptedBeforeOutOfGas => b"interrupted",
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, OperationCompletionStatus::Completed)
    }
}

impl TopEncodeMulti for OperationCompletionStatus {
    fn multi_encode_or_handle_err<O, H>(&self, output: &mut O, h: H) -> Result<(), H::HandledErr>
    where
        O: TopEncodeMultiOutput,
        H: EncodeErrorHandler,
    {
        output.push_single_value(&self.output_bytes(), h)
    }
}

#[multiversx_sc::module]
pub trait OngoingOperationModule: config::ConfigModule {
    fn run_while_it_has_gas<Process>(&self, mut process: Process) -> OperationCompletionStatus
    where
        Process: FnMut() -> LoopOp,
    {
        let gas_before = self.blockchain().get_gas_left();

        let mut loop_op = process();

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        loop {
            match loop_op {
                LoopOp::Break => return OperationCompletionStatus::Completed,
                LoopOp::Continue => {
                    if !self.can_continue_operation(gas_per_iteration) {
                        return OperationCompletionStatus::InterruptedBeforeOutOfGas;
                    }

                    loop_op = process();
                }
            }
        }
    }

    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left > MIN_GAS_TO_SAVE_PROGRESS + operation_cost
    }
}
