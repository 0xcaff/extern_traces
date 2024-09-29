use gcn::test_utils::GCNInstructionStream;
use gcn_spirv::module::construct_control_flow_graph;
use sha2::{Digest, Sha256};
use snapshot_test_utils::generate_snapshots;
use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;

#[test]
fn snapshots() -> Result<(), anyhow::Error> {
    generate_snapshots(
        "../gcn/test_data/captured/**/*.sb",
        "../gcn/test_data/captured",
        "../gcn/test_data/snapshots",
        |input, collector| {
            let shader = GCNInstructionStream::new(&input).unwrap();

            let graph = construct_control_flow_graph(&shader.instructions);

            collector.result("graph", &format!("{:#?}", graph))?;

            Ok(())
        },
    )?;

    Ok(())
}

#[derive(Debug)]
enum ShaderKind {
    Compute,
    Vertex,
    Pixel,
}

impl From<u8> for ShaderKind {
    fn from(value: u8) -> Self {
        match value {
            0 => ShaderKind::Compute,
            1 => ShaderKind::Vertex,
            2 => ShaderKind::Pixel,
            _ => panic!("Invalid shader kind: {}", value),
        }
    }
}

#[derive(Debug)]
struct ShaderInvocation<'a> {
    address: u32,
    kind: ShaderKind,
    length: u32,
    bytes: &'a [u8],
}

#[derive(Debug)]
struct CommandBuffer {
    draw: Vec<u8>,
    compute: Vec<u8>,
}

#[derive(Debug)]
struct ExtraData<'a> {
    draw_command_buffers: Vec<&'a [u8]>,
    compute_command_buffers: Vec<&'a [u8]>,
    shaders: Vec<ShaderInvocation<'a>>,
}

fn parse_extra_data(data: &[u8]) -> Result<ExtraData, anyhow::Error> {
    let mut cursor = Cursor::new(data);

    // Read count
    let mut count_bytes = [0u8; size_of::<u32>()];
    cursor.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes);

    // Read draw and compute sizes
    let mut draw_sizes = vec![0u32; count as usize];
    let mut compute_sizes = vec![0u32; count as usize];
    cursor.read_exact(bytemuck::cast_slice_mut(&mut draw_sizes))?;
    cursor.read_exact(bytemuck::cast_slice_mut(&mut compute_sizes))?;

    // Read command buffers
    let mut draw_command_buffers = Vec::with_capacity(count as usize);
    let mut compute_command_buffers = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let draw_start = cursor.position() as usize;
        cursor.seek(std::io::SeekFrom::Current(draw_sizes[i] as i64))?;
        draw_command_buffers.push(&data[draw_start..draw_start + draw_sizes[i] as usize]);
    }

    for i in 0..count as usize {
        let compute_start = cursor.position() as usize;
        cursor.seek(std::io::SeekFrom::Current(compute_sizes[i] as i64))?;
        compute_command_buffers
            .push(&data[compute_start..compute_start + compute_sizes[i] as usize]);
    }

    // Read shader count
    let mut shader_count_bytes = [0u8; size_of::<u32>()];
    cursor.read_exact(&mut shader_count_bytes)?;
    let shader_count = u32::from_le_bytes(shader_count_bytes);

    // Read shaders
    let mut shaders = Vec::with_capacity(shader_count as usize);
    for _ in 0..shader_count {
        let mut address_bytes = [0u8; size_of::<u32>()];
        cursor.read_exact(&mut address_bytes)?;
        let address = u32::from_le_bytes(address_bytes);

        let mut kind_byte = [0u8; 1];
        cursor.read_exact(&mut kind_byte)?;
        let kind = ShaderKind::from(kind_byte[0]);

        let mut length_bytes = [0u8; size_of::<u32>()];
        cursor.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes) * 4;

        let start = cursor.position() as usize;
        cursor.seek(std::io::SeekFrom::Current(length as i64))?;
        let bytes = &data[start..start + length as usize];

        shaders.push(ShaderInvocation {
            address,
            kind,
            length,
            bytes,
        });
    }

    Ok(ExtraData {
        draw_command_buffers,
        compute_command_buffers,
        shaders,
    })
}

#[test]
fn dump_shader_invocations() -> Result<(), anyhow::Error> {
    generate_snapshots(
        "test_data/captured/**/*.bin",
        "test_data/captured",
        "test_data/snapshots",
        |input, collector| {
            let extra_data = parse_extra_data(&input[..])?;

            for (command_buffer, kind) in extra_data
                .draw_command_buffers
                .iter()
                .map(|it| (it, "draw"))
                .chain(
                    extra_data
                        .compute_command_buffers
                        .iter()
                        .map(|it| (it, "compute")),
                )
            {
                if command_buffer.is_empty() {
                    continue;
                }

                let path = {
                    let mut hasher = Sha256::new();
                    hasher.update(*command_buffer);
                    let hashed = Digest::finalize(hasher);

                    let mut directory = collector.relative_path().to_path_buf();
                    directory.pop();

                    Path::new("../pm4/test_data/captures")
                        .join(directory)
                        .join(format!("{:x}.{}.pm4", &hashed, kind,))
                };

                let mut file = File::create(path)?;
                file.write_all(*command_buffer)?;
            }

            for shader in &extra_data.shaders {
                let bytes = bytemuck::cast_slice(&shader.bytes);

                let path = {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let hashed = Digest::finalize(hasher);

                    let mut directory = collector.relative_path().to_path_buf();
                    directory.pop();

                    Path::new("../gcn/test_data/captured")
                        .join(directory)
                        .join(format!(
                            "{:x}.{}.sb",
                            &hashed,
                            match shader.kind {
                                ShaderKind::Compute => "compute",
                                ShaderKind::Vertex => "vertex",
                                ShaderKind::Pixel => "pixel",
                            }
                        ))
                };

                let mut file = File::create(path)?;
                file.write_all(&bytes)?;
            }

            Ok(())
        },
    )?;

    Ok(())
}
