extern crate basic_otp;
#[macro_use]
extern crate structopt;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate app_dirs;
extern crate data_encoding;

use std::path::PathBuf;

use app_dirs::*;
use basic_otp::totp_offset;
use data_encoding::BASE32;
use structopt::StructOpt;

use std::collections::HashMap;

#[derive(StructOpt, Debug)]
enum Opt {
    #[structopt(name = "get", about = "Get a code")]
    Get {
        #[structopt(
            short = "c", long = "config", parse(from_os_str), help = "Use the specified config file"
        )]
        config: Option<PathBuf>,
        #[structopt(short = "n", long = "name", help = "The login name")]
        name: String,
        #[structopt(
            short = "m",
            long = "min-time",
            default_value = "5",
            help = "Minimum time left below which the next time slot will be used"
        )]
        min: u32,
    },
    #[structopt(name = "add", about = "Add a new login")]
    Add {
        #[structopt(
            short = "c", long = "config", parse(from_os_str), help = "Use the specified config file"
        )]
        config: Option<PathBuf>,
        #[structopt(short = "n", long = "name", help = "The login name")]
        name: String,
        #[structopt(short = "k", long = "key", help = "The key value encoded in BASE32")]
        key: String,
    },
    #[structopt(name = "rm", about = "Remove a login")]
    Rm {
        #[structopt(
            short = "c", long = "config", parse(from_os_str), help = "Use the specified config file"
        )]
        config: Option<PathBuf>,
        #[structopt(short = "n", long = "name", help = "The login name")]
        name: String,
    },
    #[structopt(name = "list", about = "List all logins")]
    List {
        #[structopt(
            short = "c", long = "config", parse(from_os_str), help = "Use the specified config file"
        )]
        config: Option<PathBuf>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Login {
    key: String,
}

const APP_INFO: AppInfo = AppInfo {
    name: "totp-cli",
    author: "Yuri91",
};

fn main() -> Result<(), Box<std::error::Error>> {
    let opt = Opt::from_args();
    match opt {
        Opt::Get { config, name, min } => {
            let config = config.unwrap_or_else(|| {
                let mut path = app_root(AppDataType::UserData, &APP_INFO).unwrap();
                path.push("logins.toml");
                path
            });
            let f = std::fs::read_to_string(config)?;
            let logins: HashMap<String, Login> = toml::from_str(&f)?;

            let key = match logins.get(&name) {
                Some(v) => BASE32.decode(v.key.as_bytes())?,
                None => {
                    eprintln!("Login not found: {}", name);
                    return Ok(());
                }
            };
            let mut slot = totp_offset(&key, 0);
            let secs_left = slot.secs_left;
            if secs_left < min {
                slot = totp_offset(&key, 1);
                eprintln!(
                    "WARNING: The time left was less then {} seconds, using next time slot",
                    secs_left
                );
            }
            eprintln!("The code expires in {} seconds", slot.secs_left);
            println!("{}", slot.code);
        }
        Opt::Add { config, name, key } => {
            let config = config.unwrap_or_else(|| {
                let mut path = app_root(AppDataType::UserData, &APP_INFO).unwrap();
                path.push("logins.toml");
                path
            });
            let f = std::fs::read_to_string(&config);
            let mut logins: HashMap<String, Login> = if let Ok(s) = f {
                toml::from_str(&s)?
            } else {
                HashMap::new()
            };
            logins.insert(name, Login { key });
            let out = toml::to_string(&logins)?;
            std::fs::write(&config, out.as_bytes())?;
        }
        Opt::Rm { config, name } => {
            let config = config.unwrap_or_else(|| {
                let mut path = app_root(AppDataType::UserData, &APP_INFO).unwrap();
                path.push("logins.toml");
                path
            });
            let f = std::fs::read_to_string(&config);
            let mut logins: HashMap<String, Login> = if let Ok(s) = f {
                toml::from_str(&s)?
            } else {
                HashMap::new()
            };
            logins.remove(&name);
            let out = toml::to_string(&logins)?;
            std::fs::write(&config, out.as_bytes())?;
        }
        Opt::List { config } => {
            let config = config.unwrap_or_else(|| {
                let mut path = app_root(AppDataType::UserData, &APP_INFO).unwrap();
                path.push("logins.toml");
                path
            });
            let f = std::fs::read_to_string(config)?;
            let logins: HashMap<String, Login> = toml::from_str(&f)?;
            for l in logins.keys() {
                println!("{}", l);
            }
        }
    }
    Ok(())
}
