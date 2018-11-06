extern crate basic_otp;
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
struct Opt {
    #[structopt(
        short = "c", long = "config", parse(from_os_str), help = "Use the specified config file"
    )]
    config: Option<PathBuf>,
    #[structopt(
        short = "m",
        long = "min-time",
        default_value = "5",
        help = "Minimum time left below which the next time slot will be used"
    )]
    min: u32,
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}
#[derive(StructOpt, Debug)]
enum Cmd {
    #[structopt(name = "get", about = "Get a code")]
    Get {
        #[structopt(short = "n", long = "name", help = "The login name")]
        name: String,
    },
    #[structopt(name = "add", about = "Add a new login")]
    Add {
        #[structopt(short = "n", long = "name", help = "The login name")]
        name: String,
        #[structopt(short = "k", long = "key", help = "The key value encoded in BASE32")]
        key: String,
    },
    #[structopt(name = "rm", about = "Remove a login")]
    Rm {
        #[structopt(short = "n", long = "name", help = "The login name")]
        name: String,
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

fn get_logins(config: &PathBuf) -> Result<HashMap<String, Login>, Box<std::error::Error>> {
    let f = std::fs::read_to_string(config);
    let logins: HashMap<String, Login> = if let Ok(s) = f {
        toml::from_str(&s)?
    } else {
        HashMap::new()
    };
    Ok(logins)
}
fn save_logins(config: &PathBuf, logins: &HashMap<String, Login>) -> Result<(), Box<std::error::Error>> {
    let out = toml::to_string(&logins)?;
    std::fs::write(&config, out.as_bytes())?;
    Ok(())
}
fn list(logins: &HashMap<String, Login>, min: u32) -> Result<(), Box<std::error::Error>> {
    let mut first = true;
    let mut off = 0;
    for (name, login) in logins {
        let key = BASE32.decode(login.key.as_bytes())?;
        let mut slot = totp_offset(&key, off);
        let secs_left = slot.secs_left;
        if first && secs_left < min {
            slot = totp_offset(&key, 1);
            off = 1;
            eprintln!(
                "WARNING: The time left was less then {} seconds, using next time slot",
                secs_left
            );
        }
        if first {
            eprintln!("The codes expire in {} seconds", slot.secs_left);
        }
        println!("{} -> {}", name, slot.code);
        first = false;
    }
    Ok(())
}
fn get(name: &str, logins: &HashMap<String, Login>, min: u32) -> Result<(), Box<std::error::Error>> {
    let key = match logins.get(name) {
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
    Ok(())
}
fn main() -> Result<(), Box<std::error::Error>> {
    let opt = Opt::from_args();
    let config = opt.config.unwrap_or_else(|| {
        let mut path = app_root(AppDataType::UserData, &APP_INFO).unwrap();
        path.push("logins.toml");
        path
    });
    let mut logins = get_logins(&config)?;
    let min = opt.min;
    match opt.cmd {
        Some(cmd) => {
            match cmd {
                Cmd::Get { name } => {
                    get(&name, &logins, min)?;
                }
                Cmd::Add { name, key } => {
                    logins.insert(name, Login { key });
                    save_logins(&config, &logins)?;
                }
                Cmd::Rm { name } => {
                    logins.remove(&name);
                    save_logins(&config, &logins)?;
                }
            }
        }
        None => {
            list(&logins, min)?;
        }
    }
    Ok(())
}
