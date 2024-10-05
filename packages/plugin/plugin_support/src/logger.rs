use crate::println;
use alloc::slice;
use core::ops::{Deref, DerefMut};
use core::ptr::{null_mut, NonNull};

struct ThreadLoggingState {
    thread_id: u64,
    is_finished: bool,
    current_buffer: BufferState,

    failed_reservations_count: u64,
    last_failed_reservations_count: u64,
    last_counter_flush_time: u64,
}

impl ThreadLoggingState {
    pub fn new() -> ThreadLoggingState {
        let thread_id = unsafe { scePthreadSelf() };

        let buffer = BufferState::try_create(INITIAL_ALLOCATION_SIZE).unwrap();

        ThreadLoggingState {
            thread_id,
            is_finished: false,
            current_buffer: buffer,
            failed_reservations_count: 0,
            last_failed_reservations_count: 0,
            last_counter_flush_time: 0,
        }
    }
}

impl ThreadLoggingState {
    fn reserve(&mut self, reservation_size: usize) -> Option<BufferReservation> {
        let current_buffer = &mut self.current_buffer;

        let free_space = current_buffer.free_space();
        if free_space > reservation_size {
            return Some(BufferReservation {
                buffer_state: BufferReservationRef::Existing(current_buffer),
                write_idx: current_buffer.write_idx,
                available_bytes: reservation_size,
            });
        }

        let next_size = usize::max(
            (((reservation_size * 2) + (INITIAL_ALLOCATION_SIZE - 1)) / INITIAL_ALLOCATION_SIZE)
                * INITIAL_ALLOCATION_SIZE,
            current_buffer.total_size * 2,
        );

        let Some(next_buffer) = BufferState::try_create(next_size) else {
            self.failed_reservations_count += 1;
            return None;
        };

        next_buffer.last_buffer = Some(current_buffer.inner);
        Some(BufferReservation {
            write_idx: next_buffer.write_idx,
            available_bytes: reservation_size,
            buffer_state: BufferReservationRef::New(next_buffer),
        })
    }
}

struct BufferReservation<'a> {
    write_idx: usize,
    available_bytes: usize,
    buffer_state: BufferReservationRef<'a>,
}

enum BufferReservationRef<'a> {
    Existing(&'a mut BufferState),
    New(BufferState),
}

impl<'a> Deref for BufferReservationRef<'a> {
    type Target = BufferState;

    fn deref(&self) -> &Self::Target {
        match self {
            BufferReservationRef::Existing(value) => value,
            BufferReservationRef::New(value) => value,
        }
    }
}

impl<'a> DerefMut for BufferReservationRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            BufferReservationRef::Existing(value) => value,
            BufferReservationRef::New(value) => value,
        }
    }
}

const INITIAL_ALLOCATION_SIZE: usize = (16 * 1024);


impl BufferReservation {
    pub fn write(&mut self, input: &[u8]) {
        let input_len = input.len();

        if input_len > self.available_bytes {
            panic!("invariant violated, exceeded reserved space");
        }

        let bytes = self.buffer_state.bytes_mut();
        let bytes_len = bytes.len();
        let end_position = (self.write_idx + input_len) % bytes_len;
        if end_position >= self.write_idx {
            bytes[self.write_idx..end_position].copy_from_slice(input);
        } else {
            let first_chunk_size = bytes_len - self.write_idx;
            bytes[self.write_idx..].copy_from_slice(&input[0..first_chunk_size]);
            bytes[0..end_position].copy_from_slice(&input[first_chunk_size..]);
        }

        self.write_idx = end_position;
        self.available_bytes -= input_len;
    }
}

impl ThreadLoggingState {
    pub fn flush(&mut self, reservation: BufferReservation) {
        reservation.buffer_state.write_idx = reservation.write_idx;
        if let BufferReservationRef::New(new_buffer) = reservation.buffer_state {
            self.current_buffer = new_buffer;
        }
    }
}

struct BufferState {
    inner: NonNull<BufferStateInner>,
}

impl BufferState {
    pub fn try_create(total_size: usize) -> Option<BufferState> {
        if total_size > 67108864 {
            return None;
        }

        println!("allocating new buffer state @ {}", total_size);

        let mut addr = null_mut();

        let ret = unsafe {
            sceKernelMapNamedSystemFlexibleMemory(
                &mut addr,
                total_size as _,
                libc::PROT_WRITE | libc::PROT_READ,
                0,
                c"BufferState".as_ptr(),
            )
        };
        if ret != 0 {
            println!("sceKernelMapFlexibleMemory failed {:#x}", ret);
            return None;
        }

        let buffer_state = BufferState { inner: unsafe { NonNull::new_unchecked(addr as _) } };
        buffer_state.write_idx = 0;
        buffer_state.read_idx = 0;
        buffer_state.last_buffer = None;
        buffer_state.total_size = total_size;

        Some(buffer_state)
    }
}

impl Deref for BufferState {
    type Target = BufferStateInner;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl DerefMut for BufferState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

struct BufferStateInner {
    write_idx: usize,
    read_idx: usize,
    last_buffer: Option<NonNull<BufferStateInner>>,
    total_size: usize,

    bytes: [u8; 0],
}

impl BufferStateInner {
    fn bytes_len(&self) -> usize {
        self.total_size - size_of::<BufferStateInner>()
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        let len = self.bytes_len();
        unsafe { slice::from_raw_parts_mut(self.bytes.as_mut_ptr(), len) }
    }

    fn free_space(&self) -> usize {
        let read_idx = self.read_idx;
        let write_idx = self.write_idx;
        let size = self.bytes_len();

        if (write_idx >= read_idx) {
            size - (write_idx - read_idx)
        } else {
            read_idx - write_idx
        }
    }
}


extern "C" {
    fn sceKernelMapNamedSystemFlexibleMemory(
        addr: *mut *mut libc::c_void,
        len: isize,
        prot: libc::c_int,
        flags: libc::c_int,
        name: *const libc::c_char,
    ) -> libc::c_int;

    fn scePthreadSelf() -> u64;
}

