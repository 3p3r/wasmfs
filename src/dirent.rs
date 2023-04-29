#![allow(dead_code)] // todo: remove this once all functions are implemented
use crate::constants::constants;

pub struct Dirent {
    // https://github.com/streamich/memfs/blob/9aba94322789d85da41905e1aed1e20e8ffe75ec/src/Dirent.ts#L10
    pub name: String,
    pub mode: usize,
}

impl Dirent {
    fn _check_mode_property(&self, property: usize) -> bool {
        self.mode & constants::S_IFMT == property
    }

    pub fn is_directory(&self) -> bool {
        self._check_mode_property(constants::S_IFDIR)
    }

    pub fn is_file(&self) -> bool {
        self._check_mode_property(constants::S_IFREG)
    }

    pub fn is_block_device(&self) -> bool {
        self._check_mode_property(constants::S_IFBLK)
    }

    pub fn is_character_device(&self) -> bool {
        self._check_mode_property(constants::S_IFCHR)
    }

    pub fn is_symbolic_link(&self) -> bool {
        self._check_mode_property(constants::S_IFLNK)
    }

    pub fn is_fifo(&self) -> bool {
        self._check_mode_property(constants::S_IFIFO)
    }

    pub fn is_socket(&self) -> bool {
        self._check_mode_property(constants::S_IFSOCK)
    }
}
