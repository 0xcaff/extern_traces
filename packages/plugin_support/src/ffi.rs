#[repr(C)]
pub struct Args {
    pub xmm0: [u64; 2],
    pub xmm1: [u64; 2],
    pub xmm2: [u64; 2],
    pub xmm3: [u64; 2],
    pub xmm4: [u64; 2],
    pub xmm5: [u64; 2],
    pub xmm6: [u64; 2],
    pub xmm7: [u64; 2],

    pub args: [u64; 6],
    extra_args: [u64; 0],
}

impl Args {
    pub unsafe fn extra_args_mut_ptr(&mut self, arg_idx: usize) -> *mut u64 {
        self.extra_args.as_mut_ptr().offset(arg_idx as _)
    }

    pub unsafe fn extra_args_ptr(&self, arg_idx: usize) -> *const u64 {
        self.extra_args.as_ptr().offset(arg_idx as _)
    }
}

#[repr(C)]
pub struct BufferReservation {
    buffer: *mut (),
    write_idx: u64,
    available_bytes: u64,
    is_new: bool,
}

#[repr(C)]
pub struct ThreadLoggingState {
    _private: [u8; 0],
}

extern "C" {
    fn thread_logging_state_reserve_space(
        thread_state: *mut ThreadLoggingState,
        length: usize,
    ) -> BufferReservation;

    fn buffer_reservation_write(
        reservation: *mut BufferReservation,
        data: *const libc::c_uchar,
        length: usize,
    );

    fn thread_logging_state_flush_reservation(
        thread_state: *mut ThreadLoggingState,
        reservation: BufferReservation,
    );
}

impl ThreadLoggingState {
    pub fn reserve(&mut self, length: usize) -> Option<BufferReservation> {
        let reservation = unsafe { thread_logging_state_reserve_space(self, length) };
        if reservation.buffer.is_null() {
            return None;
        }

        Some(reservation)
    }

    pub fn flush(&mut self, buffer_reservation: BufferReservation) {
        unsafe {
            thread_logging_state_flush_reservation(self, buffer_reservation);
        }
    }
}

impl BufferReservation {
    pub fn write(&mut self, data: &[u8]) {
        unsafe {
            buffer_reservation_write(self, data.as_ptr(), data.len());
        }
    }
}
