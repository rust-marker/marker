use crate::error::prelude::*;
use clap::Args;

#[derive(Args, Debug)]
pub struct SetupCommand {
    /// Automatically installs the required toolchain using rustup
    #[arg(long)]
    pub auto_install_toolchain: bool,

    /// Forward the current `RUSTFLAGS` value during the driver compilation
    #[arg(long)]
    pub forward_rust_flags: bool,
}

impl SetupCommand {
    pub(crate) fn run(self) -> Result {
        let rustc_flags = self
            .forward_rust_flags
            .then(|| std::env::var("RUSTFLAGS").ok())
            .flatten();

        crate::backend::driver::install_driver(self.auto_install_toolchain, rustc_flags)
    }
}
