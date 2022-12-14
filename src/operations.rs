elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::config;

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 50_000_000;

pub enum LoopOp {
    Continue,
    Break,
    ForceBreakBeforeCompleted,
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
            match loop_op {
                LoopOp::Break => return OperationCompletionStatus::Completed,
                LoopOp::ForceBreakBeforeCompleted => {
                    return OperationCompletionStatus::InterruptedBeforeOutOfGas
                }
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
