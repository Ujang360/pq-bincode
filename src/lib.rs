use queue_file::QueueFile;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult};
use std::marker::PhantomData;
use std::path::Path;

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
    qfq: QueueFile,
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
                qfq,
                file_path,
                marker: Default::default(),
            }),
        }
    }

    pub fn enqueue(&mut self, data: T) -> IOResult<()> {
        let data_bin = data.to_bincode();

        match self.qfq.add(&data_bin[..]) {
            Err(error_result) => Err(IOError::new(IOErrorKind::Other, error_result.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn enqueue_all(&mut self, data: Vec<T>) -> IOResult<()> {
        for data_item in data {
            self.enqueue(data_item)?;
        }

        Ok(())
    }

    pub fn dequeue(&mut self) -> IOResult<Option<T>> {
        let mut dequeued_item = None;
        self.cancellable_dequeue(|next_item| {
            dequeued_item = Some(next_item);
            true
        })?;

        Ok(dequeued_item)
    }

    pub fn dequeue_all(&mut self) -> IOResult<Vec<T>> {
        let mut dequeued_items = Vec::new();

        while self.cancellable_dequeue(|next_item| {
            dequeued_items.push(next_item);
            true
        })? {}

        Ok(dequeued_items)
    }

    pub fn cancellable_dequeue<F>(&mut self, doubtful_dequeue_task: F) -> IOResult<bool>
    where
        F: FnOnce(T) -> bool,
    {
        match self.qfq.peek() {
            Err(error_result) => Err(IOError::new(IOErrorKind::Other, error_result.to_string())),
            Ok(data_bin) => {
                if data_bin.is_none() {
                    return Ok(false);
                }

                let data_bin = data_bin.unwrap();

                match T::from_bincode(&data_bin) {
                    None => Err(IOError::new(
                        IOErrorKind::InvalidData,
                        "Cannot deserialize data, invalid format!",
                    )),
                    Some(data) => {
                        if doubtful_dequeue_task(data) {
                            if let Err(error_result) = self.qfq.remove() {
                                return Err(IOError::new(IOErrorKind::Other, error_result.to_string()));
                            }

                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.qfq.size()
    }

    pub fn get_persistent_path(&self) -> String {
        self.file_path.clone()
    }
}
