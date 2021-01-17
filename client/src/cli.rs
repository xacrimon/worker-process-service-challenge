use anyhow::{anyhow, Error, Result};
use protocol::{stream_log_response, StreamLogResponse};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str;
use std::str::FromStr;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct StringList(pub Vec<String>);

impl FromStr for StringList {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Infallible> {
        Ok(StringList(s.split(',').map(|s| s.to_string()).collect()))
    }
}

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

pub trait StreamWriter {
    fn start(&mut self) -> Result<()>;
    fn write(&mut self, event: StreamLogResponse) -> Result<()>;
}

struct RawStreamWriter;

impl StreamWriter for RawStreamWriter {
    fn start(&mut self) -> Result<()> {
        println!("raw log:");
        Ok(())
    }

    fn write(&mut self, event: StreamLogResponse) -> Result<()> {
        println!("{:?}", event);
        Ok(())
    }
}

struct StdoutStreamWriter;

impl StreamWriter for StdoutStreamWriter {
    fn start(&mut self) -> Result<()> {
        println!("stdout log:");
        Ok(())
    }

    fn write(&mut self, event: StreamLogResponse) -> Result<()> {
        let response = event
            .response
            .ok_or_else(|| anyhow!("incomplete event received"))?;

        if let stream_log_response::Response::Stdout(data) = response {
            let text = str::from_utf8(&data.output)?;
            println!("> {}", text);
        }

        Ok(())
    }
}

struct StderrStreamWriter;

impl StreamWriter for StderrStreamWriter {
    fn start(&mut self) -> Result<()> {
        println!("stderr log:");
        Ok(())
    }

    fn write(&mut self, event: StreamLogResponse) -> Result<()> {
        let response = event
            .response
            .ok_or_else(|| anyhow!("incomplete event received"))?;

        if let stream_log_response::Response::Stderr(data) = response {
            let text = str::from_utf8(&data.output)?;
            println!("> {}", text);
        }

        Ok(())
    }
}
