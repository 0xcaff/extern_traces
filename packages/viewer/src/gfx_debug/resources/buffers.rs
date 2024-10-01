use crate::gfx_debug::ctx::{CommandBufferBuilder, GraphicsContext};
use anyhow::format_err;
use std::collections::BTreeMap;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::descriptor_set::layout::{DescriptorSetLayoutBinding, DescriptorType};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::shader::ShaderStages;

struct BufferResourceMemory {
    input_buffer: Subbuffer<[u8]>,
    device_buffer: Subbuffer<[u8]>,
    output_buffer: Subbuffer<[u8]>,
}

impl BufferResourceMemory {
    pub fn new(
        bytes: &[u8],
        graphics_context: &GraphicsContext,
    ) -> Result<BufferResourceMemory, anyhow::Error> {
        let input_buffer = Buffer::new_unsized::<[u8]>(
            graphics_context.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            bytes.len() as _,
        )?;

        {
            let mut local_input_buffer = input_buffer.write()?;
            local_input_buffer.copy_from_slice(bytes);
        }

        let device_buffer = Buffer::new_unsized::<[u8]>(
            graphics_context.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST
                    | BufferUsage::TRANSFER_SRC
                    | BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            bytes.len() as _,
        )?;

        let output_buffer = Buffer::new_unsized::<[u8]>(
            graphics_context.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            bytes.len() as _,
        )?;

        Ok(BufferResourceMemory {
            input_buffer,
            device_buffer,
            output_buffer,
        })
    }

    pub fn stage_input(&self, builder: &mut CommandBufferBuilder) -> Result<(), anyhow::Error> {
        builder.copy_buffer(CopyBufferInfo::buffers(
            self.input_buffer.clone(),
            self.device_buffer.clone(),
        ))?;

        Ok(())
    }

    pub fn stage_output(&self, builder: &mut CommandBufferBuilder) -> Result<(), anyhow::Error> {
        builder.copy_buffer(CopyBufferInfo::buffers(
            self.device_buffer.clone(),
            self.output_buffer.clone(),
        ))?;

        Ok(())
    }
}

pub struct BufferShaderStageResult {
    next_vertex_buffers: BTreeMap<usize, Box<[u8]>>,
}

pub struct BuffersDataContainer<'a> {
    vertex_buffers: BTreeMap<usize, BufferData<'a>>,
}

pub enum BufferData<'a> {
    Ref(&'a [u8]),
    Owned(Box<[u8]>),
}

impl BufferData<'_> {
    pub fn bytes(&self) -> &[u8] {
        match self {
            BufferData::Ref(value) => *value,
            BufferData::Owned(value) => value,
        }
    }
}

pub struct SharedDescriptorSet {
    resources: Vec<(BufferResourceMemory, usize)>,
    binding_offset: u32,
}

impl SharedDescriptorSet {
    pub fn new(
        graphics_context: &GraphicsContext,
        binding_offset: u32,
        buffer_resources_indices: &[usize],
        data: BuffersDataContainer,
    ) -> Result<SharedDescriptorSet, anyhow::Error> {
        let resources = buffer_resources_indices
            .iter()
            .map(|resource_idx| -> Result<_, anyhow::Error> {
                let data = data
                    .vertex_buffers
                    .get(resource_idx)
                    .ok_or_else(|| format_err!("missing data"))?;

                Ok((
                    BufferResourceMemory::new(data.bytes(), graphics_context)?,
                    *resource_idx,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SharedDescriptorSet {
            resources,
            binding_offset,
        })
    }

    pub fn bindings(&self) -> impl IntoIterator<Item = (u32, DescriptorSetLayoutBinding)> + '_ {
        self.resources.iter().enumerate().map(|(idx, (it, _))| {
            (
                idx as u32 + self.binding_offset,
                DescriptorSetLayoutBinding {
                    stages: ShaderStages::all_graphics(),
                    ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer)
                },
            )
        })
    }

    pub fn write_descriptor_set(&self) -> impl IntoIterator<Item = WriteDescriptorSet> + '_ {
        self.resources.iter().enumerate().map(|(idx, (it, _))| {
            WriteDescriptorSet::buffer(idx as u32 + self.binding_offset, it.device_buffer.clone())
        })
    }

    pub fn stage_input(&self, builder: &mut CommandBufferBuilder) -> Result<(), anyhow::Error> {
        for (it, _) in &self.resources {
            it.stage_input(builder)?;
        }

        Ok(())
    }

    pub fn stage_output(&self, builder: &mut CommandBufferBuilder) -> Result<(), anyhow::Error> {
        for (it, _) in &self.resources {
            it.stage_output(builder)?;
        }

        Ok(())
    }

    pub fn flush_output_memory(&self) -> Result<BufferShaderStageResult, anyhow::Error> {
        let mut next_vertex_buffers = BTreeMap::new();

        for (it, resource_idx) in &self.resources {
            let output_memory = it.output_buffer.read()?.to_vec().into_boxed_slice();
            next_vertex_buffers.insert(*resource_idx, output_memory);
        }

        Ok(BufferShaderStageResult {
            next_vertex_buffers,
        })
    }
}
