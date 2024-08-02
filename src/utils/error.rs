use crate::devices::virtio::DeviceError;

#[derive(Debug)]
pub enum OsError {
    DtbError,
}

#[derive(Debug)]
pub enum IoError {
    DeviceError(DeviceError),
    FileNotExists,
}

impl From<DeviceError> for IoError {
    fn from(value: DeviceError) -> Self {
        IoError::DeviceError(value)
    }
}

pub type IoResult<T> = Result<T, IoError>;
