use alloc::collections::BTreeMap;
use bits::TryFromBits;
use gcn::instructions::formats::{FormattedInstruction, SMEMInstruction, SOP1Instruction};
use gcn::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use gcn::instructions::ops::{SMEMOpCode, SOP1OpCode};
use gcn::instructions::Instruction;

#[derive(Clone)]
pub struct AnalysisState {
    sgprs: BTreeMap<u8, AnalysisRegisterState>,
}

impl AnalysisState {
    pub fn new(user_data_entries: &BTreeMap<u8, u32>) -> AnalysisState {
        let sgprs = {
            let mut sgprs = BTreeMap::new();

            for (slot, value) in user_data_entries {
                sgprs.insert(*slot, AnalysisRegisterState::Value(*value));
            }

            sgprs
        };

        AnalysisState { sgprs }
    }

    pub fn register_value(&self, sgpr_idx: u8) -> &AnalysisRegisterState {
        self.sgprs
            .get(&sgpr_idx)
            .unwrap_or_else(|| &AnalysisRegisterState::Undetermined)
    }

    pub fn update(&mut self, instruction: &Instruction) {
        // todo(correctness): apply general policy where any registers which get written to become undetermined
        match &instruction.inner {
            FormattedInstruction::SMEM(SMEMInstruction {
                op,
                sbase,
                sdst: ScalarDestinationOperand::ScalarGPR(sdst),
                imm,
                offset,
            }) => {
                let offset = if *imm {
                    *offset as _
                } else {
                    let operand = ScalarSourceOperand::try_from_bits(*offset as u64).unwrap();
                    match operand {
                        ScalarSourceOperand::LiteralConstant => {
                            instruction.literal_constant.unwrap()
                        }
                        ScalarSourceOperand::Destination(ScalarDestinationOperand::ScalarGPR(
                            idx,
                        )) => {
                            let Some(it) = self.sgprs.get(&idx) else {
                                return;
                            };

                            let Some(value) = it.value() else {
                                return;
                            };

                            value
                        }
                        _ => unimplemented!(),
                    }
                };
                match op {
                    SMEMOpCode::s_load_dword
                    | SMEMOpCode::s_load_dwordx2
                    | SMEMOpCode::s_load_dwordx4
                    | SMEMOpCode::s_load_dwordx8
                    | SMEMOpCode::s_load_dwordx16 => {
                        let len = match op {
                            SMEMOpCode::s_load_dword => 0u8,
                            SMEMOpCode::s_load_dwordx2 => 2,
                            SMEMOpCode::s_load_dwordx4 => 4,
                            SMEMOpCode::s_load_dwordx8 => 8,
                            SMEMOpCode::s_load_dwordx16 => 16,
                            _ => {
                                unreachable!()
                            }
                        };

                        let base_address = {
                            let base = (*sbase << 1) as u8;

                            let lower_half = self.sgprs.get(&base).unwrap().value().unwrap();
                            let upper_half = self.sgprs.get(&(base + 1)).unwrap().value().unwrap();

                            lower_half as u64 | ((upper_half as u64) << 32)
                        };

                        for register_addend in 0..len {
                            let register_idx = register_addend + *sdst;

                            self.sgprs.insert(
                                register_idx,
                                AnalysisRegisterState::Memory {
                                    base_address,
                                    offset: (offset as usize + register_addend as usize) * 4,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            FormattedInstruction::SOP1(SOP1Instruction {
                op: SOP1OpCode::s_swappc_b64,
                sdst: ScalarDestinationOperand::ScalarGPR(dst_gpr_idx),
                ..
            }) => {
                let next_program_counter = instruction.program_counter + 4;

                self.sgprs.insert(
                    *dst_gpr_idx,
                    AnalysisRegisterState::Value(next_program_counter as u32),
                );
                self.sgprs.insert(
                    *dst_gpr_idx + 1,
                    AnalysisRegisterState::Value((next_program_counter >> 32) as u32),
                );
            }
            FormattedInstruction::SOP1(SOP1Instruction {
                op: SOP1OpCode::s_mov_b32,
                sdst: ScalarDestinationOperand::ScalarGPR(dst_gpr_idx),
                ssrc0,
            }) => {
                self.sgprs.insert(
                    *dst_gpr_idx,
                    match ssrc0 {
                        ScalarSourceOperand::Destination(ScalarDestinationOperand::ScalarGPR(
                            idx,
                        )) => self.sgprs.get(idx).and_then(|it| it.value()),
                        ScalarSourceOperand::IntegerConstant(value) => Some(value.value() as u32),
                        ScalarSourceOperand::FloatConstant(value) => Some(value.value() as u32),
                        ScalarSourceOperand::LiteralConstant => {
                            Some(instruction.literal_constant.unwrap())
                        }
                        _ => None,
                    }
                    .map_or(AnalysisRegisterState::Undetermined, |it| {
                        AnalysisRegisterState::Value(it)
                    }),
                );
            }
            _ => {}
        }
    }

    pub fn get(&self, register_idx: u8) -> AnalysisRegisterState {
        self.sgprs
            .get(&register_idx)
            .cloned()
            .unwrap_or(AnalysisRegisterState::Undetermined)
    }
}

#[derive(Debug, Clone)]
pub enum AnalysisRegisterState {
    Undetermined,
    Value(u32),
    Memory { base_address: u64, offset: usize },
}

impl AnalysisRegisterState {
    pub fn value(&self) -> Option<u32> {
        match self {
            AnalysisRegisterState::Value(it) => Some(*it),
            AnalysisRegisterState::Memory {
                base_address,
                offset,
            } => Some(unsafe { *((*base_address + (*offset as u64)) as *const u32) }),
            AnalysisRegisterState::Undetermined => None,
        }
    }
}
