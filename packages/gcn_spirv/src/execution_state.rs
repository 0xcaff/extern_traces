use crate::execution_state_input::ExecutionStateInput;
use crate::logger::Logger;
use crate::r#loop::make_loop;
use crate::ExternStorageInfo;
use gcn::instructions::operands::{
    ScalarDestinationOperand, ScalarSourceOperand, SourceOperand, VectorGPR,
};
use gcn::resources::VertexBufferResource;
use rspirv::dr::Builder;
use rspirv::spirv;
use rspirv::spirv::StorageClass;
use std::collections::HashMap;

/// Holds SPIR-V representation of GCN execution state, notably:
/// * 104 sgpr registers, initialized with values passed as shader arguments.
/// * 253 vgpr registers, initialized as zero.
/// * A container for accessing direct memory
pub struct ExecutionState {
    sgpr: spirv::Word,
    vgpr: spirv::Word,
    vcc_lo: spirv::Word,
    vcc_hi: spirv::Word,
    exec_lo: spirv::Word,
    exec_hi: spirv::Word,
    m0: spirv::Word,
    thread_id: spirv::Word,
    pub scc: spirv::Word,
    pub global_types: GlobalTypes,
    pub logger: Logger,
    pub glslstd450_import: spirv::Word,

    pub buffers: HashMap<VertexBufferResource, ExternStorageInfo>,
}

impl ExecutionState {
    pub fn new(
        builder: &mut Builder,
        input: &ExecutionStateInput,
        thread_id: spirv::Word,
        global_types: GlobalTypes,
    ) -> Result<ExecutionState, anyhow::Error> {
        let u32_ptr_type = builder.type_pointer(None, StorageClass::Function, global_types.u32);
        let const_0 = builder.constant_bit32(global_types.u32, 0);
        let const_1 = builder.constant_bit32(global_types.u32, 1);

        let vgpr = {
            let vgpr_array_type_pointer =
                builder.type_pointer(None, StorageClass::Function, input.vgpr_array_type);

            let vgpr =
                builder.variable(vgpr_array_type_pointer, None, StorageClass::Function, None);

            builder.name(vgpr, "vgpr");

            vgpr
        };

        let vgpr_idx = {
            let vgpr_idx = builder.variable(u32_ptr_type, None, StorageClass::Function, None);
            builder.name(vgpr_idx, "vgpr_idx");

            vgpr_idx
        };

        let sgpr = {
            let sgpr_array_type_pointer =
                builder.type_pointer(None, StorageClass::Function, input.sgpr_array_type);

            let sgpr =
                builder.variable(sgpr_array_type_pointer, None, StorageClass::Function, None);

            builder.name(sgpr, "sgpr");

            sgpr
        };

        let sgpr_idx = {
            let sgpr_idx = builder.variable(u32_ptr_type, None, StorageClass::Function, None);
            builder.name(sgpr_idx, "sgpr_idx");

            sgpr_idx
        };

        let vcc_lo = {
            let ptr = builder.type_pointer(None, StorageClass::Function, global_types.u32);
            builder.name(ptr, "vcc_lo");

            builder.variable(ptr, None, StorageClass::Function, None)
        };

        let vcc_hi = {
            let ptr = builder.type_pointer(None, StorageClass::Function, global_types.u32);
            builder.name(ptr, "vcc_hi");

            builder.variable(ptr, None, StorageClass::Function, None)
        };

        let exec_lo = {
            let ptr = builder.type_pointer(None, StorageClass::Function, global_types.u32);
            builder.name(ptr, "exec_lo");

            builder.variable(ptr, None, StorageClass::Function, None)
        };

        let exec_hi = {
            let ptr = builder.type_pointer(None, StorageClass::Function, global_types.u32);
            builder.name(ptr, "exec_hi");

            builder.variable(ptr, None, StorageClass::Function, None)
        };

        let m0 = {
            let ptr = builder.type_pointer(None, StorageClass::Function, global_types.u32);
            builder.name(ptr, "m0");

            builder.variable(ptr, None, StorageClass::Function, None)
        };

        let scc = {
            let ptr = builder.type_pointer(None, StorageClass::Function, global_types.bool);

            builder.name(ptr, "scc");

            builder.variable(ptr, None, StorageClass::Function, None)
        };

        let glslstd450_import = builder.ext_inst_import("GLSL.std.450");

        Ok(ExecutionState {
            vgpr: {
                builder.store(vgpr_idx, const_0, None, [])?;

                make_loop(
                    builder,
                    |builder| {
                        let vgpr_idx_value =
                            builder.load(global_types.u32, None, vgpr_idx, None, [])?;

                        Ok((
                            builder.u_less_than(
                                global_types.bool,
                                None,
                                vgpr_idx_value,
                                input.vgpr_count_id,
                            )?,
                            vgpr_idx_value,
                        ))
                    },
                    |builder, vgpr_idx_value| {
                        let output_ptr =
                            builder.type_pointer(None, StorageClass::Function, global_types.u32);
                        let output_ptr =
                            builder.access_chain(output_ptr, None, vgpr, [*vgpr_idx_value])?;

                        builder.store(output_ptr, const_0, None, [])?;

                        Ok(())
                    },
                    |builder, vgpr_idx_value| {
                        let incremented =
                            builder.i_add(global_types.u32, None, *vgpr_idx_value, const_1)?;

                        builder.store(vgpr_idx, incremented, None, [])?;

                        Ok(())
                    },
                )?;

                vgpr
            },
            sgpr: {
                let sgpr_array_type_ptr =
                    builder.type_pointer(None, StorageClass::Uniform, input.sgpr_array_type);

                builder.store(sgpr_idx, const_0, None, [])?;

                let sgpr_input = builder.access_chain(
                    sgpr_array_type_ptr,
                    None,
                    input.uniform_input_value,
                    [const_0],
                )?;

                builder.name(sgpr_input, "input.sgpr");

                builder.name(sgpr_idx, "idx");

                make_loop(
                    builder,
                    |builder| {
                        let sgpr_idx_value =
                            builder.load(global_types.u32, None, sgpr_idx, None, [])?;

                        Ok((
                            builder.u_less_than(
                                global_types.bool,
                                None,
                                sgpr_idx_value,
                                input.sgpr_count_id,
                            )?,
                            sgpr_idx_value,
                        ))
                    },
                    |builder, sgpr_idx_value| {
                        let input_ptr =
                            builder.type_pointer(None, StorageClass::Uniform, global_types.u32);
                        let input =
                            builder.access_chain(input_ptr, None, sgpr_input, [*sgpr_idx_value])?;
                        let input_value = builder.load(global_types.u32, None, input, None, [])?;

                        let output_ptr =
                            builder.type_pointer(None, StorageClass::Function, global_types.u32);
                        let output_ptr =
                            builder.access_chain(output_ptr, None, sgpr, [*sgpr_idx_value])?;

                        builder.store(output_ptr, input_value, None, [])?;

                        Ok(())
                    },
                    |builder, sgpr_idx_value| {
                        let incremented =
                            builder.i_add(global_types.u32, None, *sgpr_idx_value, const_1)?;

                        builder.store(sgpr_idx, incremented, None, [])?;

                        Ok(())
                    },
                )?;

                sgpr
            },
            vcc_lo,
            vcc_hi,
            exec_lo,
            exec_hi,
            m0,
            thread_id,
            scc,
            global_types,
            logger: Logger::new(builder, const_1),
            glslstd450_import,
            buffers: {
                input
                    .entries
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            key.clone(),
                            ExternStorageInfo {
                                value_pointer: builder.access_chain(
                                    input.memory_field_type,
                                    None,
                                    *value,
                                    [const_0],
                                )?,
                                value_type: input.memory_field_type,
                            },
                        ))
                    })
                    .collect::<Result<HashMap<_, _>, anyhow::Error>>()?
            },
        })
    }

    pub fn set_vcc(&self, builder: &mut Builder, value: spirv::Word) -> Result<(), anyhow::Error> {
        let const_32 = builder.constant_bit32(self.global_types.u32, 32);

        let is_higher =
            builder.u_greater_than_equal(self.global_types.bool, None, self.thread_id, const_32)?;
        let actual_position =
            builder.i_sub(self.global_types.u32, None, self.thread_id, const_32)?;
        let actual_position_lower = builder.select(
            self.global_types.u32,
            None,
            is_higher,
            actual_position,
            self.thread_id,
        )?;

        let one = builder.constant_bit32(self.global_types.u32, 1);
        let mask =
            builder.shift_left_logical(self.global_types.u32, None, one, actual_position_lower)?;

        let lower = builder.load(self.global_types.u32, None, self.vcc_lo, None, [])?;
        let higher = builder.load(self.global_types.u32, None, self.vcc_hi, None, [])?;

        let current_value =
            builder.select(self.global_types.u32, None, is_higher, higher, lower)?;

        let imm_a = builder.bitwise_or(self.global_types.u32, None, current_value, mask)?;

        let imm_c = builder.not(self.global_types.u32, None, mask)?;

        let imm_b = builder.bitwise_and(self.global_types.u32, None, current_value, imm_c)?;

        let new_value = builder.select(self.global_types.u32, None, value, imm_a, imm_b)?;

        let lower_new = builder.select(self.global_types.u32, None, is_higher, lower, new_value)?;
        let higher_new =
            builder.select(self.global_types.u32, None, is_higher, new_value, higher)?;

        builder.store(self.vcc_lo, lower_new, None, [])?;
        builder.store(self.vcc_hi, higher_new, None, [])?;

        Ok(())
    }

    fn register_ptr(
        &self,
        builder: &mut Builder,
        ptr: spirv::Word,
        kind: &str,
        idx: u32,
    ) -> Result<spirv::Word, anyhow::Error> {
        let const_idx = builder.constant_bit32(self.global_types.u32, idx);

        let ptr_type = builder.type_pointer(None, StorageClass::Function, self.global_types.u32);

        let value = builder.access_chain(ptr_type, None, ptr, [const_idx])?;
        builder.name(value, format!("{}[{}]", kind, idx));

        Ok(value)
    }

    pub fn thread_exec(&self, builder: &mut Builder) -> Result<spirv::Word, anyhow::Error> {
        Ok(self.is_thread_zero(builder, &ScalarDestinationOperand::ExecLo)?)
    }

    pub fn execz(&self, builder: &mut Builder) -> Result<spirv::Word, anyhow::Error> {
        Ok(self.is_zero(builder, &ScalarDestinationOperand::ExecLo)?)
    }

    pub fn vccz(&self, builder: &mut Builder) -> Result<spirv::Word, anyhow::Error> {
        Ok(self.is_zero(builder, &ScalarDestinationOperand::VccLo)?)
    }

    fn is_zero(
        &self,
        builder: &mut Builder,
        lo: &ScalarDestinationOperand,
    ) -> Result<spirv::Word, anyhow::Error> {
        let hi = lo.next();

        let lo_ptr = self.scalar_dest_operand(builder, lo)?;
        let hi_ptr = self.scalar_dest_operand(builder, &hi)?;

        let lo = builder.load(self.global_types.u32, None, lo_ptr, None, [])?;
        let hi = builder.load(self.global_types.u32, None, hi_ptr, None, [])?;

        let const_0 = builder.constant_bit32(self.global_types.u32, 0);
        let lhs = builder.i_equal(self.global_types.bool, None, const_0, lo)?;
        let rhs = builder.i_equal(self.global_types.bool, None, const_0, hi)?;

        Ok(builder.logical_and(self.global_types.bool, None, lhs, rhs)?)
    }

    fn is_thread_zero(
        &self,
        builder: &mut Builder,
        lower_register: &ScalarDestinationOperand,
    ) -> Result<spirv::Word, anyhow::Error> {
        let higher_register = lower_register.next();

        let const_0 = builder.constant_bit32(self.global_types.u32, 0);

        let lo_ptr = self.scalar_dest_operand(builder, lower_register)?;
        let lo = builder.load(self.global_types.u32, None, lo_ptr, None, [])?;

        let hi_ptr = self.scalar_dest_operand(builder, &higher_register)?;
        let hi = builder.load(self.global_types.u32, None, hi_ptr, None, [])?;

        let lo_zero = builder.i_equal(self.global_types.bool, None, lo, const_0)?;
        let hi_zero = builder.i_equal(self.global_types.bool, None, hi, const_0)?;

        Ok(builder.logical_and(self.global_types.bool, None, lo_zero, hi_zero)?)
    }

    pub fn scc(&self, builder: &mut Builder) -> Result<spirv::Word, anyhow::Error> {
        Ok(builder.load(self.global_types.bool, None, self.scc, None, [])?)
    }

    pub fn vgpr_ptr(&self, builder: &mut Builder, idx: u32) -> Result<spirv::Word, anyhow::Error> {
        Ok(self.register_ptr(builder, self.vgpr, "vgpr", idx)?)
    }

    pub fn vgpr_load(&self, builder: &mut Builder, idx: u32) -> Result<spirv::Word, anyhow::Error> {
        let ptr = self.vgpr_ptr(builder, idx)?;
        Ok(builder.load(self.global_types.u32, None, ptr, None, [])?)
    }

    pub fn sgpr_ptr(&self, builder: &mut Builder, idx: u32) -> Result<spirv::Word, anyhow::Error> {
        Ok(self.register_ptr(builder, self.sgpr, "sgpr", idx)?)
    }

    pub fn sgpr_load(&self, builder: &mut Builder, idx: u32) -> Result<spirv::Word, anyhow::Error> {
        let ptr = self.sgpr_ptr(builder, idx)?;
        Ok(builder.load(self.global_types.u32, None, ptr, None, [])?)
    }

    pub fn load_src(
        &self,
        builder: &mut Builder,
        src: &SourceOperand,
        literal_constant: &Option<u32>,
    ) -> Result<spirv::Word, anyhow::Error> {
        Ok(match src {
            SourceOperand::Scalar(src) => {
                self.load_scalar_source_operand(builder, src, literal_constant)?
            }
            SourceOperand::VectorGPR(VectorGPR { register_idx }) => {
                let ptr = self.vgpr_ptr(builder, *register_idx as _)?;
                builder.load(self.global_types.u32, None, ptr, None, [])?
            }
            unknown => {
                unimplemented!("source operand {:#?}", unknown);
            }
        })
    }

    pub fn load_scalar_source_operand(
        &self,
        builder: &mut Builder,
        src: &ScalarSourceOperand,
        literal_constant: &Option<u32>,
    ) -> Result<spirv::Word, anyhow::Error> {
        Ok(match src {
            ScalarSourceOperand::Destination(dst) => {
                let ptr = self.scalar_dest_operand(builder, dst)?;
                builder.load(self.global_types.u32, None, ptr, None, [])?
            }
            ScalarSourceOperand::IntegerConstant(constant) => {
                let raw = constant.value() as u32;
                builder.constant_bit32(self.global_types.u32, raw)
            }
            ScalarSourceOperand::FloatConstant(constant) => {
                let raw = bytemuck::cast(constant.value());
                builder.constant_bit32(self.global_types.u32, raw)
            }
            ScalarSourceOperand::LiteralConstant => {
                let value = literal_constant.unwrap();
                builder.constant_bit32(self.global_types.u32, value)
            }
            unknown => {
                unimplemented!("source operand {:#?}", unknown);
            }
        })
    }

    pub fn scalar_dest_operand(
        &self,
        builder: &mut Builder,
        sdst: &ScalarDestinationOperand,
    ) -> Result<spirv::Word, anyhow::Error> {
        let ptr = match sdst {
            ScalarDestinationOperand::ScalarGPR(idx) => self.sgpr_ptr(builder, *idx as _)?,
            ScalarDestinationOperand::VccLo => self.vcc_lo,
            ScalarDestinationOperand::VccHi => self.vcc_hi,
            ScalarDestinationOperand::ExecLo => self.exec_lo,
            ScalarDestinationOperand::ExecHi => self.exec_hi,
            ScalarDestinationOperand::M0 => self.m0,
            _ => todo!(),
        };

        Ok(ptr)
    }
}

pub struct GlobalTypes {
    pub f32: spirv::Word,
    pub u32: spirv::Word,
    pub bool: spirv::Word,
}

impl GlobalTypes {
    pub fn new(builder: &mut Builder) -> GlobalTypes {
        let type_bool = builder.type_bool();
        builder.name(type_bool, "bool");

        let type_u32 = builder.type_int(32, 0);
        builder.name(type_u32, "u32");

        let type_f32 = builder.type_float(32);
        builder.name(type_f32, "f32");

        Self {
            bool: type_bool,
            u32: type_u32,
            f32: type_f32,
        }
    }
}
