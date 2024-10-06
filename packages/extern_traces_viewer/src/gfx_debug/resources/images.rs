use crate::gfx_debug::ctx::{CommandBufferBuilder, GraphicsContext};
use gcn::resources::DestinationChannelSelect;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{CopyBufferToImageInfo, CopyImageToBufferInfo};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::format::Format;
use vulkano::image::sampler::ComponentSwizzle;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

pub struct ImageBufferResourceMemory {
    input_buffer: Option<Subbuffer<[u8]>>,
    device_buffer: Arc<Image>,
    output_buffer: Subbuffer<[u8]>,
}

impl ImageBufferResourceMemory {
    pub fn new(
        bytes: Option<&[u8]>,
        format: Format,
        extent: [u32; 3],
        graphics_context: &GraphicsContext,
        usage: ImageUsage,
    ) -> Result<ImageBufferResourceMemory, anyhow::Error> {
        let len = (extent[0] as u64 * extent[1] as u64 * extent[2] as u64) * format.block_size();

        let input_buffer = if let Some(bytes) = bytes {
            assert_eq!(bytes.len(), len as usize);
            let input_buffer = Buffer::new_slice::<u8>(
                graphics_context.allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                len,
            )?;

            {
                let mut host_buffer = input_buffer.write()?;
                host_buffer.copy_from_slice(bytes);
            }

            Some(input_buffer)
        } else {
            None
        };

        let image = Image::new(
            graphics_context.allocator.clone(),
            ImageCreateInfo {
                format,
                usage: usage | ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
                extent,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )?;

        let output_buffer = Buffer::new_slice::<u8>(
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
            len,
        )?;

        Ok(ImageBufferResourceMemory {
            input_buffer,
            device_buffer: image,
            output_buffer,
        })
    }

    pub fn image_view(&self) -> Result<Arc<ImageView>, anyhow::Error> {
        Ok(ImageView::new_default(self.device_buffer.clone())?)
    }

    pub fn stage_input(&self, builder: &mut CommandBufferBuilder) -> Result<(), anyhow::Error> {
        if let Some(input_buffer) = &self.input_buffer {
            builder.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                input_buffer.clone(),
                self.device_buffer.clone(),
            ))?;
        }

        Ok(())
    }

    pub fn stage_output(&self, builder: &mut CommandBufferBuilder) -> Result<(), anyhow::Error> {
        builder.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            self.device_buffer.clone(),
            self.output_buffer.clone(),
        ))?;

        Ok(())
    }

    pub fn flush_output_memory(&self) -> Result<Box<[u8]>, anyhow::Error> {
        Ok(self.output_buffer.read()?.to_vec().into_boxed_slice())
    }
}

struct ImageDescriptor {
    descriptor: WriteDescriptorSet,
    image: Arc<Image>,
    upload_buffer: Subbuffer<[u8]>,
}

// fn image_descriptors(
//     images: &[(([u32; 8], TextureBufferResource), SamplerResource)],
//     graphics_context: &GraphicsContext,
// ) -> Result<Vec<ImageDescriptor>, anyhow::Error> {
//     let mut descriptors = vec![];
//     for (idx, ((raw_texture_resource, texture_resource), sampler_resource)) in
//         images.iter().enumerate()
//     {
//         let sampler = Sampler::new(
//             graphics_context.device.clone(),
//             SamplerCreateInfo::default(),
//         )?;
//
//         let extent = [
//             texture_resource.width as u32 + 1,
//             texture_resource.height as u32 + 1,
//             texture_resource.depth as u32 + 1,
//         ];
//
//         let format = match (&texture_resource.dfmt, &texture_resource.nfmt) {
//             (SurfaceFormat::Format8_8_8_8, TextureChannelType::UNorm) => Format::R8G8B8A8_UNORM,
//             (SurfaceFormat::Format8_8_8_8, TextureChannelType::Srgb) => Format::R8G8B8A8_SRGB,
//             (SurfaceFormat::Format8, TextureChannelType::UNorm) => Format::R8_UNORM,
//             value => {
//                 unimplemented!("{:?}", value)
//             }
//         };
//
//         let tiling_params = sce_sys::ffi::make_tiling_parameters(raw_texture_resource);
//
//         let tiling_param_ref = tiling_params.as_ref().unwrap();
//
//         let untiled_size = unsafe { sce_sys::ffi::get_untiled_surface_size(tiling_param_ref) };
//
//         let image = Image::new(
//             graphics_context.allocator.clone(),
//             ImageCreateInfo {
//                 image_type: match texture_resource.texture_type {
//                     TextureType::Type1d | TextureType::Type1dArray => ImageType::Dim1d,
//                     TextureType::Type2d
//                     | TextureType::Type2dMsaa
//                     | TextureType::Type2dArrayMsaa
//                     | TextureType::Type2dArray => ImageType::Dim2d,
//                     TextureType::Type3d => ImageType::Dim3d,
//                     _ => unimplemented!(),
//                 },
//                 format,
//                 extent,
//                 usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
//                 ..Default::default()
//             },
//             AllocationCreateInfo::default(),
//         )?;
//
//         let address = texture_resource.base_address();
//
//         let upload_buffer = Buffer::new_slice::<u8>(
//             graphics_context.allocator.clone(),
//             BufferCreateInfo {
//                 usage: BufferUsage::TRANSFER_SRC,
//                 ..Default::default()
//             },
//             AllocationCreateInfo {
//                 memory_type_filter: MemoryTypeFilter::PREFER_HOST
//                     | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
//                 ..Default::default()
//             },
//             untiled_size as DeviceSize,
//         )?;
//
//         {
//             let mut upload_buffer_bytes = upload_buffer.write()?;
//
//             unsafe {
//                 sce_sys::ffi::detile(
//                     upload_buffer_bytes.as_mut_ptr(),
//                     address as _,
//                     tiling_param_ref,
//                 )
//             };
//         }
//
//         let image_view = ImageView::new(
//             image.clone(),
//             ImageViewCreateInfo {
//                 component_mapping: ComponentMapping {
//                     r: channel_component_mapping(&texture_resource.dst_sel_x),
//                     g: channel_component_mapping(&texture_resource.dst_sel_y),
//                     b: channel_component_mapping(&texture_resource.dst_sel_z),
//                     a: channel_component_mapping(&texture_resource.dst_sel_w),
//                 },
//                 ..ImageViewCreateInfo::from_image(&image)
//             },
//         )?;
//
//         descriptors.push(ImageDescriptor {
//             upload_buffer,
//             image,
//             descriptor: WriteDescriptorSet::image_view_sampler(idx as u32 + 1, image_view, sampler),
//         })
//     }
//
//     Ok(descriptors)
// }

pub fn channel_component_mapping(select: &DestinationChannelSelect) -> ComponentSwizzle {
    match select {
        DestinationChannelSelect::Zero => ComponentSwizzle::Zero,
        DestinationChannelSelect::One => ComponentSwizzle::One,
        DestinationChannelSelect::R => ComponentSwizzle::Red,
        DestinationChannelSelect::G => ComponentSwizzle::Green,
        DestinationChannelSelect::B => ComponentSwizzle::Blue,
        DestinationChannelSelect::A => ComponentSwizzle::Alpha,
    }
}
