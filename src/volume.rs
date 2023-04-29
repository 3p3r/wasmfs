#![allow(dead_code, unused)] // todo: remove this

use crate::{
    file::{self, File},
    link::Link,
    node::Node,
    util,
};
use radix_fmt::radix_36;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    sync::Arc,
};

static mut FD_COUNTER: usize = 0x7fffffff;
static mut INO_COUNTER: usize = 0;

pub struct Volume {
    // this where every allocated Link is stored
    storage: HashMap<usize, Arc<RefCell<Link>>>,

    // Hard link to the root of this volume.
    root: Option<Arc<RefCell<Link>>>,

    // A map of I-node numbers to I-nodes.
    inodes: HashMap<usize, Arc<RefCell<Node>>>,

    // A list of reusable I-node numbers that should be
    // used first before creating a new file descriptor.
    released_inos: Vec<usize>,

    // A map of file descriptors to files.
    fds: HashMap<usize, Arc<RefCell<File>>>,

    // A list of reusable (opened and closed) file descriptors, that should be
    // used first before creating a new file descriptor.
    released_fds: Vec<usize>,

    // Max number of open files.
    max_files: usize,

    // Current number of open files.
    open_files: usize,
    // todo
    // StatWatcher: new () => StatWatcher;
    // ReadStream: new (...args) => IReadStream;
    // WriteStream: new (...args) => IWriteStream;
    // FSWatcher: new () => FSWatcher;
}

impl Default for Volume {
    fn default() -> Self {
        Volume {
            storage: HashMap::new(),
            root: None,
            inodes: HashMap::new(),
            released_inos: Vec::new(),
            fds: HashMap::new(),
            released_fds: Vec::new(),
            max_files: 10000,
            open_files: 0,
        }
    }
}

impl Volume {
    pub fn new() -> Arc<RefCell<Volume>> {
        let volume = Arc::new(RefCell::new(Volume::default()));
        let link = Link::new(volume.clone(), None, "".to_string());
        let node = volume.borrow_mut().create_node(Some(true), None);
        link.borrow_mut().set_node(node.clone());
        link.borrow_mut().get_node().borrow_mut().inc_nlink();
        link.borrow_mut()
            .set_child(".".to_string(), Some(link.clone()));
        link.borrow_mut().get_node().borrow_mut().inc_nlink();
        link.borrow_mut()
            .set_child("..".to_string(), Some(link.clone()));
        link.borrow_mut().get_node().borrow_mut().inc_nlink();
        volume.borrow_mut().root = Some(link);
        volume
        // todo: Links should register themselves in volume's storage instead of their own static map
    }

    pub fn create_link(
        &mut self,
        parent: Arc<RefCell<Link>>,
        name: String,
        is_directory: Option<bool>,
        perm: Option<usize>,
    ) -> Arc<RefCell<Link>> {
        parent
            .borrow_mut()
            .create_child(name, Some(self.create_node(is_directory, perm)))
    }

    pub fn delete_link(&mut self, link: Arc<RefCell<Link>>) -> bool {
        if let Some(parent) = link.borrow_mut().get_parent() {
            parent.borrow_mut().delete_child(link.clone());
            return true;
        }
        false
    }

    fn new_ino_number(&mut self) -> usize {
        if let Some(ino) = self.released_inos.pop() {
            return ino;
        } else {
            unsafe {
                let my_ino = INO_COUNTER;
                INO_COUNTER = (INO_COUNTER + 1) % usize::MAX;
                return my_ino;
            }
        }
    }

    fn new_fd_number(&mut self) -> usize {
        if let Some(fd) = self.released_fds.pop() {
            return fd;
        } else {
            unsafe {
                let my_fd = FD_COUNTER;
                FD_COUNTER = FD_COUNTER - 1;
                return FD_COUNTER;
            }
        }
    }

    pub fn create_node(
        &mut self,
        is_directory: Option<bool>,
        perm: Option<usize>,
    ) -> Arc<RefCell<Node>> {
        let is_directory = is_directory.unwrap_or(false);
        let ino_number = self.new_ino_number();
        let mut node = Node::new(ino_number, perm);
        if is_directory {
            node.set_is_directory();
            self.inodes.insert(node.ino, Arc::new(RefCell::new(node)));
        }
        self.inodes.get(&ino_number).unwrap().clone()
    }

    fn get_node(&mut self, ino: usize) -> Option<Arc<RefCell<Node>>> {
        self.inodes.get(&ino).map(|node| node.clone())
    }

    fn delete_node(&mut self, node: Arc<RefCell<Node>>) {
        node.borrow_mut().del();
        self.inodes.remove(&node.borrow().ino);
        self.released_inos.push(node.borrow().ino);
    }

    // Generates 6 character long random string, used by `mkdtemp`.
    pub fn gen_rand_str(&self) -> String {
        let rand = format!("{}", radix_36((js_sys::Math::random() + 1.0) as usize));
        if rand.len() < 8 {
            return self.gen_rand_str();
        } else {
            return rand[2..8].to_string();
        }
    }

    // Returns a `Link` (hard link) referenced by path "split" into steps.
    pub fn get_link(&mut self, steps: Vec<String>) -> Option<Arc<RefCell<Link>>> {
        self.root.as_ref()?.borrow_mut().walk(steps, None, None)
    }

    pub fn get_link_or_throw(
        &mut self,
        filename: String,
        func_name: Option<String>,
    ) -> Result<Arc<RefCell<Link>>, js_sys::Error> {
        let steps = util::filename_to_steps(filename.clone(), None);
        let link = self.get_link(steps.clone());
        if link.is_none() {
            return Err(crate::error::create_error(
                crate::error::FSError::ENOENT,
                func_name,
                Some(vec![filename]),
            ));
        }
        Ok(link.unwrap())
    }

    // Just like `getLink`, but also dereference/resolves symbolic links.
    pub fn get_resolved_link(&mut self, filename: String) -> Option<Arc<RefCell<Link>>> {
        let steps = util::filename_to_steps(filename.clone(), None);
        let mut link = self.root.clone();
        let mut i = 0;
        while i < steps.len() {
            let step = steps[i].clone();
            link = link?.borrow_mut().get_child(step.clone());
            if link.is_none() {
                return None;
            }
            let node = link.clone()?.borrow_mut().get_node();
            if node.borrow().is_symlink() {
                let steps = node.borrow().symlink.concat();
                link = self.root.clone();
                i = 0;
                continue;
            }
            i += 1;
        }
        link
    }

    // Just like `getLinkOrThrow`, but also dereference/resolves symbolic links.
    pub fn get_resolved_link_or_throw(
        &mut self,
        filename: String,
        func_name: Option<String>,
    ) -> Result<Arc<RefCell<Link>>, js_sys::Error> {
        let link = self.get_resolved_link(filename.clone());
        if link.is_none() {
            return Err(crate::error::create_error(
                crate::error::FSError::ENOENT,
                func_name,
                Some(vec![filename]),
            ));
        }
        Ok(link.unwrap())
    }

    pub fn resolve_symlinks(&mut self, link: Arc<RefCell<Link>>) -> Option<Arc<RefCell<Link>>> {
        let path = link.borrow_mut().get_steps()[1..].to_vec().join("/");
        self.get_resolved_link(path)
    }

    // Just like `getLinkOrThrow`, but also verifies that the link is a directory.
    fn get_link_as_dir_or_throw(
        &mut self,
        filename: String,
        func_name: Option<String>,
    ) -> Result<Arc<RefCell<Link>>, js_sys::Error> {
        let link = self.get_link_or_throw(filename.clone(), func_name.clone())?;
        if !link.borrow_mut().get_node().borrow().is_directory() {
            return Err(crate::error::create_error(
                crate::error::FSError::ENOTDIR,
                func_name,
                Some(vec![filename]),
            ));
        }
        Ok(link)
    }

    // Get the immediate parent directory of the link.
    fn get_link_parent(&mut self, steps: Vec<String>) -> Option<Arc<RefCell<Link>>> {
        let limit = steps.len() - 1;
        self.root
            .as_ref()?
            .borrow_mut()
            .walk(steps, Some(limit), None)
    }

    fn get_link_parent_as_dir_or_throw(
        &mut self,
        filename: String,
        func_name: Option<String>,
    ) -> Result<Arc<RefCell<Link>>, js_sys::Error> {
        let steps = util::filename_to_steps(filename.clone(), None);
        let link = self.get_link_parent(steps.clone());
        if link.is_none() {
            return Err(crate::error::create_error(
                crate::error::FSError::ENOENT,
                func_name,
                Some(vec![filename]),
            ));
        }
        if !link
            .clone()
            .unwrap()
            .borrow_mut()
            .get_node()
            .borrow()
            .is_directory()
        {
            return Err(crate::error::create_error(
                crate::error::FSError::ENOTDIR,
                func_name,
                Some(vec![filename]),
            ));
        }
        Ok(link.unwrap())
    }

    fn get_file_by_fd(&mut self, fd: usize) -> Option<Arc<RefCell<File>>> {
        self.fds.get(&fd).map(|file| file.clone())
    }

    fn get_file_by_fd_or_throw(
        &mut self,
        fd: usize,
        func_name: Option<String>,
    ) -> Result<Arc<RefCell<File>>, js_sys::Error> {
        let file = self.get_file_by_fd(fd);
        if file.is_none() {
            return Err(crate::error::create_error(
                crate::error::FSError::EBADF,
                func_name,
                None,
            ));
        }
        Ok(file.unwrap())
    }

    // todo: fromJSON / toJSON api

    fn _to_json(
        &mut self,
        link: Option<Arc<RefCell<Link>>>,
        json: Option<js_sys::Object>,
        path: Option<String>,
    ) -> js_sys::Object {
        let mut json = json.unwrap_or(js_sys::Object::new());
        let mut is_empty = true;
        if let Some(root) = self.root.clone() {
            let mut link = link.unwrap_or(root);
            let mut children = link.borrow_mut().children.clone();

            if link.borrow_mut().get_node().borrow_mut().is_file() {
                children = HashMap::new();
                let k = link.borrow_mut().get_name();
                let v = link
                    .borrow_mut()
                    .get_parent()
                    .unwrap()
                    .borrow_mut()
                    .get_child(k.clone())
                    .unwrap();
                children.insert(k, v);
                link = link.clone().borrow_mut().get_parent().unwrap();
            }

            for (name, _) in children {
                if name == "." || name == ".." {
                    continue;
                }
                is_empty = false;

                let child = match link.clone().borrow_mut().get_child(name.clone()) {
                    Some(child) => child,
                    None => panic!("_toJSON: unexpected undefined"),
                };

                let node = child.clone().borrow_mut().get_node();
                if node.borrow_mut().is_file() {
                    let mut filename = child.clone().borrow_mut().get_path();
                    if let Some(path) = path.clone() {
                        filename = util::path_relative(path, filename, None);
                        let v = node.borrow_mut().get_string();
                        js_sys::Reflect::set(&mut json, &filename.into(), &v.into()).unwrap();
                    }
                } else if node.borrow_mut().is_directory() {
                    json = self._to_json(Some(child), Some(json), path.clone());
                }
            }

            let mut dir_path = link.clone().borrow_mut().get_path();
            if let Some(path) = path.clone() {
                dir_path = util::path_relative(path, dir_path, None);
            }
            if dir_path != "." && is_empty {
                js_sys::Reflect::set(&mut json, &dir_path.into(), &wasm_bindgen::JsValue::null())
                    .unwrap();
            }
        }
        json
    }

    pub fn to_json(
        &mut self,
        paths: Option<Vec<String>>,
        json: Option<js_sys::Object>,
        is_relative: Option<bool>,
    ) -> js_sys::Object {
        let mut links: Vec<Arc<RefCell<Link>>> = vec![];

        if let Some(paths) = paths {
            for path in paths {
                let filename = path.clone();
                let link = self.get_resolved_link(filename);
                if link.is_none() {
                    continue;
                }
                links.push(link.unwrap());
            }
        } else {
            links.push(self.root.clone().unwrap());
        }

        let mut json = json.unwrap_or(js_sys::Object::new());

        if links.is_empty() {
            return json;
        }

        for link in links {
            let is_relative = is_relative.unwrap_or(false);
            json = self._to_json(
                Some(link.clone()),
                Some(json),
                if is_relative {
                    Some(link.borrow_mut().get_path())
                } else {
                    None
                },
            );
        }

        json
    }

    pub fn from_json(&mut self, json: js_sys::Object, cwd: Option<String>) {
        let cwd = cwd.unwrap_or_else(|| crate::util::process_cwd());
        for filename in js_sys::Object::keys(&json).iter() {
            let filename = filename.as_string().unwrap();
            let data = js_sys::Reflect::get(&json, &filename.clone().into()).unwrap();
            let filename = util::path_resolve(vec![filename, cwd.clone()], None);
            if data.is_string() {
                // todo
                // let dir = util::path_dirname(filename.clone(), None);
                // self.mkdirp_base(dir, MODE::DIR);
                // self.write_file_sync(filename.clone(), data.as_string().unwrap());
            } else {
                // todo
                // self.mkdirp_base(filename.clone(), MODE::DIR);
            }
        }
    }

    pub fn from_nested_json(&mut self, json: js_sys::Object, cwd: Option<String>) {
        self.from_json(util::flatten_json(json), cwd);
    }

    pub fn reset(&mut self) {
        let def = Volume::default();
        self.fds.clear();
        // self.root.clear(); // todo
        self.inodes.clear();
        self.open_files = def.open_files;
        self.released_fds.clear();
        self.released_inos.clear();
        self.max_files = def.max_files;
        self.storage.clear();
    }

    pub fn mount_sync(&mut self, mount_point: String, json: js_sys::Object) {
        self.from_json(json, Some(mount_point));
    }
}
