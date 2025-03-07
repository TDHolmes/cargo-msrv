use std::convert::TryFrom;
use std::path::PathBuf;

use rust_releases::{semver, Release, ReleaseIndex};
use toml_edit::Document;

use crate::check::{check_toolchain, Outcome};
use crate::config::{Config, ModeIntent};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::paths::crate_root_folder;
use crate::reporter::Output;

// NB: only public for integration testing
pub fn run_verify_msrv_action<R: Output>(
    config: &Config,
    reporter: &R,
    release_index: &ReleaseIndex,
) -> TResult<()> {
    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml.clone()),
    })?;

    let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    let manifest = CargoManifest::try_from(manifest)?;

    let version = manifest
        .minimum_rust_version()
        .ok_or_else(|| CargoMSRVError::NoMSRVKeyInCargoToml(cargo_toml.to_owned()))?;
    let version = version.try_to_semver(release_index.releases().iter().map(Release::version))?;

    let cmd = config.check_command_string();
    reporter.mode(ModeIntent::VerifyMSRV);
    let status = check_toolchain(version, config, reporter)?;
    report_verify_completion(reporter, &status, &cmd);

    if status.is_success() {
        Ok(())
    } else {
        Err(CargoMSRVError::SubCommandVerify(Error::VerifyFailed {
            expected_msrv: version.to_owned(),
            manifest: cargo_toml,
        }))
    }
}

fn report_verify_completion(output: &impl Output, status: &Outcome, cmd: &str) {
    if status.is_success() {
        output.finish_success(ModeIntent::VerifyMSRV, Some(status.version()));
    } else {
        output.finish_failure(ModeIntent::VerifyMSRV, Some(cmd));
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Crate source was found to be incompatible with its MSRV '{expected_msrv}', as defined in '{manifest}'"
    )]
    VerifyFailed {
        expected_msrv: semver::Version,
        manifest: PathBuf,
    },
}
