use anyhow::{anyhow, Error, Result};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
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
