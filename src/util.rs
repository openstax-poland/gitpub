use anyhow::Result;
use std::process::Output;

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
