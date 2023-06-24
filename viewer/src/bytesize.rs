/// byte size for 1 byte
const B: u64 = 1;
/// bytes size for 1 kilobyte
const KB: u64 = 1_000;
/// bytes size for 1 megabyte
const MB: u64 = 1_000_000;
/// bytes size for 1 gigabyte
const GB: u64 = 1_000_000_000;
/// bytes size for 1 terabyte
const TB: u64 = 1_000_000_000_000;
/// bytes size for 1 petabyte
const PB: u64 = 1_000_000_000_000_000;

/// bytes size for 1 kibibyte
const KIB: u64 = 1_024;
/// bytes size for 1 mebibyte
const MIB: u64 = 1_048_576;
/// bytes size for 1 gibibyte
const GIB: u64 = 1_073_741_824;
/// bytes size for 1 tebibyte
const TIB: u64 = 1_099_511_627_776;
/// bytes size for 1 pebibyte
const PIB: u64 = 1_125_899_906_842_624;

static UNITS: &str = "KMGTPE";
static UNITS_SI: &str = "kMGTPE";
static LN_KB: f64 = 6.931471806; // ln 1024
static LN_KIB: f64 = 6.907755279; // ln 1000

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ByteSize(u64);

pub trait ByteSizeExt {
    fn to_bytes(&self) -> ByteSize;
}

impl ByteSizeExt for u64 {
    fn to_bytes(&self) -> ByteSize {
        ByteSize(*self)
    }
}

impl From<ByteSize> for f32 {
    fn from(value: ByteSize) -> Self {
        value.as_f32()
    }
}

impl ByteSize {
    pub fn as_f32(&self) -> f32 {
        let unit = KIB;
        let unit_base = LN_KIB as f32;

        if self.0 < unit {
            self.0 as f32
        } else {
            let size = self.0 as f32;
            let exp = match (size.ln() / unit_base) as usize {
                e if e == 0 => 1,
                e => e,
            };

            (size / unit.pow(exp as u32) as f32)
        }
    }

    pub fn as_megabytes(&self) -> f32 {
        self.0 as f32 / MB as f32
    }
}

impl ToString for ByteSize {
    fn to_string(&self) -> String {
        let unit = KIB;
        let unit_base = LN_KIB;
        let unit_prefix = UNITS_SI.as_bytes();
        let unit_suffix = "B";

        if self.0 < unit {
            format!("{} B", self.0)
        } else {
            let size = self.0 as f64;
            let exp = match (size.ln() / unit_base) as usize {
                e if e == 0 => 1,
                e => e,
            };

            format!(
                "{:.1} {}{}",
                (size / unit.pow(exp as u32) as f64),
                unit_prefix[exp - 1] as char,
                unit_suffix
            )
        }
    }
}
