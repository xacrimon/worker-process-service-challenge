use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// An `OutputEvent` is any output from a process. Partial or not.
/// A stream of these should be able to be reconstructed into a full output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputEvent {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Exit(i32),
}

/// The `Output` struct handles storing a log of previous events and continously broadcasting
/// past and new events to any listeners.
#[derive(Debug)]
pub struct Output {
    pub log: Vec<OutputEvent>,
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

    /// Register a new event listener that will all future events and optionally those of the past.
    pub fn tail(&mut self, from_start: bool) -> UnboundedReceiver<OutputEvent> {
        let (tx, rx) = mpsc::unbounded_channel();

        if from_start {
            for event in &self.log {
                tx.send(event.clone()).unwrap();
            }
        }

        self.senders.push(tx);
        rx
    }

    pub fn get_events(&mut self) -> Vec<OutputEvent> {
        self.log.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{Output, OutputEvent};

    #[tokio::test]
    async fn publish_receive() {
        let mut output = Output::new();
        let mut rx = output.tail(false);
        let event = OutputEvent::Exit(5);
        output.publish(event.clone());
        assert_eq!(rx.recv().await, Some(event));
    }

    #[tokio::test]
    async fn publish_receive_past() {
        let mut output = Output::new();
        let event = OutputEvent::Exit(5);
        output.publish(event.clone());
        let mut rx = output.tail(true);
        assert_eq!(rx.recv().await, Some(event));
    }
}
