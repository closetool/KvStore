use {
    crate::{
        errors::{KvsError, Result},
        KvsEngine,
    },
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        ffi::OsStr,
        fs::{self, File},
        io::{ Read, Seek, SeekFrom, Write},
        path::PathBuf,
    },
};

const MB: u64 = 8 * 1024 * 1024;
const THRESHOLD: u64 = 1 * MB;

pub struct KvStore {
    path: PathBuf,
    log_pointer: u64,
    logs: HashMap<u64, FileWithPos>,
    index: HashMap<String, Record>,
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let log = self.get_log(self.log_pointer)?;
        let cmd = Command::Set {
            key: key.to_string(),
            value,
        };
        let (offset, length) = log.write_log(&cmd)?;
        self.index.insert(
            key,
            Record {
                log_id: self.log_pointer,
                offset,
                length,
            },
        );
        let log = self.get_log(self.log_pointer)?;
        if log.size()? > THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let rcd = self.index.get(&key).cloned();
        Ok(match rcd {
            Some(value) => {
                let log = self.get_log(value.log_id)?;
                let cmd = log.read_from_where(value.offset, value.length)?;
                if let Command::Set { key: _, value: val } = cmd {
                    Some(val)
                } else {
                    None
                }
            }
            None => None,
        })
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if let Some(_) = self.index.remove(&key) {
            let log = self.get_log(self.log_pointer)?;
            log.write_log(&Command::Remove { key: key.clone() })?;
            if log.size()? > THRESHOLD {
                self.compact()?;
            }
        } else {
            return Err(KvsError::Remove(key).into());
        }
        Ok(())
    }
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into();
        if !path.exists() {
            fs::create_dir_all(path.clone())?;
        }
        let mut kvs = KvStore {
            log_pointer: 0,
            logs: HashMap::new(),
            path: path.clone(),
            index: HashMap::new(),
        };
        kvs.build()?;
        if kvs.log_pointer == 0 {
            kvs.new_log()?;
        }
        Ok(kvs)
    }

    pub fn gen_log_name(log_id: u64) -> String {
        format!("{}.log", log_id)
    }

    pub fn new_log(&mut self) -> Result<()> {
        self.log_pointer += 1;
        let path = self.path.clone();
        let log = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(path.join(KvStore::gen_log_name(self.log_pointer)))?;
        self.logs.insert(self.log_pointer, FileWithPos::from(log)?);
        Ok(())
    }

    fn get_log(&mut self, log_id: u64) -> Result<&mut FileWithPos> {
        self.logs
            .get_mut(&log_id)
            .ok_or(KvsError::UnKnownLog(self.log_pointer).into())
    }

    fn build(&mut self) -> Result<()> {
        let path = self.path.clone();
        let ids = read_all_log_idx_and_sort(&path)?;
        let files: Vec<Result<FileWithPos>> = ids
            .iter()
            .map(|id| KvStore::gen_log_name(id.clone()))
            .map(|name| FileWithPos::new(path.join(name)))
            .collect();

        for (i, file) in files.into_iter().enumerate() {
            let mut file = file?;
            let &id = ids.get(i).unwrap();
            file.seek(SeekFrom::Start(0))?;
            let mut decoder =
                serde_json::Deserializer::from_reader(file.fd.try_clone()?).into_iter::<Command>();
            let (mut offset, mut end) = (0u64, 0u64);
            while let Some(cmd) = decoder.next() {
                end = decoder.byte_offset() as u64;
                let cmd = cmd?;
                match cmd {
                    Command::Set { key, value: _ } => {
                        self.index.insert(
                            key,
                            Record {
                                log_id: id,
                                offset,
                                length: end - offset,
                            },
                        );
                    }
                    Command::Remove { key } => {
                        self.index.remove(&key);
                    }
                }
                offset = end;
            }
            self.logs.insert(id, file);
            self.log_pointer = id;
        }
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        let old_ptr = self.log_pointer;
        self.new_log()?;
        let logs = &mut self.logs;
        let index = &mut self.index;
        for (_, rcd) in index.iter_mut() {
            let log = logs
                .get_mut(&rcd.log_id)
                .ok_or(KvsError::UnKnownLog(rcd.log_id))?;
            let cmd = log.read_from_where(rcd.offset, rcd.length)?;
            let cur_log = logs
                .get_mut(&self.log_pointer)
                .ok_or(KvsError::UnKnownLog(self.log_pointer))?;
            let (offset, length) = cur_log.write_log(&cmd)?;
            rcd.log_id = self.log_pointer;
            rcd.offset = offset;
            rcd.length = length;
        }
        let mut ids = read_all_log_idx_and_sort(&self.path)?;
        ids = ids.into_iter().filter(|v| *v <= old_ptr).collect();
        let path = self.path.clone();
        for id in ids {
            self.logs.remove(&id);
            fs::remove_file(path.join(KvStore::gen_log_name(id)))?;
        }
        self.new_log()?;
        Ok(())
    }
}

fn read_all_log_idx_and_sort(path: &PathBuf) -> Result<Vec<u64>> {
    let mut ids: Vec<u64> = fs::read_dir(path)?
        .flat_map(|d| -> Result<_> { Ok(d?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    ids.sort_unstable();
    Ok(ids)
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

struct FileWithPos {
    fd: File,
}

impl FileWithPos {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();
        let mut fd = fs::OpenOptions::new().append(true).read(true).open(path)?;
        fd.seek(SeekFrom::End(0))?;
        Ok(FileWithPos { fd })
    }

    pub fn from(fd: File) -> Result<Self> {
        let mut fd = fd;
        fd.seek(SeekFrom::End(0))?;
        Ok(FileWithPos { fd })
    }

    pub fn write_log(&mut self, cmd: &Command) -> Result<(u64, u64)> {
        let start = self.seek(SeekFrom::End(0))?;
        let cmd = serde_json::to_vec(cmd)?;
        let length = cmd.len() as u64;
        self.fd.write_all(cmd.as_slice())?;
        Ok((start, length))
    }

    pub fn read_from_where(&mut self, pos: u64, length: u64) -> Result<Command> {
        self.seek(SeekFrom::Start(pos))?;
        let mut buf = vec![0u8; length as usize];
        let buf = buf.as_mut_slice();
        self.fd.read(buf)?;
        let cmd = serde_json::from_slice(buf)?;
        Ok(cmd)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        Ok(self.fd.seek(pos)?)
    }

    pub fn size(&mut self) -> Result<u64> {
        self.seek(SeekFrom::End(0))
    }
}

#[derive(Debug, Clone, Copy)]
struct Record {
    log_id: u64,
    offset: u64,
    length: u64,
}
