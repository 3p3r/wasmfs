use crate::{node::Node, volume::Volume};
use js_sys::{Array, Date, JsString, Object, Reflect};
use once_cell::sync::Lazy;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Weak},
};

static mut LINK_REGISTRY: Lazy<HashMap<usize, Arc<RefCell<Link>>>> = Lazy::new(|| HashMap::new());
static mut LINK_REGISTRY_ID: usize = 0;

pub struct Link {
    registry_id: usize,

    pub vol: Arc<RefCell<Volume>>,

    pub parent: Option<Weak<RefCell<Link>>>,

    pub children: HashMap<String, Arc<RefCell<Link>>>,

    // Path to this node as Array: ['usr', 'bin', 'node'].
    _steps: Vec<String>,

    pub node: Option<Arc<RefCell<Node>>>,

    pub ino: usize,

    pub length: usize,

    pub name: String,
}

impl Link {
    pub fn get_parent(&self) -> Option<Arc<RefCell<Link>>> {
        match &self.parent {
            Some(parent) => match parent.upgrade() {
                Some(parent) => Some(parent),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_steps(&mut self) -> &mut Vec<String> {
        &mut self._steps
    }

    pub fn set_steps(&mut self, steps: Vec<String>) {
        self._steps = steps;
        for (child, link) in self.children.iter() {
            if child == "." || child == ".." {
                continue;
            }
            link.as_ref().borrow_mut().sync_steps();
        }
    }

    pub fn new(
        vol: Arc<RefCell<Volume>>,
        parent: Option<Weak<RefCell<Link>>>,
        name: String,
    ) -> Arc<RefCell<Link>> {
        let registry_id = unsafe {
            LINK_REGISTRY_ID += 1;
            LINK_REGISTRY_ID
        };
        let mut link = Link {
            registry_id,
            vol,
            parent,
            children: HashMap::new(),
            _steps: Vec::new(),
            node: None,
            ino: 0,
            length: 0,
            name,
        };
        link.sync_steps();
        let link = Arc::new(RefCell::new(link));
        unsafe {
            LINK_REGISTRY.insert(registry_id, link.clone());
        }
        link
    }

    pub fn set_node(&mut self, node: Arc<RefCell<Node>>) {
        self.node = Some(node);
    }

    pub fn get_node(&mut self) -> Arc<RefCell<Node>> {
        self.node.as_ref().unwrap().clone()
    }

    pub fn create_child(
        &mut self,
        name: String,
        node: Option<Arc<RefCell<Node>>>,
    ) -> Arc<RefCell<Link>> {
        let node = node.unwrap_or(self.vol.as_ref().borrow_mut().create_node(None, None));
        let this_ = unsafe { LINK_REGISTRY.get(&self.registry_id).unwrap().clone() };
        let link = Link::new(self.vol.clone(), Some(Arc::downgrade(&this_)), name.clone());
        link.as_ref().borrow_mut().set_node(node.clone());
        if node.as_ref().borrow().is_directory() {
            let mut link_ref = link.as_ref().borrow_mut();
            link_ref.children.insert(".".to_string(), link.clone());
            let curr_nlink = &node.borrow().get_nlink();
            node.borrow_mut().set_nlink(curr_nlink + 1);
        }
        self.set_child(name, Some(link.clone()));
        link
    }

    pub fn set_child(
        &mut self,
        name: String,
        link: Option<Arc<RefCell<Link>>>,
    ) -> Arc<RefCell<Link>> {
        let this_ = unsafe { LINK_REGISTRY.get(&self.registry_id).unwrap().clone() };
        let link = link.unwrap_or(Link::new(
            self.vol.clone(),
            Some(Arc::downgrade(&this_)),
            name.clone(),
        ));
        self.children.insert(name, link.clone());
        link.as_ref().borrow_mut().parent = Some(Arc::downgrade(&this_));
        self.length += 1;
        let node = link.borrow_mut().get_node();
        let node = node.as_ref().borrow();
        if node.is_directory() {
            link.borrow_mut()
                .children
                .insert("..".to_string(), this_.clone());
            let curr_nlink = self.get_node().as_ref().borrow().get_nlink();
            self.get_node()
                .as_ref()
                .borrow_mut()
                .set_nlink(curr_nlink + 1);
        }
        self.get_node()
            .as_ref()
            .borrow_mut()
            .set_mtime(Date::new_0());
        // this.emit('child:add', link, this);
        link.clone()
    }

    pub fn delete_child(&mut self, link: Arc<RefCell<Link>>) {
        let node = link.borrow_mut().get_node();
        let node = node.as_ref().borrow();
        if node.is_directory() {
            link.borrow_mut().children.remove("..");
            let curr_nlink = self.get_node().as_ref().borrow().get_nlink();
            self.get_node()
                .as_ref()
                .borrow_mut()
                .set_nlink(curr_nlink - 1);
        }
        self.children.remove(&link.as_ref().borrow().name);
        self.length -= 1;
        self.get_node()
            .as_ref()
            .borrow_mut()
            .set_mtime(Date::new_0());
        // this.emit('child:delete', link, this);
    }

    pub fn get_child(&mut self, name: String) -> Option<Arc<RefCell<Link>>> {
        self.children.get(&name).map(|link| link.clone())
    }

    pub fn get_path(&self) -> String {
        let mut path = String::new();
        for step in self._steps.iter() {
            path.push_str("/");
            path.push_str(step);
        }
        path
    }

    pub fn get_name(&mut self) -> String {
        self.get_steps().last().unwrap().clone()
    }

    pub fn walk(
        &mut self,
        steps: Vec<String>,
        stop: Option<usize>,
        i: Option<usize>,
    ) -> Option<Arc<RefCell<Link>>> {
        let stop = stop.unwrap_or(steps.len());
        let i = i.unwrap_or(0);
        let this_ = unsafe { LINK_REGISTRY.get(&self.registry_id).unwrap().clone() };

        if i >= steps.len() {
            return Some(this_);
        }
        if i >= stop {
            return Some(this_);
        }

        let step = steps[i].clone();
        let link = self.get_child(step.clone());
        if link.is_none() {
            return None;
        }
        return link
            .unwrap()
            .as_ref()
            .borrow_mut()
            .walk(steps.clone(), Some(stop), Some(i + 1));
    }

    pub fn to_json(&self) -> Object {
        let mut json = Object::new();
        let steps_arr = Array::new();
        for step in self._steps.iter() {
            steps_arr.push(&JsString::from(step.clone()));
        }
        Reflect::set(&mut json, &"steps".into(), &steps_arr.into()).unwrap();
        Reflect::set(&mut json, &"ino".into(), &self.ino.into()).unwrap();
        let children_arr = Array::new();
        for (name, _) in self.children.iter() {
            children_arr.push(&JsString::from(name.clone()));
        }
        Reflect::set(&mut json, &"children".into(), &children_arr.into()).unwrap();
        json
    }

    pub fn sync_steps(&mut self) {
        self.set_steps(if self.parent.is_some() {
            let steps = self.parent.as_ref().unwrap().clone();
            let steps = steps.upgrade().unwrap();
            let mut steps = steps.borrow_mut();
            let steps = steps.get_steps();
            steps.push(self.name.clone());
            steps.clone()
        } else {
            vec![self.name.clone()]
        });
    }
}
