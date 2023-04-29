#![allow(non_camel_case_types)]

use js_sys::{Error as JsError, Reflect};
use std::fmt::{Display, Formatter, Result};

pub enum FSError {
    ENOENT,
    EBADF,
    EINVAL,
    EPERM,
    EPROTO,
    EEXIST,
    ENOTDIR,
    EMFILE,
    EACCES,
    EISDIR,
    ENOTEMPTY,
    ENOSYS,
    ERR_FS_EISDIR,
}

impl From<FSError> for String {
    fn from(err: FSError) -> Self {
        match err {
            FSError::ENOENT => String::from("ENOENT"),
            FSError::EBADF => String::from("EBADF"),
            FSError::EINVAL => String::from("EINVAL"),
            FSError::EPERM => String::from("EPERM"),
            FSError::EPROTO => String::from("EPROTO"),
            FSError::EEXIST => String::from("EEXIST"),
            FSError::ENOTDIR => String::from("ENOTDIR"),
            FSError::EMFILE => String::from("EMFILE"),
            FSError::EACCES => String::from("EACCES"),
            FSError::EISDIR => String::from("EISDIR"),
            FSError::ENOTEMPTY => String::from("ENOTEMPTY"),
            FSError::ENOSYS => String::from("ENOSYS"),
            FSError::ERR_FS_EISDIR => String::from("ERR_FS_EISDIR"),
        }
    }
}

impl Display for FSError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            FSError::ENOENT => write!(f, "ENOENT: no such file or directory"),
            FSError::EBADF => write!(f, "EBADF: bad file descriptor"),
            FSError::EINVAL => write!(f, "EINVAL: invalid argument"),
            FSError::EPERM => write!(f, "EPERM: operation not permitted"),
            FSError::EPROTO => write!(f, "EPROTO: protocol error"),
            FSError::EEXIST => write!(f, "EEXIST: file already exists"),
            FSError::ENOTDIR => write!(f, "ENOTDIR: not a directory"),
            FSError::EMFILE => write!(f, "EMFILE: too many open files"),
            FSError::EACCES => write!(f, "EACCES: permission denied"),
            FSError::EISDIR => write!(f, "EISDIR: illegal operation on a directory"),
            FSError::ENOTEMPTY => write!(f, "ENOTEMPTY: directory not empty"),
            FSError::ENOSYS => write!(f, "ENOSYS: function not implemented"),
            FSError::ERR_FS_EISDIR => write!(f, "ERR_FS_EISDIR: illegal operation on a directory"),
        }
    }
}

pub fn create_error(code: FSError, func: Option<String>, paths: Option<Vec<String>>) -> JsError {
    let func = func.unwrap_or(String::from("unknown"));
    let paths = paths.unwrap_or(Vec::new());
    let message = format!("{}@{}: {}", code, func, paths.join(", "));
    let code = String::from(code);
    let error = JsError::new(&message);
    Reflect::set(&error, &"code".into(), &code.into()).unwrap();
    error
}
