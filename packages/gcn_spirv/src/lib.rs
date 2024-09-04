use std::collections::{BTreeMap, HashMap};

use anyhow::bail;
use gcn::instructions::formats::exp::ExportTarget;
use gcn::instructions::formats::vop3::{OpCode, OutputModifier, TransformedOperand};
use gcn::instructions::formats::{ExpInstruction, FormattedInstruction, VOP3Instruction};
use gcn::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand, VectorGPR};
use gcn::instructions::ops::{
    MIMGOpCode, MUBUFOpCode, SMEMOpCode, SOP1OpCode, SOP2OpCode, SOPPOpCode, VINTRPOpCode,
    VOP1OpCode, VOP2OpCode, VOPCOpCode,
};
use gcn::instructions::Instruction;
use itertools::Itertools;
use rspirv::dr::{Builder, Operand};
use rspirv::spirv;
use rspirv::spirv::{GLOp, SelectionControl, StorageClass};

use crate::execution_state::ExecutionState;
use gcn::resources::{SamplerResource, TextureBufferResource, VertexBufferResource};

pub mod analysis;
mod execution_state;
pub mod execution_state_input;
mod logger;
mod r#loop;
pub mod module;

pub fn wrap_exec_condition(
    builder: &mut Builder,
    context: &ExecutionState,
    instruction: &Instruction,
    f: impl Fn(&mut Builder) -> Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    context
        .logger
        .log(builder, &instruction.display().to_string(), &[])?;

    context
        .logger
        .log(builder, &format!("{:#?}", instruction.inner), &[])?;

    let exec_mask_disables = instruction.inner.exec_mask_disables();
    if exec_mask_disables {
        let exec_enabled = context.thread_exec(builder)?;

        let branch_merge = builder.id();
        let true_label = builder.id();
        builder.selection_merge(branch_merge, SelectionControl::NONE)?;

        builder.branch_conditional(exec_enabled, true_label, branch_merge, [])?;

        builder.begin_block(Some(true_label))?;

        f(builder)?;

        builder.branch(branch_merge)?;

        builder.begin_block(Some(branch_merge))?;
    } else {
        f(builder)?;
    }

    Ok(())
}

pub struct ExternStorageInfo {
    /// Pointer to storage location, something which can be `OpLoad` or `OpStore` with.
    pub value_pointer: spirv::Word,
    pub value_type: spirv::Word,
}

fn translate_instruction(
    builder: &mut Builder,
    imports: &BTreeMap<u8, ExternStorageInfo>,
    image_sample_instances: &BTreeMap<u64, (([u32; 8], TextureBufferResource), SamplerResource)>,
    image_imports: &HashMap<
        (([u32; 8], TextureBufferResource), SamplerResource),
        ExternStorageInfo,
    >,
    buffer_usage_instances: &BTreeMap<u64, VertexBufferResource>,
    exports: &BTreeMap<ExportTarget, ExternStorageInfo>,
    instruction: &Instruction,
    context: &ExecutionState,
) -> Result<(), anyhow::Error> {
    let program_counter = instruction.program_counter;

    match &instruction.inner {
        FormattedInstruction::MUBUF(instr) => match instr.op {
            MUBUFOpCode::buffer_load_format_x
            | MUBUFOpCode::buffer_load_format_xy
            | MUBUFOpCode::buffer_load_format_xyz
            | MUBUFOpCode::buffer_load_format_xyzw
            | MUBUFOpCode::buffer_store_format_x
            | MUBUFOpCode::buffer_store_format_xy
            | MUBUFOpCode::buffer_store_format_xyz
            | MUBUFOpCode::buffer_store_format_xyzw => {
                let buffer_resource = buffer_usage_instances.get(&program_counter).unwrap();
                let buffer_storage_info = context.buffers.get(&buffer_resource).unwrap();

                enum Kind {
                    Store,
                    Load,
                }

                let (count, kind) = match instr.op {
                    MUBUFOpCode::buffer_load_format_x => (1, Kind::Load),
                    MUBUFOpCode::buffer_load_format_xy => (2, Kind::Load),
                    MUBUFOpCode::buffer_load_format_xyz => (3, Kind::Load),
                    MUBUFOpCode::buffer_load_format_xyzw => (4, Kind::Load),
                    MUBUFOpCode::buffer_store_format_x => (1, Kind::Store),
                    MUBUFOpCode::buffer_store_format_xy => (2, Kind::Store),
                    MUBUFOpCode::buffer_store_format_xyz => (3, Kind::Store),
                    MUBUFOpCode::buffer_store_format_xyzw => (4, Kind::Store),
                    _ => unreachable!(),
                };

                let stride = buffer_resource.stride as u16;

                // todo: implement soffset
                let index = if instr.idxen {
                    let tmp = context.vgpr_ptr(builder, instr.vaddr.register_idx as _)?;
                    builder.load(context.global_types.u32, None, tmp, None, [])?
                } else {
                    builder.constant_bit32(context.global_types.u32, 0)
                };

                if instr.offen {
                    bail!("offen not implemented");
                }

                if instr.offset.0 != 0 {
                    bail!("offset not implemented");
                }

                let address = {
                    let stride_value =
                        builder.constant_bit32(context.global_types.u32, stride as _);
                    builder.i_mul(context.global_types.u32, None, index, stride_value)?
                };

                for idx in 0..count {
                    let memory_ptr = {
                        let const_2 = builder.constant_bit32(context.global_types.u32, 2);

                        let local_memory_offset = builder.shift_right_logical(
                            context.global_types.u32,
                            None,
                            address,
                            const_2,
                        )?;

                        let idx_value = builder.constant_bit32(context.global_types.u32, idx as _);

                        let local_memory_offset = builder.i_add(
                            context.global_types.u32,
                            None,
                            local_memory_offset,
                            idx_value,
                        )?;

                        let ptr_type = builder.type_pointer(
                            None,
                            StorageClass::StorageBuffer,
                            context.global_types.u32,
                        );

                        let access_chain = builder.access_chain(
                            ptr_type,
                            None,
                            buffer_storage_info.value_pointer,
                            [local_memory_offset],
                        )?;

                        access_chain
                    };

                    let register_idx = instr.vdata.register_idx + idx;
                    let vgpr = context.vgpr_ptr(builder, register_idx as _)?;

                    let (src, dst) = match kind {
                        Kind::Load => (memory_ptr, vgpr),
                        Kind::Store => (vgpr, memory_ptr),
                    };

                    let memory_value =
                        builder.load(context.global_types.u32, None, src, None, [])?;

                    builder.store(dst, memory_value, None, [])?;
                }
            }
            unknown => unimplemented!("{:#?}", unknown),
        },
        FormattedInstruction::SMEM(instr) => match instr.op {
            SMEMOpCode::s_buffer_load_dword
            | SMEMOpCode::s_buffer_load_dwordx2
            | SMEMOpCode::s_buffer_load_dwordx4
            | SMEMOpCode::s_buffer_load_dwordx8
            | SMEMOpCode::s_buffer_load_dwordx16 => {
                let buffer_resource = buffer_usage_instances.get(&program_counter).unwrap();
                let buffer_storage_info = context.buffers.get(&buffer_resource).unwrap();

                let dest_base_register = expect_sgpr(&instr.sdst)?;
                let dest_count: u32 = match instr.op {
                    SMEMOpCode::s_buffer_load_dword => 1,
                    SMEMOpCode::s_buffer_load_dwordx2 => 2,
                    SMEMOpCode::s_buffer_load_dwordx4 => 4,
                    SMEMOpCode::s_buffer_load_dwordx8 => 8,
                    SMEMOpCode::s_buffer_load_dwordx16 => 16,
                    _ => unreachable!(),
                };

                let offset = if instr.imm {
                    instr.offset as u32
                } else {
                    unimplemented!()
                };

                for dst_idx in 0..dest_count {
                    let register_idx = (dest_base_register as u32) + dst_idx;
                    let address = offset + dst_idx;
                    let idx = builder.constant_bit32(context.global_types.u32, address);

                    let ptr_type = builder.type_pointer(
                        None,
                        StorageClass::StorageBuffer,
                        context.global_types.u32,
                    );

                    let access_chain = builder.access_chain(
                        ptr_type,
                        None,
                        buffer_storage_info.value_pointer,
                        [idx],
                    )?;

                    let memory_value =
                        builder.load(context.global_types.u32, None, access_chain, None, [])?;

                    let sgpr = context.sgpr_ptr(builder, register_idx as _)?;

                    context.logger.log(
                        builder,
                        &format!("sgpr[{}] = %x (address 0x{:x})", register_idx, address),
                        &[memory_value],
                    )?;

                    builder.store(sgpr, memory_value, None, [])?;
                }
            }
            SMEMOpCode::s_load_dword
            | SMEMOpCode::s_load_dwordx2
            | SMEMOpCode::s_load_dwordx4
            | SMEMOpCode::s_load_dwordx8
            | SMEMOpCode::s_load_dwordx16 => {
                // do nothing for these instructions, they affect analysis
            }
            _ => unimplemented!(),
        },
        FormattedInstruction::VOP1(instr) => match instr.op {
            VOP1OpCode::v_mov_b32 => {
                let src0 = context.load_src(builder, &instr.src0, &instruction.literal_constant)?;
                let dst = context.vgpr_ptr(builder, instr.vdst.register_idx as _)?;

                builder.store(dst, src0, None, [])?;
            }
            VOP1OpCode::v_cvt_f32_i32 => {
                let src0 = context.load_src(builder, &instr.src0, &instruction.literal_constant)?;
                let dst = context.vgpr_ptr(builder, instr.vdst.register_idx as _)?;

                let s32 = builder.type_int(32, 1);

                let src0 = builder.bitcast(s32, None, src0)?;
                let src0 = builder.convert_s_to_f(context.global_types.f32, None, src0)?;
                let src0 = builder.bitcast(context.global_types.u32, None, src0)?;

                builder.store(dst, src0, None, [])?;
            }
            unknown => unimplemented!("unknown {:#?}", unknown),
        },
        FormattedInstruction::VOP2(instr) => {
            translate_vop3(
                &VOP3Instruction::from((*instr).clone()),
                &instruction.literal_constant,
                builder,
                context,
            )?;
        }
        FormattedInstruction::VOP3(instr) => {
            translate_vop3(instr, &instruction.literal_constant, builder, context)?;
        }
        FormattedInstruction::VOPC(instr) => match instr.op {
            VOPCOpCode::v_cmp_gt_u32 => {
                let src0 = context.load_src(builder, &instr.src0, &instruction.literal_constant)?;
                let src1 = context.vgpr_load(builder, instr.vsrc1.register_idx as _)?;

                let result =
                    builder.u_greater_than_equal(context.global_types.bool, None, src0, src1)?;

                let exec = context.thread_exec(builder)?;

                let result = builder.logical_and(context.global_types.bool, None, exec, result)?;

                context.set_vcc(builder, result)?;
            }
            _ => unimplemented!(),
        },
        FormattedInstruction::SOPP(instr) => match instr.op {
            SOPPOpCode::s_cbranch_execz
            | SOPPOpCode::s_cbranch_execnz
            | SOPPOpCode::s_cbranch_scc0
            | SOPPOpCode::s_cbranch_scc1
            | SOPPOpCode::s_cbranch_vccz
            | SOPPOpCode::s_cbranch_vccnz => {
                unimplemented!()
            }
            SOPPOpCode::s_endpgm => {}
            SOPPOpCode::s_waitcnt => {}
            unknown => unimplemented!("unknown {:#?}", unknown),
        },
        FormattedInstruction::SOP2(instr) => match instr.op {
            SOP2OpCode::s_lshl_b32 => {
                let src0 = context.load_scalar_source_operand(
                    builder,
                    &instr.ssrc0,
                    &instruction.literal_constant,
                )?;
                let src1 = context.load_scalar_source_operand(
                    builder,
                    &instr.ssrc1,
                    &instruction.literal_constant,
                )?;

                let mask = builder.constant_bit32(context.global_types.u32, 0b11111);
                let truncated_src1 =
                    builder.bitwise_and(context.global_types.u32, None, mask, src1)?;

                let result = builder.shift_left_logical(
                    context.global_types.u32,
                    None,
                    src0,
                    truncated_src1,
                )?;

                let const_1 = builder.constant_bit32(context.global_types.u32, 1);
                let scc_result =
                    builder.i_not_equal(context.global_types.bool, None, result, const_1)?;

                builder.store(context.scc, scc_result, None, [])?;

                let sgpr_ptr = context.sgpr_ptr(builder, expect_sgpr(&instr.sdst)? as _)?;

                builder.store(sgpr_ptr, result, None, [])?;
            }
            unknown => unimplemented!("unknown {:#?}", unknown),
        },
        FormattedInstruction::EXP(instr) => {
            let export_info = exports.get(&instr.target).unwrap();

            let mut comps = vec![];

            match instr {
                &ExpInstruction {
                    compress: true,
                    target: ExportTarget::RenderTarget(_),
                    ref vsrc0,
                    ref vsrc1,
                    vsrc2: None,
                    vsrc3: None,
                    ..
                } => {
                    for vgpr in [vsrc0, vsrc1] {
                        if let Some(VectorGPR { register_idx }) = vgpr {
                            let value = context.vgpr_load(builder, *register_idx as _).unwrap();

                            let vec_type = builder.type_vector(context.global_types.f32, 2);

                            let vec_value = builder.ext_inst(
                                vec_type,
                                None,
                                context.glslstd450_import,
                                GLOp::UnpackHalf2x16 as _,
                                [Operand::IdRef(value)],
                            )?;

                            comps.push(builder.composite_extract(
                                context.global_types.f32,
                                None,
                                vec_value,
                                [0],
                            )?);
                            comps.push(
                                builder
                                    .composite_extract(
                                        context.global_types.f32,
                                        None,
                                        vec_value,
                                        [1],
                                    )
                                    .unwrap(),
                            );
                        }
                    }
                }
                _ => {
                    for vgpr in [&instr.vsrc0, &instr.vsrc1, &instr.vsrc2, &instr.vsrc3] {
                        if let Some(VectorGPR { register_idx }) = vgpr {
                            let value = context.vgpr_load(builder, (*register_idx) as _)?;
                            let value = builder.bitcast(context.global_types.f32, None, value)?;
                            comps.push(value);
                        };
                    }
                }
            };

            let val = builder.composite_construct(export_info.value_type, None, comps)?;
            builder.store(export_info.value_pointer, val, None, [])?;
        }
        FormattedInstruction::VINTRP(instr) => {
            match instr.op {
                VINTRPOpCode::v_interp_p1_f32 => {
                    // Part one of v_interp, do nothing here, handle everything in v_interp_p2_f32.
                }
                VINTRPOpCode::v_interp_p2_f32 => {
                    // The input parameter is ignored here. We assume it is a barycentric coordinate
                    // associated with the attribute. Our translated Vulkan doesn't expose
                    // barycentric coordinates, interpolating attributes under the hood.

                    let import = imports.get(&instr.attr.0).expect("unknown import");
                    let import_value =
                        builder.load(import.value_type, None, import.value_pointer, None, [])?;
                    let field = builder.composite_extract(
                        context.global_types.f32,
                        None,
                        import_value,
                        [instr.attribute_channel as usize as u32],
                    )?;

                    let field = builder.bitcast(context.global_types.u32, None, field)?;

                    let dst = context.vgpr_ptr(builder, instr.vdst.register_idx as _)?;
                    builder.store(dst, field, None, [])?;
                }
                VINTRPOpCode::v_interp_mov_f32 => {
                    unimplemented!()
                }
            }
        }
        FormattedInstruction::MIMG(instr) => match instr.op {
            MIMGOpCode::image_sample => {
                let image_resource = image_sample_instances.get(&program_counter).unwrap();

                let image_import = image_imports.get(image_resource).unwrap();

                let image_value = builder.load(
                    image_import.value_type,
                    None,
                    image_import.value_pointer,
                    None,
                    [],
                )?;

                let coordinate = {
                    let vec2 = builder.type_vector(context.global_types.f32, 2);
                    let u = context.vgpr_load(builder, instr.vaddr.register_idx as _)?;
                    let u = builder.bitcast(context.global_types.f32, None, u)?;

                    let v = context.vgpr_load(builder, (instr.vaddr.register_idx + 1) as _)?;
                    let v = builder.bitcast(context.global_types.f32, None, v)?;

                    builder.composite_construct(vec2, None, [u, v])?
                };

                let vec4 = builder.type_vector(context.global_types.f32, 4);
                let result = builder.image_sample_implicit_lod(
                    vec4,
                    None,
                    image_value,
                    coordinate,
                    None,
                    [],
                )?;

                let base_data_register_idx = instr.vdata.register_idx;
                for (offset, idx) in instr.dmask.indices().enumerate() {
                    let register_idx = base_data_register_idx + (offset as u8);

                    let value = builder.composite_extract(
                        context.global_types.f32,
                        None,
                        result,
                        [idx as u32],
                    )?;

                    let value = builder.bitcast(context.global_types.u32, None, value)?;

                    let dest_vgpr = context.vgpr_ptr(builder, register_idx as u32)?;
                    builder.store(dest_vgpr, value, None, [])?;
                }
            }
            _ => {
                unimplemented!()
            }
        },
        FormattedInstruction::SOP1(instr) => {
            match instr.op {
                SOP1OpCode::s_mov_b32 => {
                    let dst = context.scalar_dest_operand(builder, &instr.sdst)?;
                    let src = context.load_scalar_source_operand(
                        builder,
                        &instr.ssrc0,
                        &instruction.literal_constant,
                    )?;

                    builder.store(dst, src, None, [])?;
                }
                SOP1OpCode::s_mov_b64 => {
                    // skip
                }
                SOP1OpCode::s_wqm_b64 => {
                    // skip
                }
                SOP1OpCode::s_setpc_b64 => {
                    // skip
                }
                SOP1OpCode::s_and_saveexec_b64 => {
                    let exec_lo_ptr =
                        context.scalar_dest_operand(builder, &ScalarDestinationOperand::ExecLo)?;
                    let exec_lo =
                        builder.load(context.global_types.u32, None, exec_lo_ptr, None, [])?;
                    let lower_ptr = context.scalar_dest_operand(builder, &instr.sdst)?;
                    builder.store(lower_ptr, exec_lo, None, [])?;

                    let exec_hi_ptr =
                        context.scalar_dest_operand(builder, &ScalarDestinationOperand::ExecHi)?;
                    let exec_hi =
                        builder.load(context.global_types.u32, None, exec_hi_ptr, None, [])?;
                    let higher_ptr = context.scalar_dest_operand(builder, &instr.sdst.next())?;
                    builder.store(higher_ptr, exec_hi, None, [])?;

                    let ScalarSourceOperand::Destination(src) = &instr.ssrc0 else {
                        unimplemented!()
                    };

                    let src_lo_ptr = context.scalar_dest_operand(builder, src)?;
                    let src_lo =
                        builder.load(context.global_types.u32, None, src_lo_ptr, None, [])?;
                    let exec_lo =
                        builder.bitwise_and(context.global_types.u32, None, src_lo, exec_lo)?;
                    builder.store(exec_lo_ptr, exec_lo, None, [])?;

                    let src_hi_ptr = context.scalar_dest_operand(builder, &src.next())?;
                    let src_hi =
                        builder.load(context.global_types.u32, None, src_hi_ptr, None, [])?;
                    let exec_hi =
                        builder.bitwise_and(context.global_types.u32, None, src_hi, exec_hi)?;
                    builder.store(exec_hi_ptr, exec_hi, None, [])?;

                    let execz = context.execz(builder)?;
                    let execnz = builder.logical_not(context.global_types.bool, None, execz)?;

                    builder.store(context.scc, execnz, None, [])?;
                }
                _ => unimplemented!("{:#?}", instr),
            }
        }
        unknown => unimplemented!("{:#?}", unknown),
    }

    Ok(())
}

fn translate_vop3(
    instr: &VOP3Instruction,
    literal_constant: &Option<u32>,
    builder: &mut Builder,
    context: &ExecutionState,
) -> Result<(), anyhow::Error> {
    let VOP3Instruction {
        op: OpCode::VOP2(op),
        vdst,
        src0:
            TransformedOperand {
                abs: false,
                neg: false,
                operand: src0,
            },
        src1:
            TransformedOperand {
                abs: false,
                neg: false,
                operand: src1,
            },
        src2,
        clamp: false,
        output_modifier: OutputModifier::None,
    } = instr
    else {
        unimplemented!()
    };

    let src0 = context.load_src(builder, src0, literal_constant)?;
    let src1 = context.load_src(builder, src1, literal_constant)?;

    let dst = context.vgpr_ptr(builder, vdst.register_idx as _)?;

    let result = match op {
        VOP2OpCode::v_add_f32 => {
            let src0 = builder.bitcast(context.global_types.f32, None, src0)?;
            let src1 = builder.bitcast(context.global_types.f32, None, src1)?;

            let result = builder.f_add(context.global_types.f32, None, src0, src1)?;

            builder.bitcast(context.global_types.u32, None, result)?
        }
        VOP2OpCode::v_mul_f32 => {
            let src0 = builder.bitcast(context.global_types.f32, None, src0)?;
            let src1 = builder.bitcast(context.global_types.f32, None, src1)?;

            let result = builder.f_mul(context.global_types.f32, None, src0, src1)?;

            builder.bitcast(context.global_types.u32, None, result)?
        }
        VOP2OpCode::v_mac_f32 => {
            let src0 = builder.bitcast(context.global_types.f32, None, src0)?;
            let src1 = builder.bitcast(context.global_types.f32, None, src1)?;

            let acc = builder.load(context.global_types.u32, None, dst, None, [])?;
            let acc = builder.bitcast(context.global_types.f32, None, acc)?;

            let combined = builder.f_mul(context.global_types.f32, None, src0, src1)?;

            let result = builder.f_add(context.global_types.f32, None, acc, combined)?;

            builder.bitcast(context.global_types.u32, None, result)?
        }
        VOP2OpCode::v_and_b32 => builder.bitwise_and(context.global_types.u32, None, src0, src1)?,
        VOP2OpCode::v_lshlrev_b32 => {
            let mask_constant = builder.constant_bit32(context.global_types.u32, 0b1111);

            let masked =
                builder.bitwise_and(context.global_types.u32, None, src0, mask_constant)?;

            builder.shift_left_logical(context.global_types.u32, None, src1, masked)?
        }
        VOP2OpCode::v_add_co_u32 => {
            let result = builder.i_add(context.global_types.u32, None, src0, src1)?;

            let int_0 = builder.constant_bit32(context.global_types.u32, 0);

            let did_overflow = {
                let src0_positive =
                    builder.s_greater_than_equal(context.global_types.bool, None, src0, int_0)?;
                let src1_positive =
                    builder.s_greater_than_equal(context.global_types.bool, None, src1, int_0)?;
                let operands_positive = builder.logical_and(
                    context.global_types.bool,
                    None,
                    src0_positive,
                    src1_positive,
                )?;

                let src0_neg = builder.s_less_than(context.global_types.bool, None, src0, int_0)?;
                let src1_neg = builder.s_less_than(context.global_types.bool, None, src1, int_0)?;
                let operands_negative =
                    builder.logical_and(context.global_types.bool, None, src0_neg, src1_neg)?;

                let result_neg =
                    builder.s_less_than(context.global_types.bool, None, result, int_0)?;
                let result_positive =
                    builder.s_greater_than(context.global_types.bool, None, result, int_0)?;

                let positive_overflow = builder.logical_and(
                    context.global_types.bool,
                    None,
                    operands_positive,
                    result_neg,
                )?;
                let negative_overflow = builder.logical_and(
                    context.global_types.bool,
                    None,
                    operands_negative,
                    result_positive,
                )?;

                builder.logical_or(
                    context.global_types.bool,
                    None,
                    positive_overflow,
                    negative_overflow,
                )?
            };

            context.set_vcc(builder, did_overflow)?;

            // todo: exec mask read
            result
        }
        VOP2OpCode::v_cvt_pkrtz_f16_f32 => {
            let src0 = builder.bitcast(context.global_types.f32, None, src0)?;
            let src1 = builder.bitcast(context.global_types.f32, None, src1)?;

            let vec_type = builder.type_vector(context.global_types.f32, 2);
            let vec_value = builder.composite_construct(vec_type, None, [src0, src1])?;

            // todo: rounding mode not specified for these operations

            builder.ext_inst(
                context.global_types.u32,
                None,
                context.glslstd450_import,
                GLOp::PackHalf2x16 as _,
                [Operand::IdRef(vec_value)],
            )?
        }
        unknown => {
            unimplemented!("unknown {:#?}", unknown)
        }
    };

    builder.store(dst, result, None, [])?;

    Ok(())
}

fn expect_sgpr(dst: &ScalarDestinationOperand) -> Result<u8, anyhow::Error> {
    Ok(match dst {
        ScalarDestinationOperand::ScalarGPR(idx) => *idx as _,
        unknown => bail!("unexpected {:?}", unknown),
    })
}

#[derive(Debug)]
pub struct ExportAttributes {
    pub compress: bool,
    pub done: bool,
    pub valid_mask: bool,

    pub src0: bool,
    pub src1: bool,
    pub src2: bool,
    pub src3: bool,
}

impl ExportAttributes {
    pub fn size(&self) -> usize {
        [self.src0, self.src1, self.src2, self.src3]
            .iter()
            .filter(|it| **it)
            .count()
    }
}

pub fn analyze_exports(instructions: &[Instruction]) -> Vec<(ExportTarget, ExportAttributes)> {
    let mut result = vec![];

    for instr in instructions {
        match &instr.inner {
            FormattedInstruction::EXP(exp) => result.push((
                exp.target,
                ExportAttributes {
                    compress: exp.compress,
                    done: exp.done,
                    valid_mask: exp.valid_mask,
                    src0: exp.vsrc0.is_some(),
                    src1: exp.vsrc1.is_some(),
                    src2: exp.vsrc2.is_some(),
                    src3: exp.vsrc3.is_some(),
                },
            )),
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gcn::instructions::formats::exp::ExportTarget;
    use rspirv::binary::{Assemble, Disassemble};
    use spirv_tools::val::Validator;

    use crate::module::construct_control_flow_graph;
    use gcn::test_utils::GCNInstructionStream;
    // #[test]
    // fn test() -> Result<(), anyhow::Error> {
    //     let bytes: [u8; 40] = [
    //         0x00, 0x03, 0x82, 0xc0, 0x7f, 0x00, 0x8c, 0xbf, 0x00, 0x20, 0x08, 0xe0, 0x00, 0x04,
    //         0x01, 0x80, 0x04, 0x03, 0x82, 0xc0, 0x7f, 0x00, 0x8c, 0xbf, 0x00, 0x20, 0x04, 0xe0,
    //         0x00, 0x08, 0x01, 0x80, 0x00, 0x00, 0x8c, 0xbf, 0x00, 0x20, 0x80, 0xbe,
    //     ];

    //     let shader = GCNInstructionStream::new(&bytes)?;

    //     insta::assert_snapshot!(shader.displayed());
    //     insta::assert_snapshot!(shader.debug());

    //     let (module, exports) =
    //         translate_vertex_shader(shader.instructions.iter().collect::<Vec<_>>().as_ref())?;
    //     insta::assert_snapshot!(module.disassemble());
    //     insta::assert_debug_snapshot!(exports);

    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     Ok(())
    // }

    // #[test]
    // fn shader() -> Result<(), anyhow::Error> {
    //     let data: [u8; 0x40] = [
    //         0xFF, 0x03, 0xEB, 0xBE, 0x07, 0x00, 0x00, 0x00, 0x81, 0x00, 0x02, 0x36, 0x81, 0x02,
    //         0x02, 0x34, 0xC2, 0x00, 0x00, 0x36, 0xC1, 0x02, 0x02, 0x4A, 0xC1, 0x00, 0x00, 0x4A,
    //         0x01, 0x0B, 0x02, 0x7E, 0x00, 0x0B, 0x00, 0x7E, 0x80, 0x02, 0x04, 0x7E, 0xF2, 0x02,
    //         0x06, 0x7E, 0xCF, 0x08, 0x00, 0xF8, 0x01, 0x00, 0x02, 0x03, 0x0F, 0x02, 0x00, 0xF8,
    //         0x03, 0x03, 0x03, 0x03, 0x00, 0x00, 0x81, 0xBF,
    //     ];

    //     let shader = GCNInstructionStream::new(&data)?;

    //     insta::assert_snapshot!(shader.displayed());
    //     insta::assert_snapshot!(shader.debug());

    //     let (module, exports) =
    //         translate_vertex_shader(shader.instructions.iter().collect::<Vec<_>>().as_ref())?;
    //     insta::assert_snapshot!(module.disassemble());
    //     insta::assert_debug_snapshot!(exports);

    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     Ok(())
    // }

    // #[test]
    // fn compute_shader_1() -> Result<(), anyhow::Error> {
    //     let data: [u8; 0x48] = [
    //         0xFF, 0x03, 0xEB, 0xBE, 0x0B, 0x00, 0x00, 0x00, 0x10, 0x86, 0x00, 0x8F, 0x00, 0x00,
    //         0x00, 0x4A, 0x00, 0x0D, 0x40, 0xC2, 0x7F, 0x00, 0x8C, 0xBF, 0x00, 0x00, 0x88, 0x7D,
    //         0x6A, 0x24, 0x82, 0xBE, 0x01, 0x00, 0x02, 0x36, 0x07, 0x00, 0x88, 0xBF, 0x00, 0x20,
    //         0x00, 0xE0, 0x01, 0x01, 0x01, 0x80, 0x0A, 0x00, 0x88, 0x7D, 0x6A, 0x24, 0x80, 0xBE,
    //         0x70, 0x0F, 0x8C, 0xBF, 0x00, 0x20, 0x10, 0xE0, 0x00, 0x01, 0x02, 0x80, 0x00, 0x00,
    //         0x81, 0xBF,
    //     ];

    //     let shader = GCNInstructionStream::new(&data)?;

    //     insta::assert_snapshot!(shader.displayed());
    //     insta::assert_snapshot!(shader.debug());

    //     let module = translate_compute_shader(
    //         shader.instructions.iter().collect::<Vec<_>>().as_ref(),
    //         64,
    //         1,
    //         1,
    //         16,
    //         true,
    //         false,
    //         false,
    //     )?;
    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     insta::assert_snapshot!(module.disassemble());

    //     Ok(())
    // }

    // #[test]
    // fn vertex_shader_1() -> Result<(), anyhow::Error> {
    //     let data: [u8; 0x9C] = [
    //         0xFF, 0x03, 0xEB, 0xBE, 0x13, 0x00, 0x00, 0x00, 0x00, 0x03, 0x86, 0xC0, 0x04, 0x03,
    //         0x88, 0xC0, 0x08, 0x03, 0x8A, 0xC0, 0x7F, 0x00, 0x8C, 0xBF, 0x00, 0x20, 0x04, 0xE0,
    //         0x00, 0x04, 0x03, 0x80, 0x00, 0x20, 0x0C, 0xE0, 0x00, 0x08, 0x04, 0x80, 0x00, 0x20,
    //         0x04, 0xE0, 0x00, 0x0C, 0x05, 0x80, 0x00, 0x00, 0x8C, 0xBF, 0x00, 0x09, 0x80, 0xC2,
    //         0x04, 0x09, 0x82, 0xC2, 0x08, 0x09, 0x86, 0xC2, 0x0C, 0x09, 0x84, 0xC2, 0x7F, 0x00,
    //         0x8C, 0xBF, 0x03, 0x02, 0x00, 0x7E, 0x07, 0x02, 0x02, 0x7E, 0x0F, 0x02, 0x04, 0x7E,
    //         0x0B, 0x02, 0x06, 0x7E, 0x01, 0x0A, 0x00, 0x3E, 0x05, 0x0A, 0x02, 0x3E, 0x0D, 0x0A,
    //         0x04, 0x3E, 0x09, 0x0A, 0x06, 0x3E, 0x00, 0x08, 0x00, 0x3E, 0x04, 0x08, 0x02, 0x3E,
    //         0x0C, 0x08, 0x04, 0x3E, 0x08, 0x08, 0x06, 0x3E, 0x80, 0x02, 0x08, 0x7E, 0xF2, 0x02,
    //         0x0A, 0x7E, 0xCF, 0x08, 0x00, 0xF8, 0x00, 0x01, 0x02, 0x03, 0x0F, 0x02, 0x00, 0xF8,
    //         0x08, 0x09, 0x0A, 0x0B, 0x1F, 0x02, 0x00, 0xF8, 0x0C, 0x0D, 0x04, 0x05, 0x00, 0x00,
    //         0x81, 0xBF,
    //     ];

    //     let shader = GCNInstructionStream::new(&data)?;

    //     insta::assert_snapshot!(shader.displayed());
    //     insta::assert_snapshot!(shader.debug());

    //     let (module, exports) =
    //         translate_vertex_shader(shader.instructions.iter().collect::<Vec<_>>().as_ref())?;

    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     insta::assert_snapshot!(module.disassemble());
    //     insta::assert_debug_snapshot!(exports);

    //     Ok(())
    // }

    // #[test]
    // fn pixel_shader_1() -> Result<(), anyhow::Error> {
    //     let data: [u8; 0x6C] = [
    //         0xFF, 0x03, 0xEB, 0xBE, 0x0F, 0x00, 0x00, 0x00, 0x7E, 0x04, 0x80, 0xBE, 0x7E, 0x0A,
    //         0xFE, 0xBE, 0x10, 0x03, 0xFC, 0xBE, 0x00, 0x04, 0x08, 0xC8, 0x01, 0x04, 0x09, 0xC8,
    //         0x00, 0x05, 0x0C, 0xC8, 0x01, 0x05, 0x0D, 0xC8, 0x00, 0x08, 0x80, 0xF0, 0x02, 0x02,
    //         0x61, 0x00, 0x00, 0x03, 0x0C, 0xC8, 0x01, 0x03, 0x0D, 0xC8, 0x70, 0x0F, 0x8C, 0xBF,
    //         0x02, 0x07, 0x04, 0x10, 0x00, 0x00, 0x0C, 0xC8, 0x01, 0x00, 0x0D, 0xC8, 0x00, 0x01,
    //         0x10, 0xC8, 0x01, 0x01, 0x11, 0xC8, 0x00, 0x02, 0x00, 0xC8, 0x01, 0x02, 0x01, 0xC8,
    //         0x00, 0x04, 0xFE, 0xBE, 0x03, 0x09, 0x02, 0x5E, 0x00, 0x05, 0x00, 0x5E, 0x0F, 0x1C,
    //         0x00, 0xF8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xBF,
    //     ];

    //     let shader = GCNInstructionStream::new(&data)?;

    //     insta::assert_snapshot!(shader.displayed());
    //     insta::assert_snapshot!(shader.debug());

    //     let (image_sample_instances, image_imports) = {
    //         let texture_buffer_resource = TextureBufferResource {
    //             base_addr_256: 0,
    //             mtype_l2: 0,
    //             min_lod: 0,
    //             dfmt: SurfaceFormat::Format8_8_8_8,
    //             nfmt: TextureChannelType::UNorm,
    //             mtype: 0,
    //             width: 1919,
    //             height: 1079,
    //             perf_mod: 0,
    //             interlaced: false,
    //             dst_sel_x: DestinationChannelSelect::Zero,
    //             dst_sel_y: DestinationChannelSelect::Zero,
    //             dst_sel_z: DestinationChannelSelect::Zero,
    //             dst_sel_w: DestinationChannelSelect::Zero,
    //             base_level: 0,
    //             last_level: 0,
    //             tiling_idx: TileMode::Depth2dThin64,
    //             pow2pad: false,
    //             mtype_2: false,
    //             texture_type: TextureType::Type1d,
    //             depth: 0,
    //             pitch: 0,
    //             base_array: 0,
    //             last_array: 0,
    //             min_lod_warn: 0,
    //             counter_bank_id: 0,
    //             lod_hdw_cnt_en: false,
    //         };

    //         let sampler = SamplerResource {
    //             clamp_x: 0,
    //             clamp_y: 0,
    //             clamp_z: 0,
    //             max_aniso_ratio: 0,
    //             depth_compare_func: 0,
    //             force_unorm_coords: false,
    //             aniso_threshold: 0,
    //             mc_coord_trunc: false,
    //             force_degamma: false,
    //             aniso_bias: 0,
    //             trunc_coord: false,
    //             disable_cube_wrap: false,
    //             filter_mode: 0,
    //             min_lod: LodFixed { int: 0, frac: 0 },
    //             max_lod: LodFixed { int: 0, frac: 0 },
    //             perf_mip: 0,
    //             perf_z: 0,
    //             lod_bias: 0,
    //             lod_bias_sec: 0,
    //             xy_mag_filter: 0,
    //             xy_min_filter: 0,
    //             z_filter: 0,
    //             mip_filter: 0,
    //             border_color_ptr: 0,
    //             border_color_type: 0,
    //         };

    //         let mut image_sample_instances = BTreeMap::new();

    //         let image_imports = vec![(texture_buffer_resource.clone(), sampler.clone())];
    //         image_sample_instances.insert(0x24, (texture_buffer_resource, sampler));

    //         (image_sample_instances, image_imports)
    //     };

    //     let module = translate_pixel_shader(
    //         shader.instructions.iter().collect::<Vec<_>>().as_ref(),
    //         &[
    //             (
    //                 ExportTarget::Parameter(0),
    //                 ExportAttributes {
    //                     compress: false,
    //                     done: false,
    //                     valid_mask: false,
    //                     src0: true,
    //                     src1: true,
    //                     src2: true,
    //                     src3: true,
    //                 },
    //             ),
    //             (
    //                 ExportTarget::Parameter(1),
    //                 ExportAttributes {
    //                     compress: false,
    //                     done: false,
    //                     valid_mask: false,
    //                     src0: true,
    //                     src1: true,
    //                     src2: true,
    //                     src3: true,
    //                 },
    //             ),
    //         ],
    //         image_imports.as_slice(),
    //         image_sample_instances,
    //     )?;

    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     insta::assert_snapshot!(module.disassemble());

    //     Ok(())
    // }

    // #[test]
    // fn pixel_shader_2() -> Result<(), anyhow::Error> {
    //     let data: [u8; 0x78] = [
    //         0xFF, 0x03, 0xEB, 0xBE, 0x10, 0x00, 0x00, 0x00, 0x7E, 0x04, 0x80, 0xBE, 0x7E, 0x0A,
    //         0xFE, 0xBE, 0x10, 0x03, 0xFC, 0xBE, 0x00, 0x04, 0x08, 0xC8, 0x01, 0x04, 0x09, 0xC8,
    //         0x00, 0x05, 0x0C, 0xC8, 0x01, 0x05, 0x0D, 0xC8, 0x00, 0x0F, 0x80, 0xF0, 0x02, 0x02,
    //         0x61, 0x00, 0x00, 0x00, 0x18, 0xC8, 0x01, 0x00, 0x19, 0xC8, 0x00, 0x01, 0x1C, 0xC8,
    //         0x01, 0x01, 0x1D, 0xC8, 0x00, 0x02, 0x20, 0xC8, 0x01, 0x02, 0x21, 0xC8, 0x00, 0x03,
    //         0x00, 0xC8, 0x01, 0x03, 0x01, 0xC8, 0x70, 0x0F, 0x8C, 0xBF, 0x02, 0x0D, 0x02, 0x10,
    //         0x03, 0x0F, 0x04, 0x10, 0x04, 0x11, 0x06, 0x10, 0x05, 0x01, 0x00, 0x10, 0x00, 0x04,
    //         0xFE, 0xBE, 0x01, 0x05, 0x02, 0x5E, 0x03, 0x01, 0x00, 0x5E, 0x0F, 0x1C, 0x00, 0xF8,
    //         0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xBF,
    //     ];

    //     let shader = GCNInstructionStream::new(&data)?;

    //     insta::assert_snapshot!(shader.displayed());
    //     insta::assert_snapshot!(shader.debug());

    //     let module = translate_pixel_shader(
    //         shader.instructions.iter().collect::<Vec<_>>().as_ref(),
    //         &[(
    //             ExportTarget::Parameter(0),
    //             ExportAttributes {
    //                 compress: false,
    //                 done: false,
    //                 valid_mask: false,
    //                 src0: true,
    //                 src1: true,
    //                 src2: true,
    //                 src3: true,
    //             },
    //         )],
    //         &[],
    //         BTreeMap::new(),
    //     )?;

    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     insta::assert_snapshot!(module.disassemble());

    //     Ok(())
    // }

    // #[test]
    // fn test_embedded_shader() -> Result<(), anyhow::Error> {
    //     let data = &include_bytes!(
    //         "../../../../../gcn/test_data/captured/embedded/full_screen.vert.sb"
    //     )[..];

    //     let shader = GCNInstructionStream::new(&data)?;

    //     let (module, _) = translate_vertex_shader(&shader.instructions.iter().collect::<Vec<_>>())?;

    //     let validator = spirv_tools::val::create(None);
    //     validator.validate(&module.assemble(), None)?;

    //     Ok(())
    // }

    #[test]
    fn control_flow_blocks() -> Result<(), anyhow::Error> {
        let data = &include_bytes!(
            "../../../../../gcn/test_data/captured/CUSA02394/6b46d72505c15fb8537386fa77f1471de5b58ace7a4cb2ba5b59be6166680515.compute.sb"
        )[..];

        let shader = GCNInstructionStream::new(&data)?;

        let graph =
            construct_control_flow_graph(shader.instructions.iter().collect::<Vec<_>>().as_ref());

        insta::assert_debug_snapshot!(graph);

        Ok(())
    }
}
