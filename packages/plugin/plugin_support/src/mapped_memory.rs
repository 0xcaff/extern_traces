use anyhow::bail;
use bitflags::bitflags;

pub struct MappedMemory {
    pub base: *mut libc::c_void,
    pub len: usize,
}

bitflags! {
    pub struct Protection: i32 {
        const Read = libc::PROT_READ;
        const Write = libc::PROT_WRITE;
        const Exec = libc::PROT_EXEC;
    }
}

const PAGE_SIZE: usize = 4096;

impl MappedMemory {
    pub fn allocate(length: usize) -> Result<MappedMemory, anyhow::Error> {
        Self::allocate_hint(0, length)
    }

    pub fn allocate_hint(address: u64, length: usize) -> Result<MappedMemory, anyhow::Error> {
        let pages_needed = (length + PAGE_SIZE - 1) / PAGE_SIZE;
        let total_size = pages_needed * PAGE_SIZE;

        let address = unsafe {
            libc::mmap(
                address as *mut libc::c_void,
                total_size,
                libc::PROT_WRITE | libc::PROT_READ,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if address as i64 == -1 {
            bail!("failed to map");
        };

        Ok(MappedMemory {
            len: total_size,
            base: address,
        })
    }

    pub fn protect(&mut self, protection: Protection) -> Result<(), anyhow::Error> {
        let result = unsafe { libc::mprotect(self.base, self.len, protection.bits()) };

        if result == -1 {
            bail!("failed to protect");
        }

        Ok(())
    }
}

impl Drop for MappedMemory {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base, self.len);
        }
    }
}
