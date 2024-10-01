use crate::gfx_debug::ctx::GraphicsContext;
use crate::gfx_debug::process::{EncodedShader, VertexBuffer};
use crate::gfx_debug::resources::buffers::{
    BufferShaderStageResult, BuffersDataContainer, SharedDescriptorSet,
};
use anyhow::format_err;
use gcn::resources::VertexBufferResource;
use gcn::GCNInstructionStream;
use gcn_spirv::execution_state_input::{
    compute_shader_user_data_descriptor, shader_user_data_layout_binding,
    COMPUTE_SHADER_DESCRIPTOR_SET_IDX,
};
use gcn_spirv::module::translate_compute_shader;
use itertools::Itertools;
use pm4::dispatch_direct::DispatchDirectPacket;
use pm4::ComputePipeline;
use rspirv::binary::Assemble;
use std::collections::BTreeMap;
use std::iter;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract,
};
use vulkano::descriptor_set::layout::{DescriptorSetLayout, DescriptorSetLayoutCreateInfo};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo, ShaderStages};
use vulkano::sync::GpuFuture;

pub fn process_dispatch_command(
    graphics_context: &mut GraphicsContext,
    dispatch_packet: DispatchDirectPacket,
    ComputePipeline {
        address_lo,
        user_data,
        thread_x,
        thread_y,
        thread_z,
        resource2,
        ..
    }: ComputePipeline,
    vertex_buffers: &[VertexBuffer],
    data: &BuffersDataContainer,
    known_shaders: &BTreeMap<u32, EncodedShader>,
) -> Result<BufferShaderStageResult, anyhow::Error> {
    let device = &graphics_context.device;

    let (compute_shader, descriptor_set) = {
        let descriptor_offset = 1;

        let shader = known_shaders
            .get(&address_lo)
            .ok_or_else(|| format_err!("unknown shader"))?;

        let buffer_usage_instances = shader
            .vertex_buffer_references
            .iter()
            .map(|(program_counter, buffer_idx)| {
                (
                    *program_counter,
                    vertex_buffers[*buffer_idx].vertex_buffer.resource.clone(),
                )
            })
            .collect::<BTreeMap<u64, VertexBufferResource>>();

        let buffer_resources = shader
            .vertex_buffer_references
            .iter()
            .map(|(program_counter, buffer_idx)| *buffer_idx)
            .unique()
            .into_iter()
            .collect::<Vec<_>>();

        let bytes = shader.bytes.to_vec();
        let instructions = GCNInstructionStream::new(&bytes)?;

        let module = translate_compute_shader(
            instructions.instructions.as_slice(),
            thread_x.NUM_THREAD_FULL,
            thread_y.NUM_THREAD_FULL,
            thread_z.NUM_THREAD_FULL,
            resource2.USER_SGPR,
            resource2.TGID_X_EN,
            resource2.TGID_Y_EN,
            resource2.TGID_Z_EN,
            &buffer_usage_instances,
            buffer_resources.iter().enumerate().map(|(idx, it)| {
                (
                    idx as u32 + descriptor_offset,
                    &vertex_buffers[*it].vertex_buffer.resource,
                )
            }),
        )?;

        let code = module.assemble();
        let shader =
            unsafe { ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(&code)) }?;

        let descriptor_set = SharedDescriptorSet::new(
            graphics_context,
            descriptor_offset,
            buffer_resources.as_slice(),
            &data,
        )?;

        (
            shader
                .entry_point("main")
                .ok_or_else(|| format_err!("missing entrypoint"))?,
            descriptor_set,
        )
    };

    let stage = PipelineShaderStageCreateInfo::new(compute_shader);
    let pipeline_layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())?,
    )?;

    let pipeline = vulkano::pipeline::ComputePipeline::new(
        graphics_context.device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, pipeline_layout.clone()),
    )?;

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        graphics_context.device.clone(),
        Default::default(),
    ));

    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        graphics_context.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )?;

    let compute_descriptor_set = PersistentDescriptorSet::new(
        &graphics_context.descriptor_set_allocator,
        DescriptorSetLayout::new(
            graphics_context.device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: shader_user_data_layout_binding(ShaderStages::COMPUTE)
                    .chain(descriptor_set.bindings())
                    .collect(),
                ..Default::default()
            },
        )?,
        iter::once(compute_shader_user_data_descriptor(
            graphics_context.allocator.clone(),
            &user_data,
        )?)
        .chain(descriptor_set.write_descriptor_set()),
        [],
    )?;

    builder.bind_pipeline_compute(pipeline.clone())?;

    descriptor_set.stage_input(&mut builder)?;

    builder
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            pipeline_layout.clone(),
            COMPUTE_SHADER_DESCRIPTOR_SET_IDX,
            vec![compute_descriptor_set],
        )?
        .dispatch([
            dispatch_packet.dim_x,
            dispatch_packet.dim_y,
            dispatch_packet.dim_z,
        ])?;

    descriptor_set.stage_output(&mut builder)?;

    let command_buffer = builder.build()?;

    let finished = command_buffer.execute(graphics_context.queue.clone())?;

    finished.then_signal_fence_and_flush()?.wait(None)?;

    Ok(descriptor_set.flush_output_memory()?)
}
