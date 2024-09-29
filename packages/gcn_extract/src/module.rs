use crate::analysis::AnalysisState;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use bits::{FromBits, TryFromBitsContainer};
use gcn::instructions::formats::FormattedInstruction;
use gcn::instructions::ops::SMEMOpCode;
use gcn::instructions::Instruction;
use gcn::resources::{SamplerResource, TextureBufferResource, VertexBufferResource};

#[derive(Debug)]
pub struct BufferUsage {
    pub resource: VertexBufferResource,
    pub program_counter: u64,
}

pub fn extract_buffer_usages(
    instructions: &[Instruction],
    user_data: &BTreeMap<u8, u32>,
) -> Vec<BufferUsage> {
    let mut usages = vec![];

    let mut analysis_state = AnalysisState::new(user_data);

    for instr in instructions {
        let program_counter = instr.program_counter;

        let value = match &instr.inner {
            FormattedInstruction::MUBUF(instr) => Some({
                let register_base_idx = instr.srsrc.lowest_register_idx();
                let values = (0..4)
                    .into_iter()
                    .map(|it| analysis_state.get(register_base_idx + it).value())
                    .collect::<Vec<_>>();

                let values: &[u64] = bytemuck::cast_slice(values.as_slice());
                VertexBufferResource::from_bits(values)
            }),
            FormattedInstruction::SMEM(instr) => match instr.op {
                SMEMOpCode::s_buffer_load_dword
                | SMEMOpCode::s_buffer_load_dwordx2
                | SMEMOpCode::s_buffer_load_dwordx4
                | SMEMOpCode::s_buffer_load_dwordx8
                | SMEMOpCode::s_buffer_load_dwordx16 => Some({
                    let values = (0..4)
                        .into_iter()
                        .map(|it| analysis_state.get(((instr.sbase as u8) << 1) + it).value())
                        .collect::<Vec<_>>();

                    let values: &[u64] = bytemuck::cast_slice(values.as_slice());
                    VertexBufferResource::from_bits(values)
                }),
                _ => None,
            },
            _ => None,
        };

        if let Some(vertex_buffer_resource) = value {
            usages.push(BufferUsage {
                resource: vertex_buffer_resource,
                program_counter,
            });
        }

        analysis_state.update(&instr);
    }

    usages
}

#[derive(Debug)]
pub struct ImageSamplerUsage {
    pub raw_texture_resource: [u32; 8],
    pub texture_resource: TextureBufferResource,
    pub sampler_resource: SamplerResource,
    pub program_counter: u64,
}

pub fn pixel_shader_extract_image_usages(
    instructions: &[Instruction],
    user_data: &BTreeMap<u8, u32>,
) -> Vec<ImageSamplerUsage> {
    let mut textures = Vec::new();

    let mut analysis_state = AnalysisState::new(user_data);
    for instr in instructions {
        let program_counter = instr.program_counter;

        if let FormattedInstruction::MIMG(instr) = &instr.inner {
            let sampler_resource = {
                let register_base_idx = instr.ssamp.lowest_register_idx();
                let values = (0..4)
                    .into_iter()
                    .map(|it| analysis_state.get(register_base_idx + it).value())
                    .collect::<Vec<_>>();

                let values: &[u64] = bytemuck::cast_slice(values.as_slice());
                SamplerResource::from_bits(values)
            };

            let (raw_texture_resource, texture_resource) = {
                let register_base_idx = instr.srsrc.lowest_register_idx();
                let values = (0..8)
                    .into_iter()
                    .map(|it| analysis_state.get(register_base_idx + it).value())
                    .collect::<Vec<_>>();

                let resource = {
                    let values_read: &[u64] = bytemuck::cast_slice(values.as_slice());
                    TextureBufferResource::try_from_bits_container(values_read).unwrap()
                };

                ((*values.into_boxed_slice()).try_into().unwrap(), resource)
            };

            textures.push(ImageSamplerUsage {
                raw_texture_resource,
                texture_resource,
                sampler_resource,
                program_counter,
            });
        }

        analysis_state.update(&instr);
    }

    textures
}
