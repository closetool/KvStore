use {
    crate::{Result},
    std::{
        io::{Read, Write},
        net::TcpStream,
    },
};

pub struct Client {
    addr: String,
}

impl Client {
    pub fn new(addr: String) -> Self {
        return Client { addr };
    }
    pub fn get(&self, key: &str) -> Result<String> {
        let mut conn = TcpStream::connect(self.addr.to_string())?;

        let req = ["get", key].join(" ");
        conn.write_all(req.as_bytes())?;
        conn.shutdown(std::net::Shutdown::Write)?;

        let mut buf = String::new();
        conn.read_to_string(&mut buf)?;
        Ok(buf)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = TcpStream::connect(self.addr.to_string())?;

        let req = ["set", key, value].join(" ");
        conn.write_all(req.as_bytes())?;
        conn.shutdown(std::net::Shutdown::Write)?;
        Ok(())
    }

    pub fn remove(&self, key: &str) -> Result<String> {
        let mut conn = TcpStream::connect(self.addr.to_string())?;

        let req = ["rm", key].join(" ");
        conn.write_all(req.as_bytes())?;
        conn.shutdown(std::net::Shutdown::Write)?;

        let mut buf = String::new();
        conn.read_to_string(&mut buf)?;
        Ok(buf)
    }
}
