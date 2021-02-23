use kvs::Client;

use {
    clap::{App, Arg, SubCommand},
    failure::Fail,
    kvs::{KvsError, Result},
    slog::{error,  o,  Drain, Logger},
    std::{
        process::exit,
    },
};

fn main() {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let logger = Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!());

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("CARGO_PKG_DESCRIPTION")
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .global(true)
                .takes_value(true),
        )
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

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");

    let client = Client::new(addr.to_string());

    let res = || -> Result<()> {
        match matches.subcommand() {
            ("get", Some(args)) => {
                let key = args.value_of("key").ok_or(CliError::Key)?;

                let s = client.get(key)?;
                println!("{}", s);
            }
            ("set", Some(args)) => {
                let key = args.value_of("key").ok_or(CliError::Key)?;
                let value = args.value_of("value").ok_or(CliError::Value)?;

                client.set(key, value)?;
            }
            ("rm", Some(args)) => {
                let key = args.value_of("key").ok_or(CliError::Key)?;

                let s = client.remove(key)?;
                if s.len() != 0 {
                    eprintln!("{}", s);
                    exit(1);
                }
            }
            (cmd, _) => {
                return Err(KvsError::UnKnownOperation(cmd.to_string()).into());
            }
        }
        Ok(())
    }();
    if let Err(e) = res {
        error!(logger, "can not oprate: {:?}", e);
        exit(1);
    }
}

#[derive(Fail, Debug)]
enum CliError {
    #[fail(display = "The key is wanted")]
    Key,
    #[fail(display = "The value is wanted")]
    Value,
}
