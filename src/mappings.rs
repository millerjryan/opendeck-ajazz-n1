use mirajazz::{
    device::DeviceQuery,
    types::{HidDeviceInfo, ImageFormat, ImageMirroring, ImageMode, ImageRotation},
};

// Must be unique between all the plugins, 2 characters long and match `DeviceNamespace` field in `manifest.json`
pub const DEVICE_NAMESPACE: &str = "N1";

pub const AJAZZ_VID: u16 = 0x0300;
pub const MIRABOX_VID: u16 = 0x6603;
pub const VSDINSIDE_VID: u16 = 0x5548;
pub const N1_PID: u16 = 0x3007;
pub const N1MIR_PID: u16 = 0x1000;
pub const N1VSD_PID: u16 = 0x1002;
pub const N1VSD_ALT_PID: u16 = 0x1000;

pub const N1_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID);
pub const N1MIR_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, MIRABOX_VID, N1MIR_PID);
pub const N1VSD_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, VSDINSIDE_VID, N1VSD_PID);
pub const N1VSD_ALT_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, VSDINSIDE_VID, N1VSD_ALT_PID);

pub const QUERIES: [DeviceQuery; 4] = [
    N1_QUERY,
    N1MIR_QUERY,
    N1VSD_QUERY,
    N1VSD_ALT_QUERY,
];

/// Returns correct image format for device kind and key
pub fn get_image_format_for_key(_kind: &Kind, key: u8) -> ImageFormat {
    // N1 uses different format: no rotation, no mirroring
    // With 3×6 horizontal layout:
    // Keys 5, 11, 17 are the right-column LCD screens (64×64)
    // All other keys are main buttons (96×96)
    let size = if key == 5 || key == 11 || key == 17 { (64, 64) } else { (96, 96) };
    ImageFormat {
        mode: ImageMode::JPEG,
        size,
        rotation: ImageRotation::Rot270,
        mirror: ImageMirroring::None,
    }
}

#[derive(Debug, Clone)]
pub enum Kind {
    N1,
    MiraboxN1,
    VsdInsideN1,
}

impl Kind {
    /// Matches devices VID+PID pairs to correct kinds
    pub fn from_vid_pid(vid: u16, pid: u16) -> Option<Self> {
        match vid {
            AJAZZ_VID => match pid {
                N1_PID => Some(Kind::N1),
                _ => None
            },
            MIRABOX_VID => match pid {
                N1MIR_PID => Some(Kind::MiraboxN1),
                _ => None
            },
            VSDINSIDE_VID => match pid {
                N1VSD_PID | N1VSD_ALT_PID => Some(Kind::VsdInsideN1),
                _ => None,
            }
            _ => None
        }
    }

    /// Returns protocol version for device
    pub fn protocol_version(&self) -> usize {
        3 // N1 uses protocol v3
    }

    /// Returns (rows, cols) layout for this device type
    pub fn layout(&self) -> (usize, usize) {
        // N1: 3 rows × 6 cols = 18 keys (horizontal orientation)
        // Arranged to match physical layout (device rotated 90° CW):
        // Col:    0         1         2        3        4        5
        // Row 0: [KEY_13] [KEY_10]  [KEY_7]  [KEY_4]  [KEY_1]  [LCD_16]
        // Row 1: [KEY_14] [KEY_11]  [KEY_8]  [KEY_5]  [KEY_2]  [LCD_17]
        // Row 2: [KEY_15] [KEY_12]  [KEY_9]  [KEY_6]  [KEY_3]  [LCD_18]
        // Note: The 2 top normal buttons (inputs 30, 31) are NOT shown in GUI
        // (They work but have no display, so we hide them to avoid confusion)
        (3, 6)
    }

    /// Returns number of display keys for this device
    pub fn key_count(&self) -> usize {
        // N1 has 18 display keys total (15 main + 3 top LCDs)
        // Note: 2 normal buttons (inputs 30, 31) are NOT counted as they have no display
        18
    }

    /// Returns number of encoders (dials/knobs) for this device
    /// N1 has 1 encoder (the dial)
    pub fn encoder_count(&self) -> usize {
        1
    }

    /// Returns human-readable device name
    pub fn human_name(&self) -> String {
        match &self {
            Self::N1 => "Ajazz N1",
            Self::MiraboxN1 => "Mirabox N1",
            Self::VsdInsideN1 => "VSDInside N1",
        }.to_string()
    }


}

#[derive(Debug, Clone)]
pub struct CandidateDevice {
    pub id: String,
    pub dev: HidDeviceInfo,
    pub kind: Kind,
}
