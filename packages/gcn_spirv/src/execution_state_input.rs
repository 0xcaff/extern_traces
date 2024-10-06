use rspirv::dr::{Builder, Operand};
use rspirv::spirv;
use rspirv::spirv::{Decoration, StorageClass};
use std::collections::{BTreeMap, HashMap};
use std::iter;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::descriptor_set::layout::{DescriptorSetLayoutBinding, DescriptorType};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use vulkano::shader::ShaderStages;

use gcn::resources::VertexBufferResource;
use pm4::{ComputeUserData, PixelShaderUserData, VertexShaderUserData};

use crate::execution_state::GlobalTypes;

pub const SGPR_COUNT: u32 = 104;
pub const SGPR_COUNT_U: usize = SGPR_COUNT as usize;

pub struct ExecutionStateInput {
    pub(super) uniform_input_value: spirv::Word,

    pub(super) sgpr_array_type: spirv::Word,
    pub(super) sgpr_count_id: spirv::Word,

    pub(super) vgpr_array_type: spirv::Word,
    pub(super) vgpr_count_id: spirv::Word,

    pub(super) memory_field_type: spirv::Word,
    pub(super) entries: HashMap<VertexBufferResource, spirv::Word>,
}

impl ExecutionStateInput {
    pub fn vertex<'a>(
        builder: &mut Builder,
        global_types: &GlobalTypes,
        buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
    ) -> ExecutionStateInput {
        Self::new(
            builder,
            global_types,
            buffer_resources,
            VERTEX_SHADER_DESCRIPTOR_SET_IDX,
        )
    }

    pub fn pixel<'a>(
        builder: &mut Builder,
        global_types: &GlobalTypes,
        buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
    ) -> ExecutionStateInput {
        Self::new(
            builder,
            global_types,
            buffer_resources,
            PIXEL_SHADER_DESCRIPTOR_SET_IDX,
        )
    }

    pub fn compute<'a>(
        builder: &mut Builder,
        global_types: &GlobalTypes,
        buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
    ) -> ExecutionStateInput {
        Self::new(
            builder,
            global_types,
            buffer_resources,
            COMPUTE_SHADER_DESCRIPTOR_SET_IDX,
        )
    }

    fn new<'a>(
        builder: &mut Builder,
        global_types: &GlobalTypes,
        buffer_resources: impl Iterator<Item = (u32, &'a VertexBufferResource)>,
        set_idx: u32,
    ) -> ExecutionStateInput {
        const VGPR_COUNT: u32 = 253;
        let vgpr_count_id = builder.constant_bit32(global_types.u32, VGPR_COUNT);

        let vgpr_array_type = {
            let vgpr_array_type = builder.type_array(global_types.u32, vgpr_count_id);
            builder.name(vgpr_array_type, format!("u32[{}]", VGPR_COUNT));

            vgpr_array_type
        };

        let sgpr_count_id = builder.constant_bit32(global_types.u32, SGPR_COUNT);

        let sgpr_array_type = {
            let sgpr_array_type = builder.type_array(global_types.u32, sgpr_count_id);
            builder.name(sgpr_array_type, format!("u32[{}]", SGPR_COUNT));
            builder.decorate(
                sgpr_array_type,
                Decoration::ArrayStride,
                [Operand::LiteralBit32(4)],
            );

            sgpr_array_type
        };

        let memory_type = {
            let typ = builder.type_runtime_array(global_types.u32);
            builder.name(typ, "u32[]");
            builder.decorate(typ, Decoration::ArrayStride, [Operand::LiteralBit32(4)]);

            typ
        };

        let memory_field_type =
            builder.type_pointer(None, StorageClass::StorageBuffer, memory_type);

        let memory_input_type = {
            let struct_type = builder.type_struct([memory_type]);

            builder.name(struct_type, "MemoryInput");
            builder.decorate(struct_type, Decoration::Block, []);

            builder.member_name(struct_type, 0, "memory");
            builder.member_decorate(
                struct_type,
                0,
                Decoration::Offset,
                [Operand::LiteralBit32(0)],
            );

            struct_type
        };

        let entries = buffer_resources
            .map(|(idx, it)| {
                let value_type =
                    builder.type_pointer(None, StorageClass::StorageBuffer, memory_input_type);

                let value_pointer =
                    builder.variable(value_type, None, StorageClass::StorageBuffer, None);

                builder.name(value_pointer, &format!("memory_input_{}", idx));
                builder.decorate(
                    value_pointer,
                    Decoration::DescriptorSet,
                    [Operand::LiteralBit32(set_idx)],
                );
                builder.decorate(
                    value_pointer,
                    Decoration::Binding,
                    [Operand::LiteralBit32(idx)],
                );

                (it.clone(), value_pointer)
            })
            .collect::<HashMap<VertexBufferResource, spirv::Word>>();

        let uniform_input_value = {
            let typ = {
                let uniform_input_struct_type = builder.type_struct([sgpr_array_type]);
                builder.name(uniform_input_struct_type, "UniformInput");
                builder.decorate(uniform_input_struct_type, Decoration::Block, []);

                builder.member_name(uniform_input_struct_type, 0, "sgpr");
                builder.member_decorate(uniform_input_struct_type, 0, Decoration::NonWritable, []);
                builder.member_decorate(
                    uniform_input_struct_type,
                    0,
                    Decoration::Offset,
                    [Operand::LiteralBit32(0)],
                );

                uniform_input_struct_type
            };

            let pointer_typ = builder.type_pointer(None, StorageClass::Uniform, typ);

            let uniform_input_struct_value =
                builder.variable(pointer_typ, None, StorageClass::Uniform, None);
            builder.name(uniform_input_struct_value, "uniform_input");

            builder.decorate(
                uniform_input_struct_value,
                Decoration::DescriptorSet,
                [Operand::LiteralBit32(set_idx)],
            );

            builder.decorate(
                uniform_input_struct_value,
                Decoration::Binding,
                [Operand::LiteralBit32(0)],
            );

            uniform_input_struct_value
        };

        Self {
            entries,
            uniform_input_value,
            sgpr_array_type,
            sgpr_count_id,
            vgpr_array_type,
            vgpr_count_id,
            memory_field_type,
        }
    }

    pub fn interface(&self) -> impl IntoIterator<Item = spirv::Word> + '_ {
        iter::once(self.uniform_input_value).chain(self.entries.values().cloned())
    }
}

pub const VERTEX_SHADER_DESCRIPTOR_SET_IDX: u32 = 0;

pub fn vertex_shader_user_data_descriptor(
    allocator: Arc<dyn MemoryAllocator>,
    user_data: &VertexShaderUserData,
) -> Result<WriteDescriptorSet, anyhow::Error> {
    shader_user_data(allocator, &user_data.0)
}

pub const PIXEL_SHADER_DESCRIPTOR_SET_IDX: u32 = 1;

pub fn pixel_shader_user_data_descriptor(
    allocator: Arc<dyn MemoryAllocator>,
    user_data: &PixelShaderUserData,
) -> Result<WriteDescriptorSet, anyhow::Error> {
    shader_user_data(allocator, &user_data.0)
}

pub const COMPUTE_SHADER_DESCRIPTOR_SET_IDX: u32 = 2;

pub fn compute_shader_user_data_descriptor(
    allocator: Arc<dyn MemoryAllocator>,
    user_data: &ComputeUserData,
) -> Result<WriteDescriptorSet, anyhow::Error> {
    shader_user_data(allocator, &user_data.0)
}

fn shader_user_data(
    allocator: Arc<dyn MemoryAllocator>,
    user_data: &BTreeMap<u8, u32>,
) -> Result<WriteDescriptorSet, anyhow::Error> {
    Ok(WriteDescriptorSet::buffer(
        0,
        Buffer::from_data(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            ScalarGPRInput::from_entries(user_data),
        )?,
    ))
}

pub fn shader_user_data_layout_binding(
    stages: ShaderStages,
) -> impl Iterator<Item = (u32, DescriptorSetLayoutBinding)> {
    iter::once((
        0,
        DescriptorSetLayoutBinding {
            stages,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
        },
    ))
}

#[derive(BufferContents)]
#[repr(C)]
struct ScalarGPRInput {
    sgprs: [u32; SGPR_COUNT_U],
}

impl ScalarGPRInput {
    pub fn from_entries(entries: &BTreeMap<u8, u32>) -> ScalarGPRInput {
        let mut sgprs: [u32; SGPR_COUNT_U] = [0; SGPR_COUNT_U];

        for (slot, value) in entries {
            sgprs[(*slot) as usize] = *value;
        }

        ScalarGPRInput { sgprs }
    }
}
