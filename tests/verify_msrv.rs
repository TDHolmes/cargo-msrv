use cargo_msrv::exit_code::ExitCode;
use parameterized::parameterized;
use rust_releases::{semver, Release};
use std::process::Command;

use common::run_verify;

mod common;

#[parameterized(
    folder = {
        "1.35.0",
        "1.36.0",
        "1.56.0-edition-2018",
        "1.56.0-edition-2021",
    }
)]
fn verify(folder: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_verify(
        with_args,
        vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 37, 0)),
            Release::new_stable(semver::Version::new(1, 36, 0)),
            Release::new_stable(semver::Version::new(1, 35, 0)),
            Release::new_stable(semver::Version::new(1, 34, 0)),
        ],
    );

    assert!(result.is_ok());
}

#[parameterized(
    folder = {
        "1.37.0",
        "1.38.0",
    }
)]
fn verify_failed_no_msrv_specified(folder: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("features")
        .join(folder);
    let with_args = vec!["cargo-msrv", "--path", folder.to_str().unwrap()];

    let result = run_verify(
        with_args,
        vec![
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 37, 0)),
            Release::new_stable(semver::Version::new(1, 36, 0)),
            Release::new_stable(semver::Version::new(1, 35, 0)),
            Release::new_stable(semver::Version::new(1, 34, 0)),
        ],
    );

    assert!(result.is_err());
}

#[test]
fn verify_success_zero_exit_code() {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");
    let test_subject = [cargo_msrv_dir, "features", "1.56.0-edition-2021"].join("/");

    let mut process = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--verify",
            "--path",
            &test_subject,
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Success;

    assert_eq!(exit_code, Into::<i32>::into(expected))
}

#[test]
fn verify_failure_non_zero_exit_code() {
    let cargo_msrv_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_msrv_manifest = [cargo_msrv_dir, "Cargo.toml"].join("/");

    let test_subject = [cargo_msrv_dir, "features", "unbuildable-with-msrv"].join("/");

    let mut process = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            &cargo_msrv_manifest,
            "--",
            "--verify",
            "--path",
            &test_subject,
        ])
        .spawn()
        .expect("Unable to spawn cargo-msrv via cargo in test");

    let exit_status = process
        .wait()
        .expect("Waiting for process failed during test");

    let exit_code = exit_status.code().unwrap();
    let expected = ExitCode::Failure;

    assert_eq!(exit_code, Into::<i32>::into(expected))
}
