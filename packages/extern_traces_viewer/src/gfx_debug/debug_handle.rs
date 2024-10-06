#[cfg(not(feature = "renderdoc_hooks"))]
mod implementation {
    use std::sync::Arc;
    use vulkano::instance::Instance;

    pub struct DebugHandle();

    impl DebugHandle {
        pub fn new(_instance: Arc<Instance>) -> DebugHandle {
            DebugHandle()
        }

        pub fn start_frame_capture(&mut self) {}

        pub fn end_frame_capture(&mut self) {}
    }
}

#[cfg(feature = "renderdoc_hooks")]
mod implementation {
    use renderdoc;
    use renderdoc::{DevicePointer, RenderDoc, V110};
    use std::ptr::null;
    use std::sync::Arc;
    use vulkano::instance::Instance;
    use vulkano::{Handle, VulkanObject};

    pub struct DebugHandle {
        device: DevicePointer,
        renderdoc: Result<RenderDoc<V110>, renderdoc::Error>,
    }

    impl DebugHandle {
        pub fn new(instance: Arc<Instance>) -> DebugHandle {
            let device = DevicePointer::from({
                let handle = instance.handle();
                let raw_handle = handle.as_raw() as *const *const libc::c_void;

                unsafe { *raw_handle }
            });

            DebugHandle {
                device,
                renderdoc: RenderDoc::new(),
            }
        }

        pub fn start_frame_capture(&mut self) {
            if let Ok(ref mut renderdoc) = self.renderdoc {
                renderdoc.start_frame_capture(self.device.clone(), null())
            }
        }

        pub fn end_frame_capture(&mut self) {
            if let Ok(ref mut renderdoc) = self.renderdoc {
                renderdoc.end_frame_capture(self.device.clone(), null())
            }
        }
    }
}

pub use implementation::DebugHandle;
