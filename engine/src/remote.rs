use crate::output::{Output, OutputEvent};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::{
    io::Read,
    process::{Child, Command},
    sync::{Arc, Mutex},
};
use tokio::task;

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

    /// The operating system identifier of the child process.
    pid: u32,

    /// Whether the child process has already been stopped.
    stopped: bool,
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
        let pid = child.id();

        Ok(Self {
            child: Some(child),
            pid,
            stopped: false,
        })
    }

    /// Sends SIGINT to a process should it still be running. This allows it to perform a graceful exit.
    pub fn stop(&mut self) {
        if !self.stopped {
            // This is a pretty safe thing to do as long as we are using a POSIX compliant and relatively
            // sane libc implementation. That said everything in the libc crate is marked unsafe because
            // the behaviour may differ by libc.
            let status = unsafe { libc::kill(self.pid as i32, libc::SIGINT) };

            self.stopped = true;
            assert_eq!(status, 0);
        }
    }

    /// Spawn event processors that monitor the process for things like output and termination
    /// and publishes events based on that.
    pub fn spawn_events_processor(&mut self, output_1: Arc<Mutex<Output>>) -> Result<()> {
        // Create two additional references to the `Output`.
        let output_2 = Arc::clone(&output_1);
        let output_3 = Arc::clone(&output_1);

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

        // Spawns a task that will stream events from the stdout unix pipe to the output channel.
        task::spawn_blocking(move || {
            let mut buffer = [0; READ_BUFFER_SIZE];

            while let Ok(read) = stdout.read(&mut buffer) {
                if read == 0 {
                    break;
                }

                let bytes = Vec::from(&buffer[..read]);
                let event = OutputEvent::Stdout(bytes);
                let mut output_1_guard = output_1.lock().unwrap();
                output_1_guard.publish(event);
            }
        });

        // Spawns a task that will stream events from the stderr unix pipe to the output channel.
        task::spawn_blocking(move || {
            let mut buffer = [0; READ_BUFFER_SIZE];

            while let Ok(read) = stderr.read(&mut buffer) {
                if read == 0 {
                    break;
                }

                let bytes = Vec::from(&buffer[..read]);
                let event = OutputEvent::Stderr(bytes);
                let mut output_2_guard = output_2.lock().unwrap();
                output_2_guard.publish(event);
            }
        });

        // Spawns a task that will monitor the child process for termination.
        task::spawn_blocking(move || {
            let code = child.wait().map(|s| s.code()).ok().flatten().unwrap_or(1);
            let event = OutputEvent::Exit(code);
            let mut output_3_guard = output_3.lock().unwrap();
            output_3_guard.publish(event);
        });

        Ok(())
    }
}
