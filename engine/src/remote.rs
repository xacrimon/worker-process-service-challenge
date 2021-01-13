use crate::output::{Output, OutputEvent};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    select,
    sync::oneshot,
    task,
};

/// The buffer size used for reading from stdout and stderr.
/// Here I've chosen it to be 1 KiB but this is pretty arbitrary.
/// Performance only really differs when a program writes large amounts of data
/// to stdout/stderr.
const READ_BUFFER_SIZE: usize = 1024;

/// A remote is a sort of overwatch that monitors a process.
/// It manages starting, stopping and streaming stdout/stderr + exit as events to an `Output`.
#[derive(Debug)]
pub struct Remote {
    /// The RAII handle to the child process.
    child: Option<Child>,

    kill_switch: Option<oneshot::Sender<()>>,
    kill_switch_rx: Option<oneshot::Receiver<()>>,
}

impl Remote {
    /// Creates a new remote with
    pub fn new(
        program: &str,
        working_directory: &str,
        args: &[String],
        envs: &HashMap<String, String>,
    ) -> Result<Self> {
        let mut command = Command::new(program);
        command.current_dir(working_directory).args(args).envs(envs);
        let child = command.spawn()?;
        let (kill_switch, kill_switch_rx) = oneshot::channel();

        Ok(Self {
            child: Some(child),
            kill_switch: Some(kill_switch),
            kill_switch_rx: Some(kill_switch_rx),
        })
    }

    /// Sends SIGINT to a process should it still be running. This allows it to perform a graceful exit.
    pub fn stop(&mut self) -> Result<()> {
        let kill_switch = self
            .kill_switch
            .take()
            .ok_or_else(|| anyhow!("already sent stop signal"))?;
        let _ = kill_switch.send(());
        Ok(())
    }

    /// Spawn event processors that monitor the process for things like output and termination
    /// and publishes events based on that.
    pub fn spawn_events_processor(&mut self, output: Arc<Mutex<Output>>) -> Result<()> {
        let mut kill_switch = self
            .kill_switch_rx
            .take()
            .ok_or_else(|| anyhow!("could not grab process kill switch"))?;

        // Nab the child RAII handle from the remote. If it's taken, this method has already called.
        let mut child = self
            .child
            .take()
            .ok_or_else(|| anyhow!("events processor already spawned"))?;

        // Nab the RAII stdout handle from the remote. If it's taken, this method has already called.
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("could not attach stdout"))?;

        // Nab the RAII stderr handle from the remote. If it's taken, this method has already called.
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("could not attach stderr"))?;

        task::spawn(async move {
            let mut stdout_buffer = [0; READ_BUFFER_SIZE];
            let mut stderr_buffer = [0; READ_BUFFER_SIZE];

            'outer: loop {
                select! {
                    exit_status = &mut child => {
                        let code = exit_status.map(|s| s.code()).ok().flatten().unwrap_or(1);
                        let event = OutputEvent::Exit(code);
                        let mut output_guard = output.lock().unwrap();
                        output_guard.publish(event);
                    }

                    _ = &mut kill_switch => {

                    }

                    maybe_read = stdout.read(&mut stdout_buffer) => {
                        if let Ok(read) = maybe_read {
                            if read == 0 {
                                break 'outer;
                            } else {
                                let bytes = Vec::from(&stdout_buffer[..read]);
                                let event = OutputEvent::Stdout(bytes);
                                let mut output_guard = output.lock().unwrap();
                                output_guard.publish(event);
                            }
                        } else {
                            break 'outer;
                        }
                    }

                    maybe_read = stderr.read(&mut stderr_buffer) => {
                        if let Ok(read) = maybe_read {
                            if read == 0 {
                                break 'outer;
                            } else {
                                let bytes = Vec::from(&stderr_buffer[..read]);
                                let event = OutputEvent::Stderr(bytes);
                                let mut output_guard = output.lock().unwrap();
                                output_guard.publish(event);
                            }
                        } else {
                            break 'outer;
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
