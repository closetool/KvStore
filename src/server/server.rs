use {
    crate::{ KvsEngine, KvsError, Result},
    failure::{ Fail},
    slog::{debug, error, Logger},
    std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        process::exit,
        rc::Rc,
    },
};

pub struct Server<E: KvsEngine> {
    engine: E,
    logger: Rc<Logger>,
}

impl<E: KvsEngine> Server<E> {
    pub fn new(engine: E, logger: Rc<Logger>) -> Result<Self> {
        Ok(Server { engine, logger })
    }

    pub fn serve(&mut self, addr: &String) -> Result<()> {
        let logger = self.logger.clone();
        let listener = match TcpListener::bind(addr) {
            Ok(l) => l,
            Err(e) => {
                error!(logger, "listen on {} failed: {:?}", addr, e);
                exit(1);
            }
        };

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    error!(logger, "accept tcp connection failed: {:?}", e);
                    continue;
                }
            };
            self.handle(stream)?;
        }
        Ok(())
    }

    pub fn handle(&mut self, stream: TcpStream) -> Result<()> {
        let engine = &mut self.engine;
        let logger = self.logger.clone();
        let mut stream = stream;

        debug!(logger, "accept conn: {:?}", stream);

        let mut buf = String::new();
        stream.read_to_string(&mut buf)?;

        debug!(logger, "read from stream: {}", buf);

        let params: Vec<&str> = buf.split(" ").collect();

        match params.as_slice() {
            ["get", params @ ..] => {
                let key = params.get(0).ok_or(CliError::Key)?;
                match engine.get(key.to_string())? {
                    Some(value) => {
                        if let Err(e) = stream.write_fmt(format_args!("{}", value)) {
                            error!(logger, "write data to tcp stream failed: {:?}", e);
                            return Ok(());
                        }
                    }
                    None => {
                        if let Err(e) = stream.write_fmt(format_args!("Key not found")) {
                            error!(logger, "write data to tcp stream failed: {:?}", e);
                            return Ok(());
                        }
                    }
                }
            }
            ["set", params @ ..] => {
                let key = params.get(0).ok_or(CliError::Key)?;
                let value = params.get(1).ok_or(CliError::Value)?;
                engine.set(key.to_string(), value.to_string())?;
            }
            ["rm", params @ ..] => {
                let key = params.get(0).ok_or(CliError::Key)?;
                if let Err(err) = engine.remove(key.to_string()) {
                    debug!(logger, "Key not found: {:?}", err);
                    if let Err(e) = stream.write_fmt(format_args!("Key not found: {:?}", err)) {
                        error!(logger, "write data to tcp stream failed: {:?}", e);
                        return Ok(());
                    }
                }
            }
            _ => {
                return Err(KvsError::UnKnownOperation("".to_string()).into());
            }
        }
        Ok(())
    }
}

#[derive(Fail, Debug)]
enum CliError {
    #[fail(display = "The key is wanted")]
    Key,
    #[fail(display = "The value is wanted")]
    Value,
}
