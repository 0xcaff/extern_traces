use crate::analysis::AnalysisState;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use bits::{FromBits, TryFromBitsContainer};
use gcn::instructions::formats::FormattedInstruction;
use gcn::instructions::ops::SMEMOpCode;
use gcn::instructions::Instruction;
use gcn::resources::{SamplerResource, TextureBufferResource, VertexBufferResource};

#[derive(Debug, Clone)]
pub struct VertexBufferResourceWithRaw {
    pub raw: [u32; 4],
    pub resource: VertexBufferResource,
}

impl VertexBufferResourceWithRaw {
    pub fn from_bits(values: &[u32; 4]) -> VertexBufferResourceWithRaw {
        VertexBufferResourceWithRaw {
            raw: values.clone(),
            resource: VertexBufferResource::from_bits(bytemuck::cast_slice(values)),
        }
    }
}

#[derive(Debug)]
pub struct BufferUsage {
    pub resource: VertexBufferResourceWithRaw,
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
                let mut values = [0; 4];
                for i in 0..4 {
                    values[i] = analysis_state
                        .get(register_base_idx + i as u8)
                        .value()
                        .unwrap();
                }

                VertexBufferResourceWithRaw::from_bits(&values)
            }),
            FormattedInstruction::SMEM(instr) => match instr.op {
                SMEMOpCode::s_buffer_load_dword
                | SMEMOpCode::s_buffer_load_dwordx2
                | SMEMOpCode::s_buffer_load_dwordx4
                | SMEMOpCode::s_buffer_load_dwordx8
                | SMEMOpCode::s_buffer_load_dwordx16 => Some({
                    let mut values = [0; 4];
                    for it in 0..4 {
                        values[it] = analysis_state
                            .get(((instr.sbase as u8) << 1) + it as u8)
                            .value()
                            .unwrap();
                    }

                    VertexBufferResourceWithRaw::from_bits(&values)
                }),
                _ => None,
            },
            _ => None,
        };

        if let Some(resource) = value {
            usages.push(BufferUsage {
                resource,
                program_counter,
            });
        }

        analysis_state.update(&instr);
    }

    usages
}

#[derive(Debug, Clone)]
pub struct TextureBufferResourceWithRaw {
    pub raw: [u32; 8],
    pub resource: TextureBufferResource,
}

impl TextureBufferResourceWithRaw {
    pub fn from_bits(values: &[u32; 8]) -> TextureBufferResourceWithRaw {
        TextureBufferResourceWithRaw {
            raw: values.clone(),
            resource: TextureBufferResource::try_from_bits_container(bytemuck::cast_slice(values))
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SamplerResourceWithRaw {
    pub raw: [u32; 4],
    pub resource: SamplerResource,
}

impl SamplerResourceWithRaw {
    pub fn from_bits(values: &[u32; 4]) -> SamplerResourceWithRaw {
        SamplerResourceWithRaw {
            raw: values.clone(),
            resource: SamplerResource::from_bits(bytemuck::cast_slice(values)),
        }
    }
}

#[derive(Debug)]
pub struct ImageSamplerUsage {
    pub texture_resource: TextureBufferResourceWithRaw,
    pub sampler_resource: SamplerResourceWithRaw,
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
                let mut values = [0; 4];
                for it in 0..4 {
                    values[it] = analysis_state
                        .get(register_base_idx + it as u8)
                        .value()
                        .unwrap();
                }

                SamplerResourceWithRaw::from_bits(&values)
            };

            let texture_resource = {
                let register_base_idx = instr.srsrc.lowest_register_idx();
                let mut values = [0; 8];
                for it in 0..8 {
                    values[it] = analysis_state
                        .get(register_base_idx + it as u8)
                        .value()
                        .unwrap();
                }

                TextureBufferResourceWithRaw::from_bits(&values)
            };

            textures.push(ImageSamplerUsage {
                texture_resource,
                sampler_resource,
                program_counter,
            });
        }

        analysis_state.update(&instr);
    }

    textures
}
