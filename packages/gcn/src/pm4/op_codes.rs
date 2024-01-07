use anyhow::format_err;
use strum::FromRepr;

#[derive(Debug, FromRepr)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum OpCode {
    NOP = 0x10,
    SET_BASE = 0x11,
    CLEAR_STATE = 0x12,
    INDEX_BUFFER_SIZE = 0x13,
    DISPATCH_DIRECT = 0x15,
    DISPATCH_INDIRECT = 0x16,
    ATOMIC_MEM = 0x1E,
    OCCLUSION_QUERY = 0x1F, /* GFX7+ */
    SET_PREDICATION = 0x20,
    COND_EXEC = 0x22,
    PRED_EXEC = 0x23,
    DRAW_INDIRECT = 0x24,
    DRAW_INDEX_INDIRECT = 0x25,
    INDEX_BASE = 0x26,
    DRAW_INDEX_2 = 0x27,
    CONTEXT_CONTROL = 0x28,
    INDEX_TYPE = 0x2A, /* GFX6-8 */
    DRAW_INDIRECT_MULTI = 0x2C,
    DRAW_INDEX_AUTO = 0x2D,
    DRAW_INDEX_IMMD = 0x2E, /* GFX6 only */
    NUM_INSTANCES = 0x2F,
    DRAW_INDEX_MULTI_AUTO = 0x30,
    INDIRECT_BUFFER_SI = 0x32, /* GFX6 only */
    INDIRECT_BUFFER_CONST = 0x33,
    STRMOUT_BUFFER_UPDATE = 0x34,
    DRAW_INDEX_OFFSET_2 = 0x35,
    WRITE_DATA = 0x37,
    DRAW_INDEX_INDIRECT_MULTI = 0x38,
    MEM_SEMAPHORE = 0x39,
    MPEG_INDEX = 0x3A, /* GFX6 only */
    WAIT_REG_MEM = 0x3C,
    MEM_WRITE = 0x3D,           /* GFX6 only */
    INDIRECT_BUFFER_CIK = 0x3F, /* GFX7+ */
    COPY_DATA = 0x40,
    CP_DMA = 0x41, /* GFX6 only */
    PFP_SYNC_ME = 0x42,
    SURFACE_SYNC = 0x43,  /* deprecated on GFX7, use ACQUIRE_MEM */
    ME_INITIALIZE = 0x44, /* GFX6 only */
    COND_WRITE = 0x45,
    EVENT_WRITE = 0x46,
    EVENT_WRITE_EOP = 0x47,              /* GFX6-8 */
    EVENT_WRITE_EOS = 0x48,              /* GFX6-8, breaks CP DMA */
    RELEASE_MEM = 0x49,                  /* GFX9+ [any ring] or GFX8 [compute ring only] */
    DMA_DATA = 0x50,                     /* GFX7+ */
    DISPATCH_MESH_INDIRECT_MULTI = 0x4C, /* Indirect mesh shader only dispatch [GFX only], GFX10.3+ */
    DISPATCH_TASKMESH_GFX = 0x4D,        /* Task + mesh shader dispatch [GFX side], GFX10.3+ */
    CONTEXT_REG_RMW = 0x51, /* older firmware versions on older chips don't have this */
    ONE_REG_WRITE = 0x57,   /* GFX6 only */
    ACQUIRE_MEM = 0x58,     /* GFX7+ */
    REWIND = 0x59,          /* GFX8+ [any ring] or GFX7 [compute ring only] */
    LOAD_UCONFIG_REG = 0x5E, /* GFX7+ */
    LOAD_SH_REG = 0x5F,
    LOAD_CONTEXT_REG = 0x61,
    LOAD_SH_REG_INDEX = 0x63, /* GFX8+ */
    SET_CONFIG_REG = 0x68,
    SET_CONTEXT_REG = 0x69,
    SET_SH_REG = 0x76,
    SET_SH_REG_OFFSET = 0x77,
    SET_UCONFIG_REG = 0x79,       /* GFX7+ */
    SET_UCONFIG_REG_INDEX = 0x7A, /* new for GFX9, CP ucode version >= 26 */
    LOAD_CONST_RAM = 0x80,
    WRITE_CONST_RAM = 0x81,
    DUMP_CONST_RAM = 0x83,
    INCREMENT_CE_COUNTER = 0x84,
    INCREMENT_DE_COUNTER = 0x85,
    WAIT_ON_CE_COUNTER = 0x86,
    SET_SH_REG_INDEX = 0x9B,
    LOAD_CONTEXT_REG_INDEX = 0x9F,               /* GFX8+ */
    DISPATCH_TASK_STATE_INIT = 0xA9, /* Tells the HW about the task control buffer, GFX10.3+ */
    DISPATCH_TASKMESH_DIRECT_ACE = 0xAA, /* Direct task + mesh shader dispatch [ACE side], GFX10.3+ */
    DISPATCH_TASKMESH_INDIRECT_MULTI_ACE = 0xAD, /* Indirect task + mesh shader dispatch [ACE side], GFX10.3+ */
    EVENT_WRITE_ZPASS = 0xB1,                    /* GFX11+ & PFP version >= 1458 */
    SET_CONTEXT_REG_PAIRS = 0xB8,                /* GFX11+, PFP version >= 1448 */
    SET_CONTEXT_REG_PAIRS_PACKED = 0xB9,         /* GFX11+, PFP version >= 1448 */
    SET_SH_REG_PAIRS = 0xBA,                     /* GFX11+, PFP version >= 1448 */
    SET_SH_REG_PAIRS_PACKED = 0xBB,              /* GFX11+, PFP version >= 1448 */
    SET_SH_REG_PAIRS_PACKED_N = 0xBD,            /* GFX11+, PFP version >= 1448 */
}

impl OpCode {
    pub fn from_op(value: u8) -> Result<OpCode, anyhow::Error> {
        Ok(Self::from_repr(value).ok_or_else(|| format_err!("failed to parse op code {}", value))?)
    }
}