use std::io;

use anyhow::{anyhow, Context, Result};
use cargo_preflight::{
    args::{CargoArguments, CargoSpawnedArguments, PreflightCommand},
    cargo::{build_artifact, get_host_target, get_metadata},
    harness::{AvionicsHarness, PanicCaught, PanicHang},
    shell::Shell,
    uom::si::SI,
    Vector3,
};
use preflight_impl::{
    uom::si::length::{meter, Length},
    Sensors,
};
use structopt::StructOpt;

fn main() -> io::Result<()> {
    let mut shell = Shell::new();

    let CargoSpawnedArguments::Preflight(args) = CargoSpawnedArguments::from_args();

    match args.command {
        PreflightCommand::Check { cargo } => {
            if let Err(e) = load_harness(&cargo, &mut shell) {
                shell.error(format!("{:#}", e))?
            } else {
                shell.status("Success", "built and loaded avionics harness successfully")?;
            }
        }
        PreflightCommand::Test {
            cargo,
            panic,
            display,
            sim,
        } => match load_harness(&cargo, &mut shell) {
            Err(e) => shell.error(format!("{:#}", e))?,
            Ok(harness) => match fuzz_harness(harness.setup_panic(panic)) {
                Err(e) => shell.error(format!("{:#}", e))?,
                Ok(false) => shell.error("harness failed to run")?,
                Ok(true) => shell.status("Finished", "TODO:")?,
            },
        },
        // PreflightCommand::Simulate { .. } => unimplemented!(),
    }

    Ok(())
}

fn fuzz_harness(mut harness: AvionicsHarness<PanicCaught>) -> Result<bool> {
    for _ in 0..10 {
        println!(
            "{:?}",
            Length::<SI<f64>, _>::new::<meter>(0.0f64)
                .into_format_args(meter, cargo_preflight::uom::fmt::DisplayStyle::Description),
        );

        let result = harness.guide(Sensors {
            altitude: Length::new::<meter>(0.0),
            linear_acceleration: Vector3::zero(),
            gravity_acceleration: Vector3::zero(),
            both_acceleration: Vector3::zero(),
            orientation: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            magnetic_field: Vector3::zero(),
        });
        dbg!(&result);
    }

    Ok(true)
}

fn load_harness(
    cargo_args: &CargoArguments,
    shell: &mut Shell,
) -> anyhow::Result<AvionicsHarness<PanicHang>> {
    let host_target = get_host_target()?;

    let metadata = get_metadata(&cargo_args).map_err(|e| match e {
        cargo_metadata::Error::CargoMetadata { stderr } => {
            anyhow!("{}", stderr.trim_start_matches("error: "))
        }
        e => anyhow!("{}", e),
    })?;

    let package = metadata
        .root_package()
        .context("could not find the root package for this workspace")?;

    let has_dylib_target = package
        .targets
        .iter()
        .any(|t| t.kind.contains(&"dylib".to_string()));

    if has_dylib_target {
        shell.warning(
            "the crate probably should not have a library target with a crate_type of 'dylib'",
        )?;
        shell.note("this will be added automatically when this command is run. crate_type should be `staticlib` or `cdylib`")?;
    }

    match build_artifact(&cargo_args, &host_target, package)? {
        None => Err(anyhow!(
            "the cargo build did not produce any valid artifacts"
        )),
        Some(artifact_file) => {
            shell.status("Loading", artifact_file.to_string_lossy())?;

            let harness = AvionicsHarness::load(&artifact_file)
                .context("failed to load built shared library")?;

            if let Some(harness) = harness {
                Ok(harness)
            } else {
                Err(anyhow!("the dylib was not setup using the `#[avionics_harness]` macro or is using an out of date dependency to preflight_impl"))
            }
        }
    }
}
