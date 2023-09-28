use super::check::CheckCommand;
use crate::backend::driver::DriverVersionInfo;
use crate::config::Config;
use crate::error::prelude::*;
use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct TestSetupCommand {
    #[clap(flatten)]
    check: CheckCommand,
}

impl TestSetupCommand {
    pub(crate) fn run(self, config: Option<Config>) -> Result {
        let lints = self.check.compile_lints(config)?;

        for (name, value) in lints.info.env {
            println!("env:{name}={value}");
        }

        let info = DriverVersionInfo::try_from_toolchain(
            &lints.backend_conf.toolchain,
            &lints.backend_conf.marker_dir.join("Cargo.toml"),
        )?;

        println!("info:toolchain={}", info.toolchain);
        println!("info:marker-api={}", info.api_version);

        Ok(())
    }
}
