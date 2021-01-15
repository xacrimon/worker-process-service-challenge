use anyhow::{anyhow, Error, Result};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug)]
pub struct StringList(pub Vec<String>);

impl FromStr for StringList {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Infallible> {
        Ok(StringList(s.split(',').map(|s| s.to_string()).collect()))
    }
}

#[derive(Debug)]
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
            .map(split_pair)
            .try_fold(HashMap::new(), |mut map, pair| {
                let (k, v) = pair?;
                map.insert(k, v);
                Ok(map)
            })
            .map(|map| StringMap(map))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "client")]
pub struct Opts {
    #[structopt(short, long)]
    endpoint: String,

    #[structopt(short, long)]
    domain: String,

    #[structopt(short, long)]
    username: String,

    #[structopt(flatten)]
    command: CommandOpts,
}

#[derive(Debug, StructOpt)]
pub enum CommandOpts {
    Spawn {
        #[structopt(short, long)]
        program_path: String,

        #[structopt(short, long)]
        working_directory: String,

        #[structopt(short, long)]
        args: StringList,

        #[structopt(short, long)]
        envs: StringMap,
    },

    Stop {
        #[structopt(short, long)]
        uuid: Uuid,
    },

    StreamLog {
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
