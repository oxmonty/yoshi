// E1 inline hack: this whole module is promoted into yoshi-kernels in E3.

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use anyhow::{Context as _, Result};
use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use jupyter_protocol::connection_info::{ConnectionInfo, Transport};
use jupyter_protocol::{
    ExecuteRequest, ExecutionState, JupyterMessage, JupyterMessageContent, KernelInfoRequest,
};
use jupyter_zmq_client::{
    create_client_iopub_connection, create_client_shell_connection_with_identity,
    peer_identity_for_session,
};

pub enum Event {
    Status(String),
    Ready,
    Output(String),
    Done,
    Failed(String),
}

pub struct KernelGuard {
    child: Child,
    connection_file: PathBuf,
}

impl Drop for KernelGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.connection_file);
    }
}

pub fn python_path() -> PathBuf {
    if let Ok(p) = std::env::var("YOSHI_PYTHON") {
        return PathBuf::from(p);
    }
    let venv = std::env::current_dir()
        .map(|d| d.join(".venv/bin/python"))
        .unwrap_or_default();
    if venv.exists() {
        venv
    } else {
        PathBuf::from("python3")
    }
}

fn pick_ports(n: usize) -> Result<Vec<u16>> {
    (0..n)
        .map(|_| {
            let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
            Ok(listener.local_addr()?.port())
        })
        .collect()
}

pub fn spawn_kernel(python: &PathBuf) -> Result<(ConnectionInfo, KernelGuard)> {
    let ports = pick_ports(5)?;
    let connection_info = ConnectionInfo {
        transport: Transport::TCP,
        ip: Ipv4Addr::LOCALHOST.to_string(),
        stdin_port: ports[0],
        control_port: ports[1],
        hb_port: ports[2],
        shell_port: ports[3],
        iopub_port: ports[4],
        signature_scheme: "hmac-sha256".to_string(),
        key: uuid::Uuid::new_v4().to_string(),
        kernel_name: Some("yoshi-hello".to_string()),
    };

    let connection_file =
        std::env::temp_dir().join(format!("yoshi-kernel-{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&connection_file, serde_json::to_string(&connection_info)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // the file carries the HMAC key: owner-only
        std::fs::set_permissions(&connection_file, std::fs::Permissions::from_mode(0o600))?;
    }

    let mut cmd = Command::new(python);
    cmd.args(["-m", "ipykernel_launcher", "-f"])
        .arg(&connection_file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        // own process group so interrupt/kill reaches kernel subprocesses too
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    }
    let child = cmd
        .spawn()
        .with_context(|| format!("spawning ipykernel via {python:?} (run script/bootstrap?)"))?;

    Ok((
        connection_info,
        KernelGuard {
            child,
            connection_file,
        },
    ))
}

// The session stays alive across executes: the kernel boots once (prewarmed at app
// startup), then each command is a cheap protocol round-trip.
pub async fn run_session(
    connection_info: ConnectionInfo,
    commands: UnboundedReceiver<String>,
    events: UnboundedSender<Event>,
) {
    if let Err(e) = run_session_inner(connection_info, commands, &events).await {
        let _ = events.unbounded_send(Event::Failed(e.to_string()));
    }
}

async fn run_session_inner(
    connection_info: ConnectionInfo,
    mut commands: UnboundedReceiver<String>,
    events: &UnboundedSender<Event>,
) -> Result<()> {
    let session_id = uuid::Uuid::new_v4().to_string();

    let _ = events.unbounded_send(Event::Status("connecting".into()));
    let mut iopub = create_client_iopub_connection(&connection_info, "", &session_id).await?;
    let identity = peer_identity_for_session(&session_id)?;
    let mut shell =
        create_client_shell_connection_with_identity(&connection_info, &session_id, identity)
            .await?;

    // Ready gate (PRD, kernel session loop): iopub SUB is a slow joiner, so an execute
    // sent before the subscription lands loses its output. Poll kernel_info until the
    // reply arrives AND at least one iopub message is observed.
    let _ = events.unbounded_send(Event::Status("waiting for kernel".into()));
    let mut got_reply = false;
    let mut got_iopub = false;
    for _attempt in 0..60 {
        if !got_reply {
            let req: JupyterMessage = KernelInfoRequest {}.into();
            shell.send(req).await?;
            if let Ok(Ok(reply)) =
                async_dispatcher::timeout(Duration::from_millis(500), shell.read()).await
            {
                got_reply = reply.header.msg_type == "kernel_info_reply";
            }
        }
        if !got_iopub
            && let Ok(Ok(_any)) =
                async_dispatcher::timeout(Duration::from_millis(500), iopub.read()).await
        {
            got_iopub = true;
        }
        if got_reply && got_iopub {
            break;
        }
    }
    anyhow::ensure!(
        got_reply && got_iopub,
        "kernel did not become ready (reply: {got_reply}, iopub: {got_iopub})"
    );

    let _ = events.unbounded_send(Event::Ready);

    while let Some(code) = commands.next().await {
        let request: JupyterMessage = ExecuteRequest {
            code,
            silent: false,
            store_history: true,
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        }
        .into();
        let msg_id = request.header.msg_id.clone();
        shell.send(request).await?;

        loop {
            let msg = async_dispatcher::timeout(Duration::from_secs(30), iopub.read())
                .await
                .map_err(|_| anyhow::anyhow!("timed out waiting for iopub output"))??;
            // outputs are keyed by parent msg_id, never "the currently running cell"
            if msg
                .parent_header
                .as_ref()
                .is_none_or(|h| h.msg_id != msg_id)
            {
                continue;
            }
            match msg.content {
                JupyterMessageContent::StreamContent(stream) => {
                    let _ = events.unbounded_send(Event::Output(stream.text));
                }
                JupyterMessageContent::ExecuteResult(result) => {
                    let text = result
                        .data
                        .content
                        .iter()
                        .find_map(|m| match m {
                            jupyter_protocol::MediaType::Plain(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "(non-text result)".to_string());
                    let _ = events.unbounded_send(Event::Output(text + "\n"));
                }
                JupyterMessageContent::ErrorOutput(err) => {
                    let _ = events
                        .unbounded_send(Event::Output(format!("{}: {}\n", err.ename, err.evalue)));
                }
                JupyterMessageContent::Status(status)
                    if matches!(status.execution_state, ExecutionState::Idle) =>
                {
                    // completion is iopub status:idle with matching parent, not execute_reply
                    let _ = events.unbounded_send(Event::Done);
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
