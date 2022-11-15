elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::config;

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 50_000_000;

pub enum LoopOp {
    Continue,
    Break,
}

impl LoopOp {
    fn is_break(&self) -> bool {
        return matches!(self, LoopOp::Break);
    }
}

#[elrond_wasm::module]
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
            if loop_op.is_break() {
                break;
            }

            if !self.can_continue_operation(gas_per_iteration) {
                return OperationCompletionStatus::InterruptedBeforeOutOfGas;
            }

            loop_op = process();
        }

        OperationCompletionStatus::Completed
    }

    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left > MIN_GAS_TO_SAVE_PROGRESS + operation_cost
    }
}
