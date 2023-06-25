/// bytes size for 1 kibibyte
const KIB: u64 = 1_024;
/// bytes size for 1 mebibyte
const MIB: u64 = 1_048_576;

const UNITS_SI: &str = "kMGTPE";
static LN_KIB: f64 = 6.907_755_279; // ln 1000

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

impl From<ByteSize> for f64 {
    fn from(value: ByteSize) -> Self {
        value.as_f64()
    }
}

impl ByteSize {
    #[must_use]
    pub fn as_f64(&self) -> f64 {
        let unit = KIB;
        let unit_base = LN_KIB;

        if self.0 < unit {
            self.0 as f64
        } else {
            let size = self.0 as f64;
            let exp = match (size.ln() / unit_base) as usize {
                e if e == 0 => 1,
                e => e,
            };

            size / unit.pow(exp as u32) as f64
        }
    }

    #[must_use]
    pub fn as_megabytes(&self) -> f64 {
        self.0 as f64 / MIB as f64
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

#[cfg(test)]
mod tests {
    use crate::bytesize::ByteSizeExt;
    use float_cmp::assert_approx_eq;

    #[test]
    fn test_megabytes_conversion() {
        let size = 3_221_225_472.to_bytes();

        assert_approx_eq!(f64, size.as_f64(), 3.0);
        assert_approx_eq!(f64, size.as_megabytes(), 3072.0);
        assert_eq!(size.to_string(), "3.0 GB".to_string());
    }
}
