use gcn::instructions::formats::{FormattedInstruction, SMEMInstruction, SOP1Instruction};
use gcn::instructions::operands::ScalarDestinationOperand;
use gcn::instructions::ops::{SMEMOpCode, SOP1OpCode};
use gcn::instructions::Instruction;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone)]
pub struct AnalysisState {
    sgprs: HashMap<u8, AnalysisRegisterState>,
}

impl AnalysisState {
    pub fn new(user_data_entries: &BTreeMap<u8, u32>) -> AnalysisState {
        let sgprs = {
            let mut sgprs = HashMap::new();

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
                assert!(*imm);
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

                            let lower_half = match self.sgprs.get(&base).unwrap() {
                                AnalysisRegisterState::Value(value) => *value,
                                _ => {
                                    unimplemented!()
                                }
                            };

                            let upper_half = match self.sgprs.get(&(base + 1)).unwrap() {
                                AnalysisRegisterState::Value(value) => *value,
                                _ => {
                                    unimplemented!()
                                }
                            };

                            lower_half as u64 | ((upper_half as u64) << 32)
                        };

                        for register_addend in 0..len {
                            let register_idx = register_addend + *sdst;

                            self.sgprs.insert(
                                register_idx,
                                AnalysisRegisterState::Memory {
                                    base_address,
                                    offset: (*offset + register_addend) * 4,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            FormattedInstruction::SOP1(SOP1Instruction { op, sdst, .. }) => match op {
                SOP1OpCode::s_swappc_b64 => match sdst {
                    ScalarDestinationOperand::ScalarGPR(gpr_idx) => {
                        let next_program_counter = instruction.program_counter + 4;

                        self.sgprs.insert(
                            *gpr_idx,
                            AnalysisRegisterState::Value(next_program_counter as u32),
                        );
                        self.sgprs.insert(
                            *gpr_idx + 1,
                            AnalysisRegisterState::Value((next_program_counter >> 32) as u32),
                        );
                    }
                    _ => {}
                },
                _ => {}
            },
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
    Memory { base_address: u64, offset: u8 },
}

impl AnalysisRegisterState {
    pub fn value(&self) -> u32 {
        match self {
            AnalysisRegisterState::Value(it) => *it,
            AnalysisRegisterState::Memory {
                base_address,
                offset,
            } => unsafe { *((*base_address + (*offset as u64)) as *const u32) },
            AnalysisRegisterState::Undetermined => unimplemented!(),
        }
    }
}
