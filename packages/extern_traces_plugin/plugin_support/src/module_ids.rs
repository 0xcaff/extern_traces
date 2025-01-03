use strum::FromRepr;

#[derive(FromRepr, Debug)]
#[repr(u16)]
pub enum ModuleId {
    INVALID = 0x0000,
    FIBER = 0x0006,
    ULT = 0x0007,
    NGS2 = 0x000b,
    XML = 0x0017,
    NP_UTILITY = 0x0019,
    VOICE = 0x001a,
    VOICEQOS = 0x001b,
    NP_MATCHING2 = 0x001c,
    NP_SCORE_RANKING = 0x001e,
    RUDP = 0x0021,
    NP_TUS = 0x002c,
    FACE = 0x0038,
    SMART = 0x0039,
    GAME_LIVE_STREAMING = 0x0081,
    COMPANION_UTIL = 0x0082,
    PLAYGO = 0x0083,
    FONT = 0x0084,
    VIDEO_RECORDING = 0x0085,
    S3DCONVERSION = 0x0086,
    AUDIODEC = 0x0088,
    JPEG_DEC = 0x008a,
    JPEG_ENC = 0x008b,
    PNG_DEC = 0x008c,
    PNG_ENC = 0x008d,
    VIDEODEC = 0x008e,
    MOVE = 0x008f,
    PAD_TRACKER = 0x0091,
    DEPTH = 0x0092,
    HAND = 0x0093,
    LIBIME = 0x0095,
    IME_DIALOG = 0x0096,
    NP_PARTY = 0x0097,
    FONT_FT = 0x0098,
    FREETYPE_OT = 0x0099,
    FREETYPE_OL = 0x009a,
    FREETYPE_OPT_OL = 0x009b,
    SCREEN_SHOT = 0x009c,
    NP_AUTH = 0x009d,
    SULPHA = 0x009f,
    SAVE_DATA_DIALOG = 0x00a0,
    INVITATION_DIALOG = 0x00a2,
    DEBUG_KEYBOARD = 0x00a3,
    MESSAGE_DIALOG = 0x00a4,
    AV_PLAYER = 0x00a5,
    CONTENT_EXPORT = 0x00a6,
    AUDIO_3D = 0x00a7,
    NP_COMMERCE = 0x00a8,
    MOUSE = 0x00a9,
    COMPANION_HTTPD = 0x00aa,
    WEB_BROWSER_DIALOG = 0x00ab,
    ERROR_DIALOG = 0x00ac,
    NP_TROPHY = 0x00ad,
    RESERVED30 = 0x00ae,
    RESERVED31 = 0x00af,
    NP_SNS_FACEBOOK = 0x00b0,
    MOVE_TRACKER = 0x00b1,
    NP_PROFILE_DIALOG = 0x00b2,
    NP_FRIEND_LIST_DIALOG = 0x00b3,
    APP_CONTENT = 0x00b4,
    NP_SIGNALING = 0x00b5,
    REMOTE_PLAY = 0x00b6,
    USBD = 0x00b7,
    GAME_CUSTOM_DATA_DIALOG = 0x00b8,
    RESERVED0 = 0x00b9,
    RESERVED1 = 0x00ba,
    RESERVED2 = 0x00bb,
    M4AAC_ENC = 0x00bc,
    AUDIODEC_CPU = 0x00bd,
    RESERVED32 = 0x00be,
    RESERVED33 = 0x00c0,
    RESERVED3 = 0x00c1,
    RESERVED4 = 0x00c2,
    RESERVED5 = 0x00c3,
    RESERVED6 = 0x00c4,
    ZLIB = 0x00c5,
    RESERVED8 = 0x00c6,
    CONTENT_SEARCH = 0x00c7,
    RESERVED9 = 0x00c8,
    RESERVED34 = 0x00c9,
    DECI4H = 0x00ca,
    HEAD_TRACKER = 0x00cb,
    RESERVED11 = 0x00cc,
    RESERVED12 = 0x00cd,
    SYSTEM_GESTURE = 0x00ce,
    VIDEODEC2 = 0x00cf,
    RESERVED14 = 0x00d0,
    AT9_ENC = 0x00d1,
    CONVERT_KEYCODE = 0x00d2,
    SHARE_PLAY = 0x00d3,
    HMD = 0x00d4,
    RESERVED18 = 0x00d5,
    RESERVED16 = 0x00d6,
    RESERVED17 = 0x00d7,
    FACE_TRACKER = 0x00d8,
    HAND_TRACKER = 0x00d9,
    RESERVED19 = 0x00da,
    RESERVED20 = 0x00dc,
    RESERVED21 = 0x00dd,
    RESERVED22 = 0x00de,
    RESERVED23 = 0x00df,
    RESERVED24 = 0x00e0,
    AUDIODEC_CPU_HEVAG = 0x00e1,
    LOGIN_DIALOG = 0x00e2,
    LOGIN_SERVICE = 0x00e3,
    SIGNIN_DIALOG = 0x00e4,
    RESERVED35 = 0x00e5,
    RESERVED25 = 0x00e6,
    JSON2 = 0x00e7,
    AUDIO_LATENCY_ESTIMATION = 0x00e8,
    RESERVED26 = 0x00e9,
    RESERVED27 = 0x00ea,
    HMD_SETUP_DIALOG = 0x00eb,
    RESERVED28 = 0x00ec,
    VR_TRACKER = 0x00ed,
    CONTENT_DELETE = 0x00ee,
    IME_BACKEND = 0x00ef,
    NET_CTL_AP_DIALOG = 0x00f0,
    PLAYGO_DIALOG = 0x00f1,
    SOCIAL_SCREEN = 0x00f2,
    EDIT_MP4 = 0x00f3,
    RESERVED37 = 0x00f5,
    TEXT_TO_SPEECH = 0x00f6,
    RESERVED38 = 0x00f8,
    RESERVED39 = 0x00f9,
    RESERVED40 = 0x00fa,
    BLUETOOTH_HID = 0x00fb,
    RESERVED41 = 0x00fc,
    VR_SERVICE_DIALOG = 0x00fd,
    JOB_MANAGER = 0x00fe,
    RESERVED42 = 0x00ff,
    SOCIAL_SCREEN_DIALOG = 0x0100,
    RESERVED43 = 0x0101,
    NP_TOOLKIT2 = 0x0102,
    RESERVED44 = 0x0103,
    RESERVED45 = 0x0104,
    RESERVED46 = 0x00f7,
}
