use std::path::PathBuf;

use clap::{Parser, Subcommand};
use jupyter_protocol::JupyterKernelspec;
use jupyter_zmq_client::KernelspecDir;

#[derive(Parser)]
#[command(
    name = "yoshi",
    version,
    about = "A native, GPU-rendered Jupyter notebook app"
)]
struct Cli {
    /// Notebook file to open
    notebook: Option<PathBuf>,

    /// Run the kernel round-trip check without a window (used by CI)
    #[arg(long, hide = true)]
    headless: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect Jupyter kernels
    Kernels {
        #[command(subcommand)]
        command: KernelsCommand,
    },
}

#[derive(Subcommand)]
enum KernelsCommand {
    /// List kernelspecs discovered on disk
    List,
}

pub enum Invocation {
    Gui(Option<PathBuf>),
    Headless,
    KernelsList,
}

pub fn parse() -> Invocation {
    let cli = Cli::parse();
    if cli.headless {
        return Invocation::Headless;
    }
    match cli.command {
        Some(Command::Kernels {
            command: KernelsCommand::List,
        }) => Invocation::KernelsList,
        None => Invocation::Gui(cli.notebook),
    }
}

pub fn list_kernels() -> Vec<KernelspecDir> {
    // venv kernelspecs (sys.prefix installs) first, matching the app's python
    // resolution; disk reads only — never shell out to jupyter (PRD, startup)
    let mut dirs = Vec::new();
    if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
        dirs.push(PathBuf::from(venv).join("share/jupyter"));
    }
    dirs.push(PathBuf::from(".venv/share/jupyter"));
    dirs.extend(jupyter_zmq_client::dirs::data_dirs());
    list_kernels_in(&dirs)
}

// jupyter-zmq-client's own listing is tokio-gated; this is the sync
// equivalent over its data_dirs(). Jupyter semantics: earlier data dirs
// shadow later ones for the same kernel name.
fn list_kernels_in(data_dirs: &[PathBuf]) -> Vec<KernelspecDir> {
    let mut found: Vec<KernelspecDir> = Vec::new();
    for dir in data_dirs {
        let Ok(entries) = std::fs::read_dir(dir.join("kernels")) else {
            continue;
        };
        for entry in entries.flatten() {
            let Some(kernel_name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };
            if found.iter().any(|k| k.kernel_name == kernel_name) {
                continue;
            }
            let path = entry.path();
            let Ok(bytes) = std::fs::read(path.join("kernel.json")) else {
                continue;
            };
            let Ok(kernelspec) = serde_json::from_slice::<JupyterKernelspec>(&bytes) else {
                continue;
            };
            found.push(KernelspecDir {
                kernel_name,
                path,
                kernelspec,
            });
        }
    }
    found.sort_by(|a, b| a.kernel_name.cmp(&b.kernel_name));
    found
}

pub fn print_kernels_list() {
    let kernels = list_kernels();
    if kernels.is_empty() {
        println!("No kernels found. Searched:");
        for dir in jupyter_zmq_client::dirs::data_dirs() {
            println!("  {}", dir.join("kernels").display());
        }
        return;
    }
    let width = kernels
        .iter()
        .map(|k| k.kernel_name.len())
        .max()
        .unwrap_or(0);
    println!("Available kernels:");
    for k in kernels {
        println!(
            "  {:width$}  {}  ({})",
            k.kernel_name,
            k.path.display(),
            k.kernelspec.display_name
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_kernel(data_dir: &std::path::Path, name: &str, display_name: &str) {
        let dir = data_dir.join("kernels").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("kernel.json"),
            format!(
                r#"{{"argv": ["python", "-m", "ipykernel"], "display_name": "{display_name}", "language": "python"}}"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn lists_kernels_and_shadows_duplicates_by_dir_order() {
        // given: two data dirs where both define "python3" and the first also has "extra"
        let root = std::env::temp_dir().join(format!("yoshi-cli-test-{}", std::process::id()));
        let (first, second) = (root.join("first"), root.join("second"));
        write_kernel(&first, "python3", "First Python");
        write_kernel(&first, "extra", "Extra");
        write_kernel(&second, "python3", "Second Python");

        // when: listing across both dirs in order
        let kernels = list_kernels_in(&[first, second]);

        // then: names are sorted, and the first dir's python3 shadows the second's
        let names: Vec<&str> = kernels.iter().map(|k| k.kernel_name.as_str()).collect();
        assert_eq!(names, ["extra", "python3"]);
        let python3 = kernels.iter().find(|k| k.kernel_name == "python3").unwrap();
        assert_eq!(python3.kernelspec.display_name, "First Python");

        std::fs::remove_dir_all(&root).ok();
    }
}
