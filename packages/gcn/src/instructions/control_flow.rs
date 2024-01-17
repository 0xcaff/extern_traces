use crate::instructions::operands::ScalarSourceOperand;
use crate::{
    FormattedInstruction, Instruction, SOP1Instruction, SOP1OpCode, SOP2Instruction, SOP2OpCode,
    SOPKInstruction, SOPKOpCode, SOPPInstruction, SOPPOpCode,
};

#[derive(Debug)]
pub struct ControlFlowInformation {
    pub has_next_instruction: bool,
    pub branch_target: Option<BranchTarget>,
}

#[derive(Debug)]
pub enum BranchTarget {
    Indirect(IndirectValue),
    Direct(u64),
}

#[derive(Debug)]
pub enum IndirectValue {
    Register(ScalarSourceOperand),
}

impl Instruction {
    pub fn control_flow_information(&self) -> ControlFlowInformation {
        match &self.inner {
            FormattedInstruction::SOPP(SOPPInstruction { op, simm16 }) => {
                let branch_target = Some(BranchTarget::Direct(
                    self.program_counter + (*simm16 as u64 * 4) + 4,
                ));

                match op {
                    SOPPOpCode::s_endpgm => {
                        return ControlFlowInformation {
                            has_next_instruction: false,
                            branch_target: None,
                        }
                    }
                    SOPPOpCode::s_branch => {
                        return ControlFlowInformation {
                            branch_target,
                            has_next_instruction: false,
                        }
                    }
                    SOPPOpCode::s_cbranch_vccz
                    | SOPPOpCode::s_cbranch_vccnz
                    | SOPPOpCode::s_cbranch_execz
                    | SOPPOpCode::s_cbranch_execnz
                    | SOPPOpCode::s_cbranch_scc0
                    | SOPPOpCode::s_cbranch_scc1
                    | SOPPOpCode::s_cbranch_cdbgsys
                    | SOPPOpCode::s_cbranch_cdbguser
                    | SOPPOpCode::s_cbranch_cdbgsys_and_user
                    | SOPPOpCode::s_cbranch_cdbgsys_or_user => {
                        return ControlFlowInformation {
                            branch_target,
                            has_next_instruction: true,
                        }
                    }
                    SOPPOpCode::s_trap => {
                        unimplemented!("trap handler not implemented")
                    }
                    _ => {}
                }
            }
            FormattedInstruction::SOP1(SOP1Instruction { op, sdst, ssrc0 }) => match op {
                SOP1OpCode::s_setpc_b64 | SOP1OpCode::s_swappc_b64 | SOP1OpCode::s_rfe_b64 => {
                    return ControlFlowInformation {
                        has_next_instruction: false,
                        branch_target: Some(BranchTarget::Indirect(IndirectValue::Register(
                            ssrc0.clone(),
                        ))),
                    }
                }
                _ => {}
            },
            FormattedInstruction::SOP2(SOP2Instruction {
                op: SOP2OpCode::s_cbranch_g_fork,
                ssrc1,
                ..
            }) => {
                return ControlFlowInformation {
                    has_next_instruction: true,
                    branch_target: Some(BranchTarget::Indirect(IndirectValue::Register(
                        ssrc1.clone(),
                    ))),
                }
            }
            FormattedInstruction::SOPK(SOPKInstruction {
                op: SOPKOpCode::s_cbranch_i_fork,
                simm16,
                ..
            }) => {
                return ControlFlowInformation {
                    has_next_instruction: false,
                    branch_target: Some(BranchTarget::Direct(
                        self.program_counter + 4 + (*simm16 as u64),
                    )),
                }
            }
            _ => {}
        };

        ControlFlowInformation {
            has_next_instruction: true,
            branch_target: None,
        }
    }
}
