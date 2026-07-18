use std::path::PathBuf;

use clap::{Parser, Subcommand};
use jupyter_protocol::JupyterKernelspec;

#[derive(Parser)]
#[command(
    name = "yoshi",
    version,
    about = "A native, GPU-rendered Jupyter notebook app"
)]
struct Cli {
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

/// What the yoshi binary should do for this invocation.
pub enum Invocation {
    Gui,
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
        None => Invocation::Gui,
    }
}

pub struct Kernelspec {
    pub name: String,
    pub path: PathBuf,
    pub spec: JupyterKernelspec,
}

pub fn list_kernels() -> Vec<Kernelspec> {
    list_kernels_in(&jupyter_zmq_client::dirs::data_dirs())
}

// jupyter-zmq-client's own listing is tokio-gated; this is the sync
// equivalent over its data_dirs(). Jupyter semantics: earlier data dirs
// shadow later ones for the same kernel name.
fn list_kernels_in(data_dirs: &[PathBuf]) -> Vec<Kernelspec> {
    let mut found: Vec<Kernelspec> = Vec::new();
    for dir in data_dirs {
        let Ok(entries) = std::fs::read_dir(dir.join("kernels")) else {
            continue;
        };
        for entry in entries.flatten() {
            let Some(name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };
            if found.iter().any(|k| k.name == name) {
                continue;
            }
            let path = entry.path();
            let Ok(bytes) = std::fs::read(path.join("kernel.json")) else {
                continue;
            };
            let Ok(spec) = serde_json::from_slice::<JupyterKernelspec>(&bytes) else {
                continue;
            };
            found.push(Kernelspec { name, path, spec });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
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
    let width = kernels.iter().map(|k| k.name.len()).max().unwrap_or(0);
    println!("Available kernels:");
    for k in kernels {
        println!(
            "  {:width$}  {}  ({})",
            k.name,
            k.path.display(),
            k.spec.display_name
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
        let names: Vec<&str> = kernels.iter().map(|k| k.name.as_str()).collect();
        assert_eq!(names, ["extra", "python3"]);
        let python3 = kernels.iter().find(|k| k.name == "python3").unwrap();
        assert_eq!(python3.spec.display_name, "First Python");

        std::fs::remove_dir_all(&root).ok();
    }
}
