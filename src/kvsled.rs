use {
    crate::{KvsEngine, KvsError, Result},
    sled::*,
};

pub struct SledKvsEngine{
    engine: Db,
}

impl SledKvsEngine {
    pub fn new(db:Db) -> Self {
        SledKvsEngine{ engine:db}
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.engine.set(key, value.as_bytes())?;
        self.engine.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.engine.get(key)? {
            Some(v) => {
                let v: &[u8] = v.as_ref();
                let v = String::from_utf8_lossy(v).to_string();
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let value = self.engine
            .del(key.clone())?;
        self.engine.flush()?;
        if let None = value {
            return Err(KvsError::Remove(key).into());
        }
        Ok(())
    }
}
