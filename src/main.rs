/// This file is heavily based on https://github.com/prefix-dev/rip/blob/b7ea9397d969beae682e1c59b5a899d24876c4ac/crates/rip_bin/src/main.rs
/// Which is licensed under the BSD-3-Clause license.
use indoc::formatdoc;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use clap::Parser;
use itertools::Itertools;
use miette::{Context, IntoDiagnostic};
use tracing_subscriber::filter::Directive;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use url::Url;

use rattler_installs_packages::tags::WheelTags;
use rattler_installs_packages::{
    normalize_index_url, resolve, Pep508EnvMakers, PinnedPackage, Requirement,
};

use indicatif::{MultiProgress, ProgressDrawTarget};
use std::io;
use std::sync::OnceLock;
use tracing_subscriber::fmt::MakeWriter;

/// Returns a global instance of [`indicatif::MultiProgress`].
///
/// Although you can always create an instance yourself any logging will interrupt pending
/// progressbars. To fix this issue, logging has been configured in such a way to it will not
/// interfere if you use the [`indicatif::MultiProgress`] returning by this function.
pub fn global_multi_progress() -> MultiProgress {
    static GLOBAL_MP: OnceLock<MultiProgress> = OnceLock::new();
    GLOBAL_MP
        .get_or_init(|| {
            let mp = MultiProgress::new();
            mp.set_draw_target(ProgressDrawTarget::stderr_with_hz(20));
            mp
        })
        .clone()
}

#[derive(Clone)]
pub struct IndicatifWriter {
    progress_bars: MultiProgress,
}

impl IndicatifWriter {
    pub fn new(pb: MultiProgress) -> Self {
        Self { progress_bars: pb }
    }
}

impl io::Write for IndicatifWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.progress_bars.suspend(|| io::stderr().write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.progress_bars.suspend(|| io::stderr().flush())
    }
}

impl<'a> MakeWriter<'a> for IndicatifWriter {
    type Writer = IndicatifWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(num_args=1.., required=true)]
    specs: Vec<Requirement>,

    /// Base URL of the Python Package Index (default <https://pypi.org/simple>). This should point
    /// to a repository compliant with PEP 503 (the simple repository API).
    #[clap(default_value = "https://pypi.org/simple/", long)]
    index_url: Url,

    #[clap(short)]
    verbose: bool,

    #[clap(default_value = "BUCK", long)]
    output_file: String,
}

fn gen_buck_file_content(packages: &[PinnedPackage]) -> miette::Result<String> {
    // Here is what we want to generate roughly speaking
    //
    // remote_file(
    //     name = "requests-download",
    //     url = "https://files.pythonhosted.org/packages/51/bd/23c926cd341ea6b7dd0b2a00aba99ae0f828be89d72b2190f27c11d4b7fb/requests-2.22.0-py2.py3-none-any.whl",
    //     sha1 = "e1fc28120002395fe1f2da9aacea4e15a449d9ee",
    //     out = "requests-2.22.0-py2.py3-none-any.whl",
    // )

    // remote_file(
    //     name = "chardet-download",
    //     url = "https://files.pythonhosted.org/packages/38/6f/f5fbc992a329ee4e0f288c1fe0e2ad9485ed064cac731ed2fe47dcc38cbf/chardet-5.2.0-py3-none-any.whl",
    //     sha1 = "2facc0387556aa8a2956ef682d49fc3eae56d30a",
    //     out = "chardet-5.2.0-py3-none-any.whl",
    // )

    // prebuilt_python_library(
    //     name = "requests",
    //     binary_src = ":requests-download",
    //     # deps = [":chardet"],
    //     visibility = ["PUBLIC"],
    // )

    // prebuilt_python_library(
    //     name = "chardet",
    //     binary_src = ":chardet-download",
    // )
    Ok(packages
        .iter()
        .map(|package| {
            let mut artifacts = package.artifacts.clone();
            artifacts.sort_by(|a, b| {
                // The idea of this sort is to prefer built wheels over the source distributions.
                // However there are many wheels with just the source distribution (i.e. no native extensions).
                if a.filename.as_wheel().is_none() {
                    return std::cmp::Ordering::Greater;
                }
                if b.filename.as_wheel().is_none() {
                    return std::cmp::Ordering::Less;
                }
                return std::cmp::Ordering::Equal;
            });
            let url = artifacts.first().unwrap().url.as_str();
            let (url_without_sha, sha) = url.split_once("#sha256=").unwrap();
            let filename = url_without_sha.split("/").last().unwrap();
            return formatdoc!(
                r#"
remote_file(
    name = "{name}-download",
    url = "{url}",
    sha256 = "{sha}",
    out = "{out}",
)

prebuilt_python_library(
    name = "{name}",
    binary_src = ":{name}-download",
)
"#,
                name = package.name,
                url = url_without_sha,
                sha = sha,
                out = filename,
            );
        })
        .join("\n"))
}

async fn actual_main() -> miette::Result<()> {
    let args = Args::parse();

    // Setup tracing subscriber
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(IndicatifWriter::new(global_multi_progress())))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| get_default_env_filter(args.verbose)),
        )
        .init();

    // Determine cache directory
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| miette::miette!("failed to determine cache directory"))?
        .join("rattler/pypi");
    tracing::info!("cache directory: {}", cache_dir.display());

    // Construct a package database
    let package_db = rattler_installs_packages::PackageDb::new(
        Default::default(),
        &[normalize_index_url(args.index_url.clone())],
        cache_dir,
    )
    .into_diagnostic()
    .wrap_err_with(|| {
        format!(
            "failed to construct package database for index {}",
            args.index_url
        )
    })?;

    // Determine the environment markers for the current machine
    let env_markers = Pep508EnvMakers::from_env()
        .await
        .into_diagnostic()
        .wrap_err_with(|| {
            "failed to determine environment markers for the current machine (could not run Python)"
        })?;
    tracing::debug!(
        "extracted the following environment markers from the system python interpreter:\n{:#?}",
        env_markers
    );

    let compatible_tags = WheelTags::from_env().await.into_diagnostic()?;
    tracing::debug!(
        "extracted the following compatible wheel tags from the system python interpreter: {}",
        compatible_tags.tags().format(", ")
    );

    // Solve the environment
    let blueprint = match resolve(
        &package_db,
        &args.specs,
        &env_markers,
        Some(&compatible_tags),
        HashMap::default(),
        HashMap::default(),
    )
    .await
    {
        Ok(blueprint) => blueprint,
        Err(err) => miette::bail!("Could not solve for the requested requirements:\n{err}"),
    };

    // Output the selected versions
    println!("{}:", console::style("Resolved environment").bold());
    for spec in args.specs.iter() {
        println!("- {}", spec);
    }

    // Generate the buck file
    let buck_file_content = gen_buck_file_content(blueprint.as_slice())?;
    let mut output = File::create(args.output_file).into_diagnostic()?;
    writeln!(output, "{}", buck_file_content).into_diagnostic()?;

    println!();
    let mut tabbed_stdout = tabwriter::TabWriter::new(std::io::stdout());
    writeln!(
        tabbed_stdout,
        "{}\t{}",
        console::style("Name").bold(),
        console::style("Version").bold()
    )
    .into_diagnostic()?;
    for pinned_package in blueprint.into_iter().sorted_by(|a, b| a.name.cmp(&b.name)) {
        write!(tabbed_stdout, "{name}", name = pinned_package.name.as_str()).into_diagnostic()?;
        if !pinned_package.extras.is_empty() {
            write!(
                tabbed_stdout,
                "[{}]",
                pinned_package.extras.iter().map(|e| e.as_str()).join(",")
            )
            .into_diagnostic()?;
        }
        writeln!(
            tabbed_stdout,
            "\t{version}",
            version = pinned_package.version
        )
        .into_diagnostic()?;
    }
    tabbed_stdout.flush().into_diagnostic()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = actual_main().await {
        eprintln!("{e:?}");
    }
}

/// Constructs a default [`EnvFilter`] that is used when the user did not specify a custom RUST_LOG.
pub fn get_default_env_filter(verbose: bool) -> EnvFilter {
    let mut result = EnvFilter::new("rip=info")
        .add_directive(Directive::from_str("rattler_installs_packages=info").unwrap());

    if verbose {
        result = result.add_directive(Directive::from_str("resolvo=info").unwrap());
    }

    result
}
