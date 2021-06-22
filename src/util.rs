use anyhow::Result;
use std::{fmt, process::Output};

pub trait OutputEx {
    fn exit_on_fail(self) -> Result<()>;
}

impl OutputEx for Output {
    fn exit_on_fail(self) -> Result<()> {
        if !self.status.success() {
            let err = String::from_utf8(self.stderr)?;
            eprintln!("{}", err);
            std::process::exit(self.status.code().unwrap_or(1));
        }

        Ok(())
    }
}

pub fn format_bytes(bytes: &[u8]) -> impl fmt::Debug + '_ {
    struct Bytes<'a>(&'a [u8]);

    impl fmt::Debug for Bytes<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("b\"")?;

            for &b in self.0 {
                if b.is_ascii_graphic() {
                    write!(f, "{}", b as char)?;
                } else {
                    write!(f, "\\x{:02x}", b)?;
                }
            }

            f.write_str("\"")?;
            Ok(())
        }
    }

    Bytes(bytes)
}
