# E3: Kernel runtime

A headless integration test launches ipykernel, passes the ready handshake, executes `print("hi")`, receives the stream output, interrupts a busy kernel, and shuts down cleanly; shipped as a `yoshi-kernels` crate with CI coverage.

Spec: [Kernel session loop](../../PRD.md#kernel-session-loop)

## Stories

- [ ] Kernelspec discovery reads kernelspec JSON directly from disk (all standard dirs, including `~/Library/Jupyter/kernels` on macOS; never shells out to `jupyter`)
- [ ] Managed default kernel: first launch provisions CPython + ipykernel via the bundled `uv` into `~/.local/share/yoshi/managed-kernel/` (outside the app bundle; first provision needs network; clear progress + error states) — see [Managed default kernel](../../PRD.md#managed-default-kernel)
- [ ] Launch in its own process group, connection file written 0600 to the Jupyter runtime dir, `kill_on_drop`, stale-file cleanup; shutdown/restart lifecycle
- [ ] Ready gate after every launch and restart: `kernel_info_request` reply + first observed iopub `status` before accepting work (iopub SUB is a slow joiner)
- [ ] Session actor: shell + iopub + control routing tasks; outputs keyed by `parent_header.msg_id`; consumes `execute_reply` for `execution_count` and the ok/error/aborted verdict; `allow_stdin: false` on every execute
- [ ] Interrupt honors the kernelspec `interrupt_mode`: SIGINT to the process group (ipykernel's default), `interrupt_request` on control for message-mode kernels
- [ ] Execution state machine (starting/idle/busy/dead) exposed as a watch channel; CI installs python + ipykernel and runs the round-trip headlessly
