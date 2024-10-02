use crate::gfx_debug::ctx::GraphicsContext;
use crate::gfx_debug::process::{EncodedShader, VertexBuffer};
use crate::gfx_debug::resources::buffers::{
    BufferShaderStageResult, BuffersDataContainer, SharedDescriptorSet,
};
use crate::gfx_debug::resources::images::ImageBufferResourceMemory;
use anyhow::format_err;
use gcn::resources::VertexBufferResource;
use gcn::GCNInstructionStream;
use gcn_spirv::execution_state_input::{
    shader_user_data_layout_binding, vertex_shader_user_data_descriptor,
    VERTEX_SHADER_DESCRIPTOR_SET_IDX,
};
use gcn_spirv::module::translate_vertex_shader;
use itertools::Itertools;
use pm4::draw_index_auto::DrawIndexAutoPacket;
use pm4::{BlendOp, ColorFormat, CombFunc, CompareFrag, VertexShader, ZFormat, VGT_DI_PRIM_TYPE};
use rspirv::binary::Assemble;
use std::collections::BTreeMap;
use std::iter;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract,
    RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::layout::{DescriptorSetLayout, DescriptorSetLayoutCreateInfo};
use vulkano::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::format::{ClearValue, Format};
use vulkano::image::{ImageLayout, ImageUsage};
use vulkano::pipeline::graphics::color_blend::{
    AttachmentBlend, BlendFactor, ColorBlendAttachmentState, ColorBlendState, ColorComponents,
};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Framebuffer,
    FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, Subpass, SubpassDescription,
};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo, ShaderStages};
use vulkano::sync::GpuFuture;

pub struct DrawShaderStageResult {
    pub buffers: BufferShaderStageResult,
    pub color_buffer: Option<Box<[u8]>>,
    pub depth_buffer: Option<Box<[u8]>>,
}

pub fn process_draw_command(
    graphics_context: &GraphicsContext,
    draw_packet: DrawIndexAutoPacket,
    pipeline_input: pm4::GraphicsPipeline,
    vertex_buffers: &[VertexBuffer],
    data: &BuffersDataContainer,
    known_shaders: &BTreeMap<u32, EncodedShader>,
    last_color_buffer: &Option<Box<[u8]>>,
    last_depth_buffer: &Option<Box<[u8]>>,
) -> Result<DrawShaderStageResult, anyhow::Error> {
    let color_buffer = (|| -> Result<_, anyhow::Error> {
        let Some(it) = &pipeline_input.color_buffer.shader_mask else {
            return Ok(None);
        };

        if it.OUTPUT0_ENABLE == 0 {
            return Ok(None);
        }

        let Some(color_buffer) = pipeline_input.color_buffer.color0 else {
            return Ok(None);
        };

        let format = match color_buffer.info.FORMAT {
            ColorFormat::COLOR_8_8_8_8 => Format::R8G8B8A8_UNORM,
            _ => unimplemented!(),
        };

        let color_image_resource = ImageBufferResourceMemory::new(
            last_color_buffer.as_ref().map(|it| it.as_ref()),
            format,
            [1920, 1080, 1],
            graphics_context,
            ImageUsage::COLOR_ATTACHMENT,
        )?;

        Ok(Some((format, color_image_resource, color_buffer)))
    })()?;

    let color_buffer_exists = color_buffer.is_some();

    let depth_buffer = if pipeline_input
        .depth_buffer
        .depth
        .control
        .as_ref()
        .unwrap()
        .Z_ENABLE
    {
        let z = pipeline_input.depth_buffer.z.unwrap();
        let format = match z.info.FORMAT {
            ZFormat::Z_32_FLOAT => Format::D32_SFLOAT,
            _ => unimplemented!(),
        };

        let depth_image_resource = ImageBufferResourceMemory::new(
            last_depth_buffer.as_ref().map(|it| it.as_ref()),
            format,
            [1920, 1080, 1],
            graphics_context,
            ImageUsage::DEPTH_STENCIL_ATTACHMENT,
        )?;

        Some((format, depth_image_resource))
    } else {
        None
    };

    let depth_buffer_exists = depth_buffer.as_ref().is_some();

    let (vertex_shader, vertex_shader_user_data, vertex_shader_exports, vertex_descriptor_set_ttt) = {
        let VertexShader {
            entrypoint_gpu_address,
            user_data,
            ..
        } = &pipeline_input.vertex_shader;

        let descriptor_offset = 1;

        let shader = known_shaders.get(&entrypoint_gpu_address.unwrap()).unwrap();

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

        let (module, exports) = translate_vertex_shader(
            &instructions.instructions,
            &buffer_usage_instances,
            buffer_resources.iter().enumerate().map(|(idx, it)| {
                (
                    idx as u32 + descriptor_offset,
                    &vertex_buffers[*it].vertex_buffer.resource,
                )
            }),
        )?;

        let code = module.assemble();
        let shader = unsafe {
            ShaderModule::new(
                graphics_context.device.clone(),
                ShaderModuleCreateInfo::new(&code),
            )
        }?;

        let entry_point = shader
            .entry_point("main")
            .ok_or_else(|| format_err!("missing entrypoint"))?;

        let descriptor_set = SharedDescriptorSet::new(
            graphics_context,
            descriptor_offset,
            buffer_resources.as_slice(),
            data,
        )?;

        (entry_point, user_data, exports, descriptor_set)
    };

    // let (
    //     pixel_shader,
    //     image_descriptors,
    //     pixel_buffers_descriptor_set_layout_bindings,
    //     pixel_buffers_write_descriptor_set,
    //     stage_pixel_input_shared_memory,
    //     stage_pixel_output_shared_memory,
    //     flush_pixel_shared_writes,
    // ) = {
    //     let pixel_shader = &pipeline_input.pixel_shader;

    //     let analysis_state = AnalysisState::new(&pixel_shader.user_data.0);

    //     let shader_invocation =
    //         unsafe { ShaderInvocation::decode_from_memory(pixel_shader.address.unwrap()) }?;

    //     let instructions = shader_invocation.as_flat_instructions()?;

    //     let image_sample_instances =
    //         pixel_shader_extract_image_usages(instructions.as_slice(), &pixel_shader.user_data.0);

    //     let image_sample_instances = image_sample_instances
    //         .into_iter()
    //         .map(
    //             |ImageSamplerUsage {
    //                  texture_resource,
    //                  sampler_resource,
    //                  program_counter,
    //              }| {
    //                 (
    //                     program_counter,
    //                     (
    //                         (texture_resource.raw, texture_resource.resource.clone()),
    //                         sampler_resource.resource.clone(),
    //                     ),
    //                 )
    //             },
    //         )
    //         .unique()
    //         .collect::<BTreeMap<_, _>>();

    //     let images = image_sample_instances
    //         .values()
    //         .cloned()
    //         .unique()
    //         .collect::<Vec<_>>();

    //     let image_descriptors = image_descriptors(images.as_slice(), graphics_context).unwrap();

    //     let buffer_usages =
    //         extract_buffer_usages(instructions.as_slice(), &pixel_shader.user_data.0);

    //     let buffer_usage_instances = buffer_usages
    //         .into_iter()
    //         .map(|it| (it.program_counter, it.resource.resource))
    //         .collect::<BTreeMap<u64, VertexBufferResource>>();

    //     let buffer_resources = buffer_usage_instances
    //         .values()
    //         .unique()
    //         .cloned()
    //         .enumerate()
    //         .map(|(idx, it)| ((idx + images.len()) as u32 + 1, it))
    //         .collect::<Vec<_>>();

    //     let (
    //         descriptor_set_layout_bindings,
    //         write_descriptor_set,
    //         stage_input_shared_memory,
    //         stage_output_shared_memory,
    //         flush_shared_writes,
    //     ) = make_shared_descriptor_set(graphics_context, buffer_resources.as_slice()).unwrap();

    //     let module = translate_pixel_shader(
    //         instructions.as_slice(),
    //         vertex_shader_exports.as_slice(),
    //         images
    //             .iter()
    //             .enumerate()
    //             .map(|(idx, it)| ((idx + 1) as u32, it)),
    //         image_sample_instances,
    //         &buffer_usage_instances,
    //         buffer_resources.iter().map(|(idx, it)| (*idx, it)),
    //     )?;
    //     let code = module.assemble();

    //     let shader = unsafe {
    //         ShaderModule::new(
    //             graphics_context.device.clone(),
    //             ShaderModuleCreateInfo::new(&code),
    //         )
    //     }?;

    //     (
    //         shader.entry_point("main").unwrap(),
    //         image_descriptors,
    //         descriptor_set_layout_bindings,
    //         write_descriptor_set,
    //         stage_input_shared_memory,
    //         stage_output_shared_memory,
    //         flush_shared_writes,
    //     )
    // };

    let render_pass = {
        let mut attachments = vec![];

        let color_attachment_idx = color_buffer.as_ref().map(|(format, _, _)| {
            let attachment_idx = attachments.len();

            attachments.push(AttachmentDescription {
                format: *format,
                load_op: AttachmentLoadOp::Load,
                store_op: AttachmentStoreOp::Store,
                initial_layout: ImageLayout::ColorAttachmentOptimal,
                final_layout: ImageLayout::ColorAttachmentOptimal,
                ..Default::default()
            });

            attachment_idx
        });

        let depth_attachment_idx = depth_buffer.as_ref().map(|(format, _)| {
            let attachment_idx = attachments.len();

            let render_control = pipeline_input.depth_buffer.render_control.as_ref().unwrap();

            attachments.push(AttachmentDescription {
                format: *format,
                load_op: if render_control.DEPTH_CLEAR_ENABLE {
                    AttachmentLoadOp::Clear
                } else {
                    AttachmentLoadOp::Load
                },
                store_op: AttachmentStoreOp::Store,
                initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                ..Default::default()
            });

            attachment_idx
        });

        let subpass_description = SubpassDescription {
            color_attachments: color_attachment_idx
                .map(|idx| {
                    Some(AttachmentReference {
                        attachment: idx as u32,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    })
                })
                .into_iter()
                .collect(),
            depth_stencil_attachment: depth_attachment_idx.map(|idx| AttachmentReference {
                attachment: idx as u32,
                layout: ImageLayout::DepthStencilAttachmentOptimal,
                ..Default::default()
            }),
            ..Default::default()
        };

        RenderPass::new(
            graphics_context.device.clone(),
            RenderPassCreateInfo {
                attachments,
                subpasses: vec![subpass_description],
                ..Default::default()
            },
        )?
    };

    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: {
                let mut attachments = vec![];

                if let Some((_, color_buffer_image, _)) = color_buffer.as_ref() {
                    attachments.push(color_buffer_image.image_view()?);
                }

                if let Some((_, depth_buffer_image)) = depth_buffer.as_ref() {
                    attachments.push(depth_buffer_image.image_view()?);
                }

                attachments
            },
            ..Default::default()
        },
    )?;

    let vertex_descriptor_set = PersistentDescriptorSet::new(
        &graphics_context.descriptor_set_allocator,
        DescriptorSetLayout::new(
            graphics_context.device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: shader_user_data_layout_binding(ShaderStages::VERTEX)
                    .chain(vertex_descriptor_set_ttt.bindings())
                    .collect(),
                ..Default::default()
            },
        )
        .unwrap(),
        iter::once(
            vertex_shader_user_data_descriptor(
                graphics_context.allocator.clone(),
                &vertex_shader_user_data,
            )
            .unwrap(),
        )
        .chain(vertex_descriptor_set_ttt.write_descriptor_set())
        .collect::<Vec<_>>(),
        [],
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

    // let pixel_descriptor_set = {
    //     let image_descriptors = {
    //         let mut results = vec![];

    //         for descriptor in image_descriptors {
    //             builder
    //                 .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
    //                     descriptor.upload_buffer,
    //                     descriptor.image,
    //                 ))?;

    //             results.push(descriptor.descriptor);
    //         }

    //         results
    //     };

    //     PersistentDescriptorSet::new(
    //         &graphics_context.descriptor_set_allocator,
    //         DescriptorSetLayout::new(
    //             graphics_context.device.clone(),
    //             DescriptorSetLayoutCreateInfo {
    //                 bindings: shader_user_data_layout_binding(ShaderStages::FRAGMENT)
    //                     .chain((0..image_descriptors.len()).map(|it| {
    //                         (
    //                             (it + 1) as u32,
    //                             DescriptorSetLayoutBinding {
    //                                 stages: ShaderStages::FRAGMENT,
    //                                 ..DescriptorSetLayoutBinding::descriptor_type(
    //                                     DescriptorType::CombinedImageSampler,
    //                                 )
    //                             },
    //                         )
    //                     }))
    //                     .chain(pixel_buffers_descriptor_set_layout_bindings)
    //                     .collect(),
    //                 ..Default::default()
    //             },
    //         )?,
    //         {
    //             let mut result = vec![];

    //             result.push(
    //                 pixel_shader_user_data_descriptor(
    //                     graphics_context.allocator.clone(),
    //                     &pipeline_input.pixel_shader.user_data,
    //                 )?,
    //             );

    //             result.extend(image_descriptors);

    //             result.extend(pixel_buffers_write_descriptor_set);

    //             result
    //         },
    //         [],
    //     )?
    // };

    let (pipeline, pipeline_layout) = {
        let stages = {
            let mut stages = Vec::new();
            stages.push(PipelineShaderStageCreateInfo::new(vertex_shader));

            match pipeline_input
                .vertex_grouper_tesselator
                .primitive_type
                .unwrap()
                .PRIM_TYPE
            {
                VGT_DI_PRIM_TYPE::DI_PT_TRILIST => {
                    // do nothing, this is the default
                }
                VGT_DI_PRIM_TYPE::DI_PT_RECTLIST => {
                    // Only really setup to work with the clear embedded shader. If more
                    // usages are found of RECT_LIST, will implement a more robust
                    // remapping.

                    mod geometry {
                        vulkano_shaders::shader! {
                            ty: "geometry",
                            path: "src/gfx_debug/rect_list_to_tri_list.geometry.glsl",
                        }
                    }

                    let geometry = geometry::load(graphics_context.device.clone())
                        .unwrap()
                        .entry_point("main")
                        .unwrap();

                    stages.push(PipelineShaderStageCreateInfo::new(geometry));
                }
                _ => unimplemented!(),
            };

            {
                // todo: use real pixel shader

                mod ps {
                    vulkano_shaders::shader! {
                        ty: "fragment",
                        src: r"
                            #version 450

                            layout(location = 0) in vec2 input1;

                            layout(location = 0) out vec4 FragColor;

                            void main()
                            {
                                // Emit a fixed color (in this case, a shade of blue)
                                vec3 fixedColor = vec3(0.2, 0.4, 0.8);

                                // Set the output color (fully opaque)
                                FragColor = vec4(fixedColor, 1.0);
                            }
                        ",
                    }
                }

                let pixel_shader = ps::load(graphics_context.device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap();

                stages.push(PipelineShaderStageCreateInfo::new(pixel_shader));
            }

            stages
        };

        let layout = PipelineLayout::new(graphics_context.device.clone(), {
            let mut set_layouts = vec![DescriptorSetLayoutCreateInfo::default(); 1];

            set_layouts[VERTEX_SHADER_DESCRIPTOR_SET_IDX as usize].bindings =
                vertex_descriptor_set.layout().bindings().clone();

            // set_layouts[PIXEL_SHADER_DESCRIPTOR_SET_IDX as usize].bindings =
            //     pixel_descriptor_set.layout().bindings().clone();

            PipelineLayoutCreateInfo {
                set_layouts: set_layouts
                    .into_iter()
                    .map(|it| {
                        DescriptorSetLayout::new(graphics_context.device.clone(), it).unwrap()
                    })
                    .collect(),
                ..Default::default()
            }
        })?;

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        (
            GraphicsPipeline::new(
                graphics_context.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(VertexInputState::new()),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState {
                        viewports: [Viewport {
                            offset: [0.0, 1080.0],
                            extent: [1920.0, -1080.0],
                            depth_range: 0.0..=1.0,
                        }]
                        .into_iter()
                        .collect(),
                        ..Default::default()
                    }),
                    rasterization_state: Some(RasterizationState::default()),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: color_buffer.as_ref().map(|(_, _, color_buffer)| {
                        ColorBlendState::with_attachment_states(
                            subpass.num_color_attachments(),
                            ColorBlendAttachmentState {
                                blend: color_buffer.blend.as_ref().map(|it| AttachmentBlend {
                                    src_color_blend_factor: blend_factor_from(&it.COLOR_SRCBLEND),
                                    dst_color_blend_factor: blend_factor_from(&it.COLOR_DESTBLEND),
                                    color_blend_op: blend_op_from(&it.COLOR_COMB_FCN),
                                    src_alpha_blend_factor: blend_factor_from(&it.ALPHA_SRCBLEND),
                                    dst_alpha_blend_factor: blend_factor_from(&it.ALPHA_DESTBLEND),
                                    alpha_blend_op: blend_op_from(&it.ALPHA_COMB_FCN),
                                }),
                                color_write_mask: ColorComponents::all(),
                                color_write_enable: true,
                            },
                        )
                    }),
                    depth_stencil_state: depth_buffer.as_ref().map(|_| DepthStencilState {
                        depth: Some(DepthState {
                            write_enable: true,
                            compare_op: {
                                match pipeline_input.depth_buffer.depth.control.unwrap().ZFUNC {
                                    CompareFrag::FRAG_NEVER => CompareOp::Always,
                                    CompareFrag::FRAG_LEQUAL => CompareOp::LessOrEqual,
                                    CompareFrag::FRAG_ALWAYS => CompareOp::Never,
                                    _ => unimplemented!(),
                                }
                            },
                        }),
                        ..Default::default()
                    }),
                    subpass: Some(subpass.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout.clone())
                },
            )?,
            layout,
        )
    };

    if let Some((_, it, _)) = &color_buffer {
        it.stage_input(&mut builder)?;
    }

    if let Some((_, it)) = &depth_buffer {
        it.stage_input(&mut builder)?;
    }

    vertex_descriptor_set_ttt.stage_input(&mut builder)?;
    // stage_pixel_input_shared_memory(&mut builder).unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: {
                    let mut clear_values = vec![];

                    if color_buffer_exists {
                        clear_values.push(None);
                    }

                    if depth_buffer_exists {
                        let render_control = pipeline_input.depth_buffer.render_control.unwrap();
                        clear_values.push(if render_control.DEPTH_CLEAR_ENABLE {
                            let clear_value = pipeline_input.depth_buffer.depth.clear.unwrap();

                            Some(ClearValue::Depth(bytemuck::cast(clear_value)))
                        } else {
                            None
                        });
                    }

                    clear_values
                },
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )?
        .bind_pipeline_graphics(pipeline.clone())?;

    builder.bind_descriptor_sets(
        PipelineBindPoint::Graphics,
        pipeline_layout.clone(),
        VERTEX_SHADER_DESCRIPTOR_SET_IDX,
        vec![vertex_descriptor_set],
    )?;

    // builder
    //     .bind_descriptor_sets(
    //         PipelineBindPoint::Graphics,
    //         pipeline_layout.clone(),
    //         PIXEL_SHADER_DESCRIPTOR_SET_IDX,
    //         vec![pixel_descriptor_set],
    //     )?;

    builder.draw(draw_packet.index_count, 1, 0, 0)?;

    builder.end_render_pass(Default::default())?;

    vertex_descriptor_set_ttt.stage_output(&mut builder)?;
    // stage_pixel_output_shared_memory(&mut builder).unwrap();

    if let Some((_, it, _)) = &color_buffer {
        it.stage_output(&mut builder)?;
    }

    if let Some((_, it)) = &depth_buffer {
        it.stage_output(&mut builder)?;
    }

    let command_buffer = builder.build()?;

    let finished = command_buffer.execute(graphics_context.queue.clone())?;

    finished.then_signal_fence_and_flush()?.wait(None)?;

    let buffers = vertex_descriptor_set_ttt.flush_output_memory()?;
    // flush_pixel_shared_writes().unwrap();

    Ok(DrawShaderStageResult {
        color_buffer: if let Some((_, image_buffer, _)) = &color_buffer {
            Some(image_buffer.flush_output_memory()?)
        } else {
            None
        },
        depth_buffer: if let Some((_, image_buffer)) = &depth_buffer {
            Some(image_buffer.flush_output_memory()?)
        } else {
            None
        },
        buffers,
    })
}

fn blend_factor_from(op: &BlendOp) -> BlendFactor {
    match op {
        BlendOp::BLEND_ZERO => BlendFactor::Zero,
        BlendOp::BLEND_ONE => BlendFactor::One,
        BlendOp::BLEND_SRC_COLOR => BlendFactor::SrcColor,
        BlendOp::BLEND_ONE_MINUS_SRC_COLOR => BlendFactor::OneMinusSrcColor,
        BlendOp::BLEND_SRC_ALPHA => BlendFactor::SrcAlpha,
        BlendOp::BLEND_ONE_MINUS_SRC_ALPHA => BlendFactor::OneMinusSrcAlpha,
        BlendOp::BLEND_DST_ALPHA => BlendFactor::DstAlpha,
        BlendOp::BLEND_ONE_MINUS_DST_ALPHA => BlendFactor::OneMinusDstAlpha,
        BlendOp::BLEND_DST_COLOR => BlendFactor::DstColor,
        BlendOp::BLEND_ONE_MINUS_DST_COLOR => BlendFactor::OneMinusDstColor,
        BlendOp::BLEND_SRC_ALPHA_SATURATE => BlendFactor::SrcAlphaSaturate,
        BlendOp::BLEND_CONSTANT_COLOR => BlendFactor::ConstantColor,
        BlendOp::BLEND_ONE_MINUS_CONSTANT_COLOR => BlendFactor::OneMinusConstantColor,
        BlendOp::BLEND_SRC1_COLOR => BlendFactor::Src1Color,
        BlendOp::BLEND_INV_SRC1_COLOR => BlendFactor::OneMinusSrc1Color,
        BlendOp::BLEND_SRC1_ALPHA => BlendFactor::Src1Alpha,
        BlendOp::BLEND_INV_SRC1_ALPHA => BlendFactor::OneMinusSrc1Alpha,
        BlendOp::BLEND_CONSTANT_ALPHA => BlendFactor::ConstantAlpha,
        BlendOp::BLEND_ONE_MINUS_CONSTANT_ALPHA => BlendFactor::OneMinusConstantAlpha,
        _ => unimplemented!("{:?}", op),
    }
}

fn blend_op_from(comb_func: &CombFunc) -> vulkano::pipeline::graphics::color_blend::BlendOp {
    match comb_func {
        CombFunc::COMB_DST_PLUS_SRC => vulkano::pipeline::graphics::color_blend::BlendOp::Add,
        CombFunc::COMB_SRC_MINUS_DST => vulkano::pipeline::graphics::color_blend::BlendOp::Subtract,
        CombFunc::COMB_MIN_DST_SRC => vulkano::pipeline::graphics::color_blend::BlendOp::Min,
        CombFunc::COMB_MAX_DST_SRC => vulkano::pipeline::graphics::color_blend::BlendOp::Max,
        CombFunc::COMB_DST_MINUS_SRC => {
            vulkano::pipeline::graphics::color_blend::BlendOp::ReverseSubtract
        }
    }
}
