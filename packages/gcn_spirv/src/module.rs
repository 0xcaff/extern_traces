use crate::execution_state::{ExecutionState, GlobalTypes};
use crate::execution_state_input::{ExecutionStateInput, PIXEL_SHADER_DESCRIPTOR_SET_IDX};
use crate::r#loop::make_loop;
use crate::{
    analyze_exports, translate_instruction, wrap_exec_condition, ExportAttributes,
    ExternStorageInfo,
};
use bits::FromBits;
use gcn::instructions::control_flow::{BranchTarget, ControlFlowInformation};
use gcn::instructions::formats::exp::ExportTarget;
use gcn::instructions::formats::FormattedInstruction;
use gcn::instructions::ops::{SMEMOpCode, SOPPOpCode};
use gcn::instructions::Instruction;
use gcn::resources::{SamplerResource, TextureBufferResource, VertexBufferResource};
use rspirv::dr::{Builder, Operand};
use rspirv::spirv;
use rspirv::spirv::{
    AddressingModel, BuiltIn, Capability, Decoration, Dim, ExecutionMode, ExecutionModel,
    FunctionControl, ImageFormat, MemoryModel, SelectionControl, StorageClass,
};
use std::collections::{BTreeMap, HashMap, HashSet};

struct U32Vec3 {
    inner: spirv::Word,
}

impl U32Vec3 {
    pub fn field(&self, builder: &mut Builder, idx: u32) -> Result<spirv::Word, anyhow::Error> {
        let u32 = builder.type_int(32, 0);

        let const_index = builder.constant_bit32(u32, idx);
        let ptr_type = builder.type_pointer(None, StorageClass::Input, u32);
        let local_ptr = builder.access_chain(ptr_type, None, self.inner, [const_index])?;

        let local_value = builder.load(u32, None, local_ptr, None, [])?;

        Ok(local_value)
    }
}

#[derive(Debug)]
pub struct ControlFlowBlock {
    start_instruction_idx: usize,
    end_instruction_idx: usize,
    last_instruction_branch: bool,
}

pub fn construct_control_flow_graph(instructions: &[Instruction]) -> Vec<ControlFlowBlock> {
    let instructions_with_control_flow = instructions
        .iter()
        .map(|it| (it.control_flow_information(), it))
        .collect::<Vec<_>>();

    let jump_target_addresses = instructions_with_control_flow
        .iter()
        .flat_map(|(control_flow_information, _)| {
            let branch_target = control_flow_information.branch_target.as_ref()?;
            match branch_target {
                BranchTarget::Indirect(_) => {
                    // resolving indirect branches unimplemented
                    unimplemented!()
                }
                BranchTarget::Direct(it) => Some(*it),
            }
        })
        .collect::<HashSet<_>>();

    let mut control_flow_blocks: Vec<ControlFlowBlock> = vec![];

    let mut last_instruction_branch = false;

    for (idx, instruction) in instructions.iter().enumerate() {
        if last_instruction_branch
            || jump_target_addresses.contains(&instruction.program_counter)
            || idx == 0
        {
            control_flow_blocks.push(ControlFlowBlock {
                start_instruction_idx: idx,
                end_instruction_idx: idx,
                last_instruction_branch: false,
            });
        }

        let block = control_flow_blocks.last_mut().unwrap();
        block.end_instruction_idx = idx;

        last_instruction_branch = matches!(
            instructions_with_control_flow[idx].0,
            ControlFlowInformation {
                has_next_instruction: false,
                ..
            } | ControlFlowInformation {
                branch_target: Some(..),
                ..
            }
        );

        block.last_instruction_branch = last_instruction_branch;
    }

    control_flow_blocks
}

pub fn translate_compute_shader<'a>(
    instructions: &[Instruction],
    local_x: u16,
    local_y: u16,
    local_z: u16,
    user_data_slots_count: u64,
    thread_group_id_x_enable: bool,
    thread_group_id_y_enable: bool,
    thread_group_id_z_enable: bool,
    buffer_usage_instances: &BTreeMap<u64, VertexBufferResource>,
    buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
) -> Result<rspirv::dr::Module, anyhow::Error> {
    let mut builder = Builder::new();

    builder.capability(Capability::Shader);
    builder.set_version(1, 4);

    builder.memory_model(AddressingModel::Logical, MemoryModel::GLSL450);

    let global_types = GlobalTypes::new(&mut builder);
    let inputs = ExecutionStateInput::compute(&mut builder, &global_types, buffer_resources);

    let void = builder.type_void();
    let voidf = builder.type_function(void, []);
    let main = builder.begin_function(void, None, FunctionControl::NONE, voidf)?;

    let u32x3 = builder.type_vector(global_types.u32, 3);

    let local_invocation_id = {
        let ptr_type = builder.type_pointer(None, StorageClass::Input, u32x3);

        let ptr = builder.variable(ptr_type, None, StorageClass::Input, None);
        builder.name(ptr, "local_invocation_id_ptr");

        builder.decorate(
            ptr,
            Decoration::BuiltIn,
            [Operand::BuiltIn(BuiltIn::LocalInvocationId)],
        );

        U32Vec3 { inner: ptr }
    };

    let work_group_id = {
        let ptr_type = builder.type_pointer(None, StorageClass::Input, u32x3);

        let ptr = builder.variable(ptr_type, None, StorageClass::Input, None);
        builder.name(ptr, "work_group_id_ptr");

        builder.decorate(
            ptr,
            Decoration::BuiltIn,
            [Operand::BuiltIn(BuiltIn::WorkgroupId)],
        );

        U32Vec3 { inner: ptr }
    };

    builder.entry_point(ExecutionModel::GLCompute, main, "main", {
        let mut interface = vec![];

        interface.extend(inputs.interface());

        interface.push(local_invocation_id.inner);
        interface.push(work_group_id.inner);

        interface
    });

    {
        let local_x = builder.constant_bit32(global_types.u32, local_x as _);
        let local_y = builder.constant_bit32(global_types.u32, local_y as _);
        let local_z = builder.constant_bit32(global_types.u32, local_z as _);

        builder.execution_mode(main, ExecutionMode::LocalSize, [local_x, local_y, local_z])
    }

    builder.begin_block(None)?;

    let branching_label = {
        let ptr_type = builder.type_pointer(None, StorageClass::Function, global_types.u32);

        let var = builder.variable(ptr_type, None, StorageClass::Function, None);

        var
    };

    let iters_limit = {
        let ptr_type = builder.type_pointer(None, StorageClass::Function, global_types.u32);

        let var = builder.variable(ptr_type, None, StorageClass::Function, None);

        var
    };

    let thread_id = builder.id();

    let translation_context = ExecutionState::new(&mut builder, &inputs, thread_id, global_types)?;

    {
        let acc = local_invocation_id.field(&mut builder, 0)?;

        let dim_x = builder.constant_bit32(translation_context.global_types.u32, local_x as _);
        let field_y = local_invocation_id.field(&mut builder, 1)?;
        let elem = builder.i_mul(translation_context.global_types.u32, None, dim_x, field_y)?;
        let acc = builder.i_add(translation_context.global_types.u32, None, elem, acc)?;

        let dim_xy = builder.constant_bit32(
            translation_context.global_types.u32,
            (local_y * local_x) as _,
        );
        let field_z = local_invocation_id.field(&mut builder, 2)?;
        let elem = builder.i_mul(translation_context.global_types.u32, None, dim_xy, field_z)?;
        builder.i_add(
            translation_context.global_types.u32,
            Some(thread_id),
            elem,
            acc,
        )?;
    };

    let file_name = builder.string("compute_shader.sb");
    let exports = BTreeMap::new();

    if local_x != 1 {
        let vgpr_ptr = translation_context.vgpr_ptr(&mut builder, 0)?;
        let invocation_value = local_invocation_id.field(&mut builder, 0)?;
        builder.store(vgpr_ptr, invocation_value, None, [])?;
    }

    if local_y != 1 {
        let vgpr_ptr = translation_context.vgpr_ptr(&mut builder, 1)?;
        let invocation_value = local_invocation_id.field(&mut builder, 1)?;
        builder.store(vgpr_ptr, invocation_value, None, [])?;
    }

    if local_z != 1 {
        let vgpr_ptr = translation_context.vgpr_ptr(&mut builder, 2)?;
        let invocation_value = local_invocation_id.field(&mut builder, 2)?;
        builder.store(vgpr_ptr, invocation_value, None, [])?;
    }

    if thread_group_id_x_enable {
        let offset = 0;

        let sgpr_ptr =
            translation_context.sgpr_ptr(&mut builder, user_data_slots_count as u32 + offset)?;
        let value = work_group_id.field(&mut builder, offset)?;
        builder.store(sgpr_ptr, value, None, [])?;
    }

    if thread_group_id_y_enable {
        let offset = 1;

        let sgpr_ptr =
            translation_context.sgpr_ptr(&mut builder, user_data_slots_count as u32 + offset)?;
        let value = work_group_id.field(&mut builder, offset)?;
        builder.store(sgpr_ptr, value, None, [])?;
    }

    if thread_group_id_z_enable {
        let offset = 2;

        let sgpr_ptr =
            translation_context.sgpr_ptr(&mut builder, user_data_slots_count as u32 + offset)?;
        let value = work_group_id.field(&mut builder, offset)?;
        builder.store(sgpr_ptr, value, None, [])?;
    }

    let imports = BTreeMap::new();
    let image_imports = HashMap::new();
    let image_sample_instances = BTreeMap::new();

    {
        let const_0 = builder.constant_bit32(translation_context.global_types.u32, 0);
        builder.store(branching_label, const_0, None, [])?;
    };

    {
        let const_0 = builder.constant_bit32(translation_context.global_types.u32, 0);
        builder.store(iters_limit, const_0, None, [])?;
    };

    {
        let const_1 = builder.constant_bit32(translation_context.global_types.u32, 1);
        let branch_loop_iters_limit =
            builder.constant_bit32(translation_context.global_types.u32, 1024);

        make_loop(
            &mut builder,
            |builder| {
                let value = builder.load(
                    translation_context.global_types.u32,
                    None,
                    iters_limit,
                    None,
                    [],
                )?;

                Ok((
                    builder.u_less_than(
                        translation_context.global_types.bool,
                        None,
                        value,
                        branch_loop_iters_limit,
                    )?,
                    value,
                ))
            },
            |mut builder, _| {
                let switch_merge = builder.id();

                let control_flow_blocks = construct_control_flow_graph(instructions);
                let labeled_blocks = control_flow_blocks
                    .into_iter()
                    .enumerate()
                    .map(|(idx, it)| (idx, builder.id(), it))
                    .collect::<Vec<_>>();

                let branching_label_value = builder.load(
                    translation_context.global_types.u32,
                    None,
                    branching_label,
                    None,
                    [],
                )?;

                builder.selection_merge(switch_merge, SelectionControl::NONE)?;

                builder.switch(
                    branching_label_value,
                    switch_merge,
                    labeled_blocks.iter().map(|(idx, block_label, it)| {
                        (Operand::LiteralBit32(*idx as _), *block_label)
                    }),
                )?;

                for (idx, block_label, it) in &labeled_blocks {
                    builder.begin_block(Some(*block_label))?;

                    let last_idx =
                        it.end_instruction_idx - if it.last_instruction_branch { 1 } else { 0 };
                    for instr_idx in it.start_instruction_idx..=last_idx {
                        let instruction = &instructions[instr_idx];

                        builder.line(file_name, instr_idx as _, 0);
                        wrap_exec_condition(
                            builder,
                            &translation_context,
                            &instruction,
                            |builder| {
                                translate_instruction(
                                    builder,
                                    &imports,
                                    &image_sample_instances,
                                    &image_imports,
                                    buffer_usage_instances,
                                    &exports,
                                    &instruction,
                                    &translation_context,
                                )?;

                                Ok(())
                            },
                        )?;
                    }

                    if it.last_instruction_branch {
                        let instruction = &instructions[it.end_instruction_idx];

                        match &instruction.inner {
                            FormattedInstruction::SOPP(instr) => match instr.op {
                                SOPPOpCode::s_cbranch_execz
                                | SOPPOpCode::s_cbranch_execnz
                                | SOPPOpCode::s_cbranch_scc0
                                | SOPPOpCode::s_cbranch_scc1
                                | SOPPOpCode::s_cbranch_vccz
                                | SOPPOpCode::s_cbranch_vccnz => {
                                    let (condition, negate) = match instr.op {
                                        SOPPOpCode::s_cbranch_execz => {
                                            (translation_context.execz(builder)?, false)
                                        }
                                        SOPPOpCode::s_cbranch_execnz => {
                                            (translation_context.execz(builder)?, true)
                                        }
                                        SOPPOpCode::s_cbranch_scc0 => {
                                            (translation_context.scc(builder)?, false)
                                        }
                                        SOPPOpCode::s_cbranch_scc1 => {
                                            (translation_context.scc(builder)?, true)
                                        }
                                        SOPPOpCode::s_cbranch_vccz => {
                                            (translation_context.vccz(builder)?, false)
                                        }
                                        SOPPOpCode::s_cbranch_vccnz => {
                                            (translation_context.vccz(builder)?, true)
                                        }
                                        _ => unimplemented!(),
                                    };

                                    let value = if negate {
                                        builder.logical_not(
                                            translation_context.global_types.bool,
                                            None,
                                            condition,
                                        )?
                                    } else {
                                        condition
                                    };

                                    let end_label = builder.id();
                                    let true_label = builder.id();
                                    let false_label = builder.id();

                                    builder.selection_merge(end_label, SelectionControl::NONE)?;
                                    builder.branch_conditional(
                                        value,
                                        true_label,
                                        false_label,
                                        [],
                                    )?;

                                    {
                                        builder.begin_block(Some(true_label))?;

                                        let target_address = ((instr.simm16 as u64) << 2)
                                            + 4
                                            + instruction.program_counter;

                                        let (idx, _, _) = labeled_blocks
                                            .iter()
                                            .find(|(idx, _, block)| {
                                                let instr =
                                                    &instructions[block.start_instruction_idx];
                                                instr.program_counter == target_address
                                            })
                                            .unwrap();

                                        let label_value = builder.constant_bit32(
                                            translation_context.global_types.u32,
                                            *idx as u32,
                                        );
                                        builder.store(branching_label, label_value, None, [])?;
                                        builder.branch(end_label)?;
                                    }

                                    {
                                        builder.begin_block(Some(false_label))?;

                                        let next_idx = (idx + 1) as u32;
                                        let next_idx_const = builder.constant_bit32(
                                            translation_context.global_types.u32,
                                            next_idx,
                                        );

                                        builder.store(branching_label, next_idx_const, None, [])?;
                                        builder.branch(end_label)?;
                                    }

                                    builder.begin_block(Some(end_label))?;
                                }
                                SOPPOpCode::s_endpgm => {
                                    // break out of the loop
                                    builder.store(
                                        iters_limit,
                                        branch_loop_iters_limit,
                                        None,
                                        [],
                                    )?;
                                }
                                _ => unimplemented!(),
                            },
                            _ => unimplemented!(),
                        }
                    } else {
                        let next_idx = (idx + 1) as u32;
                        let next_idx_const =
                            builder.constant_bit32(translation_context.global_types.u32, next_idx);

                        builder.store(branching_label, next_idx_const, None, [])?;
                    }

                    builder.branch(switch_merge)?;
                }

                builder.begin_block(Some(switch_merge))?;

                Ok(())
            },
            |builder, value| {
                let incremented =
                    builder.i_add(translation_context.global_types.u32, None, *value, const_1)?;

                builder.store(iters_limit, incremented, None, [])?;

                Ok(())
            },
        )?;
    }

    builder.ret()?;
    builder.end_function()?;

    Ok(builder.module())
}

pub fn translate_vertex_shader<'a>(
    instructions: &[Instruction],
    buffer_usage_instances: &BTreeMap<u64, VertexBufferResource>,
    buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
) -> Result<(rspirv::dr::Module, Vec<(ExportTarget, ExportAttributes)>), anyhow::Error> {
    let mut builder = Builder::new();
    builder.capability(Capability::Shader);
    builder.set_version(1, 4);

    builder.memory_model(AddressingModel::Logical, MemoryModel::GLSL450);

    let global_types = GlobalTypes::new(&mut builder);

    let analyzed_exports = analyze_exports(instructions);
    let exports = analyzed_exports
        .iter()
        .map(|(target, attrs)| {
            let size = attrs.size();
            let value_type = builder.type_vector(global_types.f32, size as _);

            let vector_type_pointer = builder.type_pointer(None, StorageClass::Output, value_type);

            let value_pointer =
                builder.variable(vector_type_pointer, None, StorageClass::Output, None);

            match target {
                ExportTarget::Position(0) => {
                    if size != 4 {
                        unimplemented!("invalid size {}", size)
                    }

                    builder.decorate(
                        value_pointer,
                        Decoration::BuiltIn,
                        [Operand::BuiltIn(BuiltIn::Position)],
                    );
                }
                ExportTarget::Parameter(idx) => {
                    builder.decorate(
                        value_pointer,
                        Decoration::Location,
                        [Operand::LiteralBit32(*idx as _)],
                    );
                }
                other => unimplemented!("{:?}", other),
            };

            (
                target.clone(),
                ExternStorageInfo {
                    value_pointer,
                    value_type,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let vertex_index_ptr = {
        let vertex_index_ptr_type =
            builder.type_pointer(None, StorageClass::Input, global_types.u32);

        let vertex_index_ptr =
            builder.variable(vertex_index_ptr_type, None, StorageClass::Input, None);
        builder.name(vertex_index_ptr, "vertex_index_ptr");
        builder.decorate(
            vertex_index_ptr,
            Decoration::BuiltIn,
            [Operand::BuiltIn(BuiltIn::VertexIndex)],
        );

        vertex_index_ptr
    };

    let inputs = ExecutionStateInput::vertex(&mut builder, &global_types, buffer_resources);

    let void = builder.type_void();
    let voidf = builder.type_function(void, []);
    let main = builder.begin_function(void, None, FunctionControl::NONE, voidf)?;

    builder.entry_point(ExecutionModel::Vertex, main, "main", {
        let mut interface = exports
            .values()
            .map(|it| it.value_pointer)
            .collect::<Vec<_>>();

        interface.push(vertex_index_ptr);
        interface.extend(inputs.interface());

        interface
    });

    builder.begin_block(None)?;

    let thread_id = builder.constant_bit32(global_types.u32, 0);
    let translation_context = ExecutionState::new(&mut builder, &inputs, thread_id, global_types)?;
    let file_name = builder.string("vertex_shader.sb");

    let vertex_index = builder.load(
        translation_context.global_types.u32,
        None,
        vertex_index_ptr,
        None,
        [],
    )?;
    builder.name(vertex_index, "vertex_index");

    let vgpr0_ptr = translation_context.vgpr_ptr(&mut builder, 0)?;
    builder.store(vgpr0_ptr, vertex_index, None, [])?;

    let imports = BTreeMap::new();
    let image_sample_instances = BTreeMap::new();
    let image_imports = HashMap::new();
    for (idx, instruction) in instructions.iter().enumerate() {
        builder.line(file_name, idx as _, 0);
        translate_instruction(
            &mut builder,
            &imports,
            &image_sample_instances,
            &image_imports,
            buffer_usage_instances,
            &exports,
            instruction,
            &translation_context,
        )?;
    }

    builder.ret()?;
    builder.end_function()?;

    let module = builder.module();

    Ok((module, analyzed_exports))
}

pub fn translate_pixel_shader<'a>(
    instructions: &[Instruction],
    vertex_shader_exports: &[(ExportTarget, ExportAttributes)],
    image_imports: impl Iterator<
        Item = (
            u32,
            &'a (([u32; 8], TextureBufferResource), SamplerResource),
        ),
    >,
    image_sample_instances: BTreeMap<u64, (([u32; 8], TextureBufferResource), SamplerResource)>,
    buffer_usage_instances: &BTreeMap<u64, VertexBufferResource>,
    buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
) -> Result<rspirv::dr::Module, anyhow::Error> {
    let mut builder = Builder::new();

    builder.capability(Capability::Shader);
    builder.set_version(1, 4);

    builder.memory_model(AddressingModel::Logical, MemoryModel::GLSL450);

    let global_types = GlobalTypes::new(&mut builder);

    let imports = {
        vertex_shader_exports
            .iter()
            .flat_map(|(target, attrs)| {
                let ExportTarget::Parameter(parameter) = target else {
                    return None;
                };

                let size = attrs.size();
                let value_type = builder.type_vector(global_types.f32, size as _);

                let vector_type_pointer =
                    builder.type_pointer(None, StorageClass::Input, value_type);

                let value_pointer =
                    builder.variable(vector_type_pointer, None, StorageClass::Input, None);

                builder.decorate(
                    value_pointer,
                    Decoration::Location,
                    [Operand::LiteralBit32(*parameter as _)],
                );

                Some((
                    *parameter,
                    ExternStorageInfo {
                        value_type,
                        value_pointer,
                    },
                ))
            })
            .collect::<BTreeMap<_, _>>()
    };

    let image_imports = image_imports
        .map(|(idx, image_import)| {
            let image_type = builder.type_image(
                global_types.f32,
                Dim::Dim2D,
                0,
                0,
                0,
                1,
                ImageFormat::Unknown,
                None,
            );

            let sampled_image_type = builder.type_sampled_image(image_type);

            let sampled_image_pointer_type =
                builder.type_pointer(None, StorageClass::UniformConstant, sampled_image_type);

            let image = builder.variable(
                sampled_image_pointer_type,
                None,
                StorageClass::UniformConstant,
                None,
            );

            builder.decorate(
                image,
                Decoration::DescriptorSet,
                [Operand::LiteralBit32(PIXEL_SHADER_DESCRIPTOR_SET_IDX)],
            );

            builder.decorate(image, Decoration::Binding, [Operand::LiteralBit32(idx)]);

            (
                image_import.clone(),
                ExternStorageInfo {
                    value_pointer: image,
                    value_type: sampled_image_type,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let analyzed_exports = analyze_exports(instructions);
    let exports = analyzed_exports
        .iter()
        .map(|(target, attrs)| {
            let ExportTarget::RenderTarget(0) = target else {
                unimplemented!("{:?}", target)
            };

            let render_target_idx = 0;

            let value = {
                let value_type = builder.type_vector(global_types.f32, 4);

                let value_pointer_type =
                    builder.type_pointer(None, StorageClass::Output, value_type);

                let value_pointer =
                    builder.variable(value_pointer_type, None, StorageClass::Output, None);

                builder.decorate(
                    value_pointer,
                    Decoration::Location,
                    [Operand::LiteralBit32(render_target_idx as _)],
                );

                ExternStorageInfo {
                    value_pointer,
                    value_type,
                }
            };

            (ExportTarget::RenderTarget(render_target_idx), value)
        })
        .collect::<BTreeMap<_, _>>();

    let inputs = ExecutionStateInput::pixel(&mut builder, &global_types, buffer_resources);

    let void = builder.type_void();
    let voidf = builder.type_function(void, []);
    let main = builder.begin_function(void, None, FunctionControl::NONE, voidf)?;

    builder.entry_point(ExecutionModel::Fragment, main, "main", {
        let mut interface = Vec::new();

        interface.extend(imports.iter().map(|(k, v)| v.value_pointer));
        interface.extend(inputs.interface());
        interface.extend(image_imports.values().map(|it| it.value_pointer));
        interface.extend(exports.iter().map(|(k, v)| v.value_pointer));

        interface
    });
    // todo: not sure whether this is right
    builder.execution_mode(main, ExecutionMode::OriginUpperLeft, []);

    builder.begin_block(None)?;

    let thread_id = builder.constant_bit32(global_types.u32, 0);
    let translation_context = ExecutionState::new(&mut builder, &inputs, thread_id, global_types)?;
    let file_name = builder.string("fragment_shader.sb");

    for (idx, instruction) in instructions.iter().enumerate() {
        builder.line(file_name, idx as _, 0);
        translate_instruction(
            &mut builder,
            &imports,
            &image_sample_instances,
            &image_imports,
            &buffer_usage_instances,
            &exports,
            instruction,
            &translation_context,
        )?;
    }

    builder.ret()?;
    builder.end_function()?;

    Ok(builder.module())
}
