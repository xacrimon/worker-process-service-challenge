mod output;
mod remote;

pub use output::OutputEvent;

use anyhow::{anyhow, Result};
use output::Output;
use remote::Remote;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

/// An engine represents an abstraction on top of the OS
/// that allows you to run jobs associated with a username and a unique id
/// while capturing and streaming output.
pub struct Engine {
    remotes: HashMap<UniqueJobId, Mutex<Remote>>,
    outputs: HashMap<UniqueJobId, Arc<Mutex<Output>>>,
}

impl Engine {
    pub fn new() -> Engine {
        Self {
            remotes: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    /// Spawn a new job associated with a certain username using the given
    /// username, program path, working directory, arguments and environment variables.
    pub fn spawn(
        &mut self,
        username: String,
        program: &str,
        working_directory: &str,
        args: &[String],
        envs: &HashMap<String, String>,
    ) -> Result<Uuid> {
        // Create a new job id based on a random UUID and the supplied username.
        let uuid = Uuid::new_v4();
        let id = UniqueJobId::new(username, uuid);

        // Create the remote for the job and attach an output to it.
        let mut remote = Remote::new(program, working_directory, args, envs)?;
        let output = Arc::new(Mutex::new(Output::new()));
        remote.spawn_events_processor(Arc::clone(&output))?;

        self.remotes.insert(id.clone(), Mutex::new(remote));
        self.outputs.insert(id, output);
        Ok(uuid)
    }

    /// Stop the specified job. If the job has already terminated, nothing will be done.
    pub fn stop(&self, id: &UniqueJobId) -> Result<()> {
        let mut remote = self
            .remotes
            .get(id)
            .ok_or(anyhow!("job does not exist"))?
            .lock()
            .unwrap();

        remote.stop();
        Ok(())
    }

    /// Creates an event stream that receives all future output events from a job.
    pub fn tail_log(&self, id: &UniqueJobId) -> Result<UnboundedReceiver<OutputEvent>> {
        let mut output = self
            .outputs
            .get(id)
            .ok_or(anyhow!("job does not exist"))?
            .lock()
            .unwrap();

        Ok(output.tail())
    }

    /// Creates an event stream that receives all past and future output events from a job.
    pub fn tail_log_from_start(&self, id: &UniqueJobId) -> Result<UnboundedReceiver<OutputEvent>> {
        let mut output = self
            .outputs
            .get(id)
            .ok_or(anyhow!("job does not exist"))?
            .lock()
            .unwrap();

        Ok(output.tail_from_start())
    }
}

/// Represents a job associated with a username.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UniqueJobId {
    user: String,
    job: Uuid,
}

impl UniqueJobId {
    pub fn new(user: String, job: Uuid) -> UniqueJobId {
        Self { user, job }
    }
}
