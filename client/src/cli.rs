use anyhow::{anyhow, Error, Result};
use protocol::{stream_log_response, StreamLogResponse};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str;
use std::str::FromStr;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use uuid::Uuid;

/// A newtype around a vec of strings to allow structopt to parse it.
#[derive(Debug, Default)]
pub struct StringList(pub Vec<String>);

impl FromStr for StringList {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Infallible> {
        Ok(StringList(s.split(',').map(|s| s.to_string()).collect()))
    }
}

/// A newtype around a map of strings to strings to allow structopt to parse it.
#[derive(Debug, Default)]
pub struct StringMap(pub HashMap<String, String>);

impl FromStr for StringMap {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        fn split_pair(s: &str) -> Result<(String, String)> {
            let mut iter = s.split('=');
            let k = iter.next().ok_or_else(|| anyhow!("no env key provided"))?;
            let v = iter
                .next()
                .ok_or_else(|| anyhow!("no env value provided"))?;

            Ok((k.trim().into(), v.trim().into()))
        }

        s.split(',')
            .filter(|s| !s.is_empty())
            .map(split_pair)
            .try_fold(HashMap::new(), |mut map, pair| {
                let (k, v) = pair?;
                map.insert(k, v);
                Ok(map)
            })
            .map(StringMap)
    }
}

/// The base CLI options.
#[derive(Debug, StructOpt)]
#[structopt(name = "client")]
pub struct Opts {
    #[structopt(short, long)]
    pub endpoint: String,

    #[structopt(short, long)]
    pub domain: String,

    #[structopt(short, long)]
    pub username: String,

    #[structopt(flatten)]
    pub command: CommandOpts,
}

/// This represents all subcommands.
#[derive(Debug, StructOpt)]
pub enum CommandOpts {
    Spawn {
        #[structopt(short, long)]
        program_path: String,

        #[structopt(short, long, default_value = ".")]
        working_directory: String,

        #[structopt(short, long, default_value = "")]
        args: StringList,

        #[structopt(short, long, default_value = "")]
        envs: StringMap,
    },

    Stop {
        #[structopt(short, long)]
        uuid: Uuid,
    },

    StreamLog {
        #[structopt(short, long, case_insensitive = true)]
        stream_type: StreamType,

        #[structopt(short, long)]
        uuid: Uuid,

        #[structopt(short, long)]
        past_events: bool,
    },

    Status {
        #[structopt(short, long)]
        uuid: Uuid,
    },
}

arg_enum! {
    /// Possible options for ways to display the incoming event stream.
    #[derive(Debug, PartialEq, Eq)]
    pub enum StreamType {
        Raw,
        Stdout,
        Stderr,
    }
}

impl StreamType {
    pub fn writer(&self) -> Box<dyn StreamWriter + Send + 'static> {
        match self {
            Self::Raw => Box::new(RawStreamWriter),
            Self::Stdout => Box::new(StdoutStreamWriter),
            Self::Stderr => Box::new(StderrStreamWriter),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StreamStatus {
    ExpectingMore,
    Terminated(i32),
}

pub trait StreamWriter {
    fn start(&mut self) -> Result<()>;
    fn write(&mut self, event: StreamLogResponse) -> Result<StreamStatus>;
}

/// Event writer that writes all events in their raw form.
struct RawStreamWriter;

impl StreamWriter for RawStreamWriter {
    fn start(&mut self) -> Result<()> {
        println!("raw log:");
        Ok(())
    }

    fn write(&mut self, event: StreamLogResponse) -> Result<StreamStatus> {
        let response = event
            .response
            .ok_or_else(|| anyhow!("incomplete event received"))?;

        println!("{:?}", response);
        Ok(status_from_response(&response))
    }
}

/// Event writer that filters out stdout events and displays them as text.
struct StdoutStreamWriter;

impl StreamWriter for StdoutStreamWriter {
    fn start(&mut self) -> Result<()> {
        println!("stdout log:");
        Ok(())
    }

    fn write(&mut self, event: StreamLogResponse) -> Result<StreamStatus> {
        let response = event
            .response
            .ok_or_else(|| anyhow!("incomplete event received"))?;

        let status = status_from_response(&response);
        if let stream_log_response::Response::Stdout(data) = response {
            let text = str::from_utf8(&data.output)?;
            print!("{}", text);
        }

        Ok(status)
    }
}

/// Event writer that filters out stderr events and displays them as text.
struct StderrStreamWriter;

impl StreamWriter for StderrStreamWriter {
    fn start(&mut self) -> Result<()> {
        println!("stderr log:");
        Ok(())
    }

    fn write(&mut self, event: StreamLogResponse) -> Result<StreamStatus> {
        let response = event
            .response
            .ok_or_else(|| anyhow!("incomplete event received"))?;

        let status = status_from_response(&response);
        if let stream_log_response::Response::Stderr(data) = response {
            let text = str::from_utf8(&data.output)?;
            print!("{}", text);
        }

        Ok(status)
    }
}

fn status_from_response(response: &stream_log_response::Response) -> StreamStatus {
    if let stream_log_response::Response::Exit(event) = response {
        StreamStatus::Terminated(event.code)
    } else {
        StreamStatus::ExpectingMore
    }
}
