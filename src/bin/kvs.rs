use {
    clap::{App, Arg, SubCommand},
    failure::Fail,
    kvs::{KvStore, KvsError, Result},
    std::process::exit,
};

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("CARGO_PKG_DESCRIPTION")
        .author(env!("CARGO_PKG_AUTHORS"))
        //.arg(Arg::with_name("version").short("V").long("version"))
        .subcommand(
            SubCommand::with_name("get")
                .arg(Arg::with_name("key").required(true))
                .about("fetch value matched by the key"),
        )
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").required(true))
                .arg(Arg::with_name("value").required(true))
                .about("set the key to value in cache"),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .arg(Arg::with_name("key").required(true))
                .about("rm the key value pair in cache"),
        )
        .get_matches();

    if std::env::args().len() < 2 {
        println!("{}", matches.usage());
        exit(1);
    }

    let mut kvs = KvStore::open("./")?;

    Ok(match matches.subcommand() {
        ("get", Some(args)) => {
            let key = args.value_of("key").ok_or(CliError::Key)?;
            match kvs.get(key.to_string())? {
                Some(value) => {
                    println!("{}", value)
                }
                None => {
                    println!("Key not found")
                }
            }
        }
        ("set", Some(args)) => {
            let key = args.value_of("key").ok_or(CliError::Key)?;
            let value = args.value_of("value").ok_or(CliError::Value)?;
            kvs.set(key.to_string(), value.to_string())?;
        }
        ("rm", Some(args)) => {
            let key = args.value_of("key").ok_or(CliError::Key)?;
            if let Err(err) = kvs.remove(key.to_string()) {
                println!("Key not found");
                exit(1);
            }
        }
        (cmd, _) => {
            return Err(KvsError::UnKnownOperation(cmd.to_string()).into());
        }
    })
}

#[derive(Fail, Debug)]
enum CliError {
    #[fail(display = "The key is wanted")]
    Key,
    #[fail(display = "The value is wanted")]
    Value,
}
