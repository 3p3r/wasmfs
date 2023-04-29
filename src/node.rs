use crate::constants::constants;
use js_sys::{Array, Date, JsString, Object, Reflect};

pub struct Node {
    pub ino: usize,

    _uid: usize,
    _gid: usize,

    _atime: Date,
    _mtime: Date,
    _ctime: Date,

    pub buf: Option<Vec<u8>>,

    _perm: usize,

    pub mode: usize,

    _nlink: usize,

    pub symlink: Vec<String>,
}

impl Node {
    pub fn new(ino: usize, perm: Option<usize>) -> Self {
        let perm = perm.unwrap_or(0o666);
        let mode = constants::S_IFREG | perm;
        Self {
            ino,
            _uid: 0,
            _gid: 0,
            _atime: Date::new_0(),
            _mtime: Date::new_0(),
            _ctime: Date::new_0(),
            buf: None,
            _perm: perm,
            mode,
            _nlink: 1,
            symlink: Vec::new(),
        }
    }

    // getter and setter for ctime
    pub fn get_ctime(&self) -> Date {
        self._ctime.clone()
    }
    pub fn set_ctime(&mut self, ctime: Date) {
        self._ctime = ctime;
    }

    // getter and setter for uid
    pub fn set_uid(&mut self, uid: usize) {
        self._uid = uid;
        self.set_ctime(Date::new_0());
    }
    pub fn get_uid(&self) -> usize {
        self._uid
    }

    // getter and setter for gid
    pub fn set_gid(&mut self, gid: usize) {
        self._gid = gid;
        self.set_ctime(Date::new_0());
    }
    pub fn get_gid(&self) -> usize {
        self._gid
    }

    // getter and setter for atime
    pub fn get_atime(&self) -> Date {
        self._atime.clone()
    }
    pub fn set_atime(&mut self, atime: Date) {
        self._atime = atime;
        self.set_ctime(Date::new_0());
    }

    // getter and setter for mtime
    pub fn get_mtime(&self) -> Date {
        self._mtime.clone()
    }
    pub fn set_mtime(&mut self, mtime: Date) {
        self._mtime = mtime;
        self.set_ctime(Date::new_0());
    }

    // getter and setter for perm
    pub fn get_perm(&self) -> usize {
        self._perm
    }
    pub fn set_perm(&mut self, perm: usize) {
        self._perm = perm;
        self.set_ctime(Date::new_0());
    }

    // getter and setter for nlink
    pub fn get_nlink(&self) -> usize {
        self._nlink
    }
    pub fn set_nlink(&mut self, nlink: usize) {
        self._nlink = nlink;
        self.set_ctime(Date::new_0());
    }
    pub fn inc_nlink(&mut self) {
        self._nlink += 1;
        self.set_ctime(Date::new_0());
    }
    pub fn dec_nlink(&mut self) {
        self._nlink -= 1;
        self.set_ctime(Date::new_0());
    }

    pub fn touch(&mut self) {
        self.set_mtime(Date::new_0());
        // todo: this.emit('change', this);
    }

    pub fn get_string(&mut self) -> String {
        self.set_atime(Date::new_0());
        String::from_utf8(self.get_buffer()).unwrap()
    }

    pub fn set_string(&mut self, string: String) {
        self.buf = Some(string.into_bytes());
        self.touch();
    }

    pub fn get_buffer(&mut self) -> Vec<u8> {
        self.set_atime(Date::new_0());
        match &self.buf {
            Some(buf) => buf.clone(),
            None => {
                let buf = Vec::new();
                self.set_buffer(buf.clone());
                buf
            }
        }
    }

    pub fn set_buffer(&mut self, buffer: Vec<u8>) {
        self.buf = Some(buffer);
        self.touch();
    }

    pub fn get_size(&mut self) -> usize {
        match &self.buf {
            Some(buf) => buf.len(),
            None => 0,
        }
    }

    pub fn set_mode_property(&mut self, property: usize) {
        self.mode = (self.mode & !constants::S_IFMT) | property;
    }

    pub fn set_is_file(&mut self) {
        self.set_mode_property(constants::S_IFREG);
    }

    pub fn set_is_directory(&mut self) {
        self.set_mode_property(constants::S_IFDIR);
    }

    pub fn set_is_symlink(&mut self) {
        self.set_mode_property(constants::S_IFLNK);
    }

    pub fn is_file(&self) -> bool {
        (self.mode & constants::S_IFMT) == constants::S_IFREG
    }

    pub fn is_directory(&self) -> bool {
        (self.mode & constants::S_IFMT) == constants::S_IFDIR
    }

    pub fn is_symlink(&self) -> bool {
        (self.mode & constants::S_IFMT) == constants::S_IFLNK
    }

    pub fn make_symlink(&mut self, steps: Vec<String>) {
        self.set_is_symlink();
        self.symlink = steps;
    }

    pub fn write(
        &mut self,
        buf: Vec<u8>,
        off: Option<usize>,
        len: Option<usize>,
        pos: Option<usize>,
    ) -> usize {
        let off = off.unwrap_or(0);
        let len = len.unwrap_or(buf.len());
        let pos = pos.unwrap_or(0);
        let this_buf = self.ensure_buffer();

        if pos + len > this_buf.len() {
            this_buf.resize(pos + len, 0);
        }

        this_buf.splice(pos..pos + len, buf[off..off + len].iter().cloned());
        self.touch();
        len
    }

    pub fn read(
        &mut self,
        mut buf: Vec<u8>,
        off: Option<usize>,
        len: Option<usize>,
        pos: Option<usize>,
    ) -> usize {
        self._atime = Date::new_0();

        let off = off.unwrap_or(0);
        let len = len.unwrap_or(buf.len());
        let pos = pos.unwrap_or(0);
        let this_buf = self.ensure_buffer();

        let mut actual_len = len;
        if actual_len > buf.len() {
            actual_len = buf.len();
        }
        if actual_len + pos > this_buf.len() {
            actual_len = this_buf.len() - pos;
        }

        buf.splice(
            off..off + actual_len,
            this_buf[pos..pos + actual_len].iter().cloned(),
        );
        actual_len
    }

    pub fn truncate(&mut self, len: usize) {
        if len == 0 {
            self.buf = Some(Vec::new());
        } else {
            let this_buf = self.ensure_buffer();
            this_buf.resize(len, 0);
        }
        self.touch();
    }

    pub fn chmod(&mut self, perm: usize) {
        self.set_perm(perm);
        self.mode = (self.mode & !0o777) | perm;
        self.touch();
    }

    pub fn chown(&mut self, uid: usize, gid: usize) {
        self.set_uid(uid);
        self.set_gid(gid);
        self.touch();
    }

    pub fn can_read(&self, uid: Option<usize>, gid: Option<usize>) -> bool {
        let uid = uid.unwrap_or(0);
        let gid = gid.unwrap_or(0);

        if self.get_perm() & constants::S_IROTH != 0 {
            return true;
        }

        if gid == self.get_gid() {
            if self.get_perm() & constants::S_IRGRP != 0 {
                return true;
            }
        }

        if uid == self.get_uid() {
            if self.get_perm() & constants::S_IRUSR != 0 {
                return true;
            }
        }

        false
    }

    pub fn can_write(&self, uid: Option<usize>, gid: Option<usize>) -> bool {
        let uid = uid.unwrap_or(0);
        let gid = gid.unwrap_or(0);

        if self.get_perm() & constants::S_IWOTH != 0 {
            return true;
        }

        if gid == self.get_gid() {
            if self.get_perm() & constants::S_IWGRP != 0 {
                return true;
            }
        }

        if uid == self.get_uid() {
            if self.get_perm() & constants::S_IWUSR != 0 {
                return true;
            }
        }

        false
    }

    pub fn del(&mut self) {
        // todo: this.emit('delete', this);
    }

    pub fn to_json(&mut self) -> Object {
        let mut json = Object::new();
        Reflect::set(&mut json, &"ino".into(), &self.ino.into()).unwrap();
        Reflect::set(&mut json, &"uid".into(), &self.get_uid().into()).unwrap();
        Reflect::set(&mut json, &"gid".into(), &self.get_gid().into()).unwrap();
        Reflect::set(&mut json, &"atime".into(), &self.get_atime().into()).unwrap();
        Reflect::set(&mut json, &"mtime".into(), &self.get_mtime().into()).unwrap();
        Reflect::set(&mut json, &"ctime".into(), &self.get_ctime().into()).unwrap();
        Reflect::set(&mut json, &"perm".into(), &self.get_perm().into()).unwrap();
        Reflect::set(&mut json, &"mode".into(), &self.mode.into()).unwrap();
        Reflect::set(&mut json, &"nlink".into(), &self.get_nlink().into()).unwrap();
        let symlinks_arr = Array::new();
        for symlink in self.symlink.iter() {
            symlinks_arr.push(&symlink.into());
        }
        Reflect::set(&mut json, &"symlink".into(), &symlinks_arr.into()).unwrap();
        Reflect::set(
            &mut json,
            &"data".into(),
            &JsString::from(self.get_string().clone()),
        )
        .unwrap();
        json
    }

    fn ensure_buffer(&mut self) -> &mut Vec<u8> {
        if self.buf.is_none() {
            self.buf = Some(Vec::new());
        }
        self.buf.as_mut().unwrap()
    }
}
