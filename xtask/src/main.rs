//! Repository tasks: `cargo xtask ci` runs the local gates, `cargo xtask renderer-check`
//! enforces the software-renderer dependency, `cargo xtask package` builds the installers,
//! `cargo xtask release-check` and `cargo xtask changelog` prepare a release, and
//! `cargo xtask widget-idle` measures the idle footprint on Windows. Arguments are read from
//! `std::env::args()` to keep this small.

mod idle;
mod package;
mod release;
mod renderer;

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use anyhow::{Result, bail};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["ci"] => ci(),
        ["renderer-check"] => renderer::run(),
        ["package"] => package::run(),
        ["release-check", version] => release::run_check(version),
        ["changelog", version] => release::run_changelog(version),
        ["widget-idle"] => idle::run(None),
        ["widget-idle", "--minutes", minutes] => idle::run(Some(minutes)),
        _ => {
            eprintln!(
                "usage: cargo xtask <ci | renderer-check | package | release-check <version> | \
                 changelog <version> | widget-idle [--minutes N]>"
            );
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

/// The repository root, from this crate's manifest directory.
pub(crate) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
}

/// One CI step: a program and its arguments.
struct Step {
    program: &'static str,
    args: &'static [&'static str],
}

const fn step(program: &'static str, args: &'static [&'static str]) -> Step {
    Step { program, args }
}

const STEPS: &[Step] = &[
    step("cargo", &["fmt", "--all", "--check"]),
    step(
        "cargo",
        &[
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    ),
    step("cargo", &["nextest", "run", "--all-features"]),
    // The snapshot feature stays off by default; this build keeps its code compiling.
    step("cargo", &["build", "--features", "snapshot"]),
    step("cargo", &["xtask", "renderer-check"]),
    step("cargo", &["deny", "check"]),
];

fn ci() -> Result<()> {
    for (index, step) in STEPS.iter().enumerate() {
        let number = index + 1;
        println!("$ {} {}", step.program, step.args.join(" "));
        let status = Command::new(step.program)
            .args(step.args)
            .current_dir(root())
            .status()?;
        if !status.success() {
            println!(
                "xtask ci: FAILED at step {number} ({} {})",
                step.program,
                step.args.join(" ")
            );
            bail!(
                "`{} {}` exited with {status}",
                step.program,
                step.args.join(" ")
            );
        }
    }
    println!("xtask ci: all {} steps passed", STEPS.len());
    Ok(())
}
