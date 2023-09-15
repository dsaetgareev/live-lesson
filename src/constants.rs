// This is read at compile time, please restart if you change this value.
pub static AUDIO_CODEC: &str = "opus"; // https://www.w3.org/TR/webcodecs-codec-registry/#audio-codec-registry
pub static VIDEO_CODEC: &str = "vp09.00.10.08"; // profile 0,level 1.0, bit depth 8,

// Commented out because it is not as fast as vp9.

// pub static VIDEO_CODEC: &str = "av01.0.01M.08";
// av01: AV1
// 0 profile: main profile
// 01 level: level2.1
// M tier: Main tier
// 08 bit depth = 8 bits

pub const AUDIO_CHANNELS: u32 = 1u32;
pub const AUDIO_SAMPLE_RATE: u32 = 48000u32;
pub const AUDIO_BITRATE: f64 = 50000f64;

// vga resolution
pub const VIDEO_HEIGHT: i32 = 480i32;
pub const VIDEO_WIDTH: i32 = 640i32;

// setting for screen sharing
pub const SCREEN_VIDEO_HEIGHT: i32 = 1080i32;
pub const SCREEN_VIDEO_WIDTH: i32 = 1920i32;



