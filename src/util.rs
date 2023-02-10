use anyhow::Result;
use std::{fmt, process::Command};

pub trait CommandEx {
    /// Wait for command to complete successfully
    fn wait_or_fail(&mut self) -> Result<()>;
}

impl CommandEx for Command {
    fn wait_or_fail(&mut self) -> Result<()> {
        if crate::output::is_verbose() {
            let status = self.status()?;
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        } else {
            let out = self.output()?;
            if !out.status.success() {
                let err = String::from_utf8(out.stderr)?;
                eprintln!("{err}");
                std::process::exit(out.status.code().unwrap_or(1));
            }
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
                    write!(f, "\\x{b:02x}")?;
                }
            }

            f.write_str("\"")?;
            Ok(())
        }
    }

    Bytes(bytes)
}
