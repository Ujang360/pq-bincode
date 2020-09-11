use queue_file::QueueFile;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Mutex;

pub trait IBincodeSerializable<T = Self>
where
    Self: DeserializeOwned + Serialize + Clone + Send + Sized,
{
    fn from_bincode(bincode_slice: &[u8]) -> Option<Self> {
        bincode::deserialize(bincode_slice).ok()
    }

    fn to_bincode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

pub struct PQBincode<T>
where
    T: IBincodeSerializable,
{
    qfq: Mutex<QueueFile>,
    file_path: String,
    marker: PhantomData<T>,
}

impl<T> PQBincode<T>
where
    T: IBincodeSerializable,
{
    pub fn new<S: AsRef<Path> + ToString>(path: S) -> IOResult<Self> {
        let file_path = path.to_string();

        match QueueFile::open(path) {
            Err(error_result) => Err(IOError::new(IOErrorKind::Other, error_result.to_string())),
            Ok(qfq) => Ok(Self {
                qfq: Mutex::new(qfq),
                file_path,
                marker: Default::default(),
            }),
        }
    }

    pub fn enqueue(&mut self, data: T) -> IOResult<()> {
        let data_bin = data.to_bincode();

        match self.qfq.lock().unwrap().add(&data_bin[..]) {
            Err(error_result) => Err(IOError::new(IOErrorKind::Other, error_result.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn dequeue(&mut self) -> IOResult<Option<T>> {
        let mut guarded_qfq = self.qfq.lock().unwrap();

        match guarded_qfq.iter().next() {
            None => Ok(None),
            Some(data_bin) => match T::from_bincode(&data_bin) {
                None => Err(IOError::new(
                    IOErrorKind::InvalidData,
                    "Cannot deserialize data, invalid format!",
                )),
                Some(data) => {
                    if let Err(error_result) = guarded_qfq.remove() {
                        return Err(IOError::new(IOErrorKind::Other, error_result.to_string()));
                    }

                    Ok(Some(data))
                }
            },
        }
    }

    pub fn cancellable_dequeue<F>(&mut self, doubtful_dequeue_task: F) -> IOResult<bool>
    where
        F: FnOnce(T) -> bool,
    {
        let mut guarded_qfq = self.qfq.lock().unwrap();

        match guarded_qfq.iter().next() {
            None => Ok(false),
            Some(data_bin) => match T::from_bincode(&data_bin) {
                None => Err(IOError::new(
                    IOErrorKind::InvalidData,
                    "Cannot deserialize data, invalid format!",
                )),
                Some(data) => {
                    if doubtful_dequeue_task(data) {
                        if let Err(error_result) = guarded_qfq.remove() {
                            return Err(IOError::new(IOErrorKind::Other, error_result.to_string()));
                        }

                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
            },
        }
    }

    pub fn count(&self) -> IOResult<usize> {
        Ok(self.qfq.lock().unwrap().size())
    }

    pub fn get_persistent_path(&self) -> String {
        self.file_path.clone()
    }
}
