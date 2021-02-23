use {
    clap::{App, Arg},
    kvs::{ KvStore, KvsEngine,SledKvsEngine,  Result},
    slog::{ error, info, o, Drain, Logger},
    std::{
        fs,
        io::{Write},
        process::exit,
        rc::Rc,
    },
};

fn main() {
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
        .arg(
            Arg::with_name("engine")
                .short("e")
                .long("engine")
                .global(true)
                .takes_value(true),
        )
        .get_matches();

    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let logger = Rc::new(Logger::root(
        slog_term::FullFormat::new(plain).build().fuse(),
        o!(),
    ));

    info!(logger, "name: {}", "kvs-server");
    info!(logger, "version: {}", env!("CARGO_PKG_VERSION"));

    let cli_engine = matches.value_of("engine").unwrap_or("kvs");

    let pre_engine = match fs::read_to_string("./pre_engine") {
        Ok(s) => s,
        Err(_) => cli_engine.to_string(),
    };

    if !cli_engine.to_string().eq(&pre_engine) {
        error!(logger, "use db engine different from previous");
        exit(1);
    }

    match fs::File::create("./pre_engine") {
        Ok(mut f) => if let Err(e) = f.write_all(cli_engine.to_string().as_bytes()){
            error!(logger,"can not write pre_engine: {:?}",e);
            exit(1);
        },
        Err(e) => {error!(logger,"can not create pre_engine file: {:?}",e);exit(1);},
    };

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");

    info!(logger, "addr: {}", addr);

    let res = || -> Result<()> {
        Ok(match cli_engine {
            "kvs" => run(KvStore::open("./")?, addr.to_string(), logger.clone())?,
            "sled" => {let db = sled::Db::start_default("./")?;run(SledKvsEngine::new(db), addr.to_string(), logger.clone())?},
            e @ _ => {
                error!(logger, "no such engine: {}", e);
                exit(1);
            }
        })
    }();

    if let Err(e) = res {
        error!(logger, "communicate with server failed: {:?}", e);
        exit(1);
    }
}

fn run<E: KvsEngine>(engine: E, addr: String, logger: Rc<Logger>) -> Result<()> {
    let mut server = kvs::Server::new(engine, logger)?;
    server.serve(&addr)?;
    Ok(())
}
