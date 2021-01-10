use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// An `OutputEvent` is any output from a process. Partial or not.
/// A stream of these should be able to be reconstructed into a full output.
#[derive(Debug, Clone)]
pub enum OutputEvent {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Exit(i32),
}

/// The `Output` struct handles storing a log of previous events and continously broadcasting
/// past and new events to any listeners.
pub struct Output {
    log: Vec<OutputEvent>,
    senders: Vec<UnboundedSender<OutputEvent>>,
}

impl Output {
    /// Creates a new event log and broadcast channel.
    pub fn new() -> Self {
        Self {
            log: Vec::new(),
            senders: Vec::new(),
        }
    }

    /// Publish an event. This stores the event in a log and publishes it to all active listeners.
    pub fn publish(&mut self, event: OutputEvent) {
        // Attempt to send the events to all registered listeners and any listeners that have become inactive.
        self.senders
            .retain(|sender| sender.send(event.clone()).is_ok());

        self.log.push(event);
    }

    /// Register a new event listener that will stream all future events.
    pub fn tail(&mut self) -> UnboundedReceiver<OutputEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.senders.push(tx);
        rx
    }

    /// Register a new event listener that will stream all past and future events.
    pub fn tail_from_start(&mut self) -> UnboundedReceiver<OutputEvent> {
        let (tx, rx) = mpsc::unbounded_channel();

        for event in &self.log {
            tx.send(event.clone()).unwrap();
        }

        self.senders.push(tx);
        rx
    }
}
