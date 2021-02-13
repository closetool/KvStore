use {
    clap::{App, Arg, SubCommand},
    kvs::KvStore,
    std::{path::PathBuf, process::exit},
};

fn main() {
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

    if std::env::args().len() < 1 {
        println!("{}", matches.usage());
        exit(1);
    }

    let mut kvs = KvStore::new();

    match matches.subcommand() {
        ("get", Some(args)) => {
            panic!("unimplemented");
            let key = args.value_of("key").unwrap();
            match kvs.get(key.to_string()) {
                Some(value) => {
                    println!("{} => {}", key, value)
                }
                None => {}
            }
        }
        ("set", Some(args)) => {
            panic!("unimplemented");
            let key = args.value_of("key").unwrap();
            let value = args.value_of("value").unwrap();
            kvs.set(key.to_string(), value.to_string());
        }
        ("rm", Some(args)) => {
            panic!("unimplemented");
            let key = args.value_of("key").unwrap();
            kvs.remove(key.to_string());
        }
        (cmd, _) => {
            panic!(format!("no command {}, please check input", cmd))
        }
    }
}
