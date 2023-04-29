use crate::{link::Link, node::Node, stats::Stats};
use std::{cell::RefCell, sync::Arc};

pub struct File {
    pub fd: usize,
    pub link: Arc<RefCell<Link>>,
    pub node: Arc<RefCell<Node>>,
    pub position: usize,
    pub flags: usize,
}

impl File {
    pub fn new(
        link: Arc<RefCell<Link>>,
        node: Arc<RefCell<Node>>,
        flags: usize,
        fd: usize,
    ) -> Arc<RefCell<File>> {
        Arc::new(RefCell::new(File {
            fd,
            link,
            node,
            position: 0,
            flags,
        }))
    }

    pub fn get_string(&self, encoding: Option<String>) -> String {
        let encoding = encoding.unwrap_or(String::from("utf8")).to_lowercase();
        assert!(encoding == "utf8" || encoding == "utf-8");
        let mut node = self.node.borrow_mut();
        return node.get_string();
    }

    pub fn set_string(&mut self, str: String) {
        let mut node = self.node.borrow_mut();
        node.set_string(str);
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        let mut node = self.node.borrow_mut();
        return node.get_buffer();
    }

    pub fn set_buffer(&mut self, buf: Vec<u8>) {
        let mut node = self.node.borrow_mut();
        node.set_buffer(buf);
    }

    pub fn get_size(&self) -> usize {
        let mut node = self.node.borrow_mut();
        return node.get_size();
    }

    pub fn truncate(&mut self, len: usize) {
        let mut node = self.node.borrow_mut();
        node.truncate(len);
    }

    pub fn seek_to(&mut self, pos: usize) {
        self.position = pos;
    }

    pub fn stats(&self) -> Stats {
        unimplemented!();
    }

    pub fn write(
        &mut self,
        buf: Vec<u8>,
        offset: Option<usize>,
        length: Option<usize>,
        position: Option<usize>,
    ) -> usize {
        let mut node = self.node.borrow_mut();
        let bytes = node.write(buf, offset, length, position);
        self.position = position.unwrap_or(self.position) + bytes;
        return bytes;
    }

    pub fn read(
        &mut self,
        buf: Vec<u8>,
        offset: Option<usize>,
        length: Option<usize>,
        position: Option<usize>,
    ) -> usize {
        let mut node = self.node.borrow_mut();
        let bytes = node.read(buf, offset, length, position);
        self.position = position.unwrap_or(self.position) + bytes;
        return bytes;
    }

    pub fn chmod(&mut self, perm: usize) {
        let mut node = self.node.borrow_mut();
        node.chmod(perm);
    }

    pub fn chown(&mut self, uid: usize, gid: usize) {
        let mut node = self.node.borrow_mut();
        node.chown(uid, gid);
    }
}
