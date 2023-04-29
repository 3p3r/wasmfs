#[allow(non_camel_case_types)]
pub struct constants {
    // https://github.com/streamich/memfs/blob/9aba94322789d85da41905e1aed1e20e8ffe75ec/src/constants.ts#L1
}

impl constants {
    pub const O_RDONLY: usize = 0;
    pub const O_WRONLY: usize = 1;
    pub const O_RDWR: usize = 2;
    pub const O_CREAT: usize = 64;
    pub const O_EXCL: usize = 128;
    pub const O_NOCTTY: usize = 256;
    pub const O_TRUNC: usize = 512;
    pub const O_APPEND: usize = 1024;
    pub const O_DIRECTORY: usize = 65536;
    pub const O_NOATIME: usize = 262144;
    pub const O_NOFOLLOW: usize = 131072;
    pub const O_SYNC: usize = 1052672;
    pub const O_DIRECT: usize = 16384;
    pub const O_NONBLOCK: usize = 2048;

    pub const S_IFMT: usize = 61440;
    pub const S_IFREG: usize = 32768;
    pub const S_IFDIR: usize = 16384;
    pub const S_IFCHR: usize = 8192;
    pub const S_IFBLK: usize = 24576;
    pub const S_IFIFO: usize = 4096;
    pub const S_IFLNK: usize = 40960;
    pub const S_IFSOCK: usize = 49152;
    pub const S_IRWXU: usize = 448;
    pub const S_IRUSR: usize = 256;
    pub const S_IWUSR: usize = 128;
    pub const S_IXUSR: usize = 64;
    pub const S_IRWXG: usize = 56;
    pub const S_IRGRP: usize = 32;
    pub const S_IWGRP: usize = 16;
    pub const S_IXGRP: usize = 8;
    pub const S_IRWXO: usize = 7;
    pub const S_IROTH: usize = 4;
    pub const S_IWOTH: usize = 2;
    pub const S_IXOTH: usize = 1;

    pub const F_OK: usize = 0;
    pub const R_OK: usize = 4;
    pub const W_OK: usize = 2;
    pub const X_OK: usize = 1;

    pub const UV_FS_SYMLINK_DIR: usize = 1;
    pub const UV_FS_SYMLINK_JUNCTION: usize = 2;
    pub const UV_FS_COPYFILE_EXCL: usize = 1;
    pub const UV_FS_COPYFILE_FICLONE: usize = 2;
    pub const UV_FS_COPYFILE_FICLONE_FORCE: usize = 4;

    pub const COPYFILE_EXCL: usize = 1;
    pub const COPYFILE_FICLONE: usize = 2;
    pub const COPYFILE_FICLONE_FORCE: usize = 4;
}

pub struct S {
    //https://github.com/streamich/memfs/blob/9aba94322789d85da41905e1aed1e20e8ffe75ec/src/constants.ts#L53
}

impl S {
    pub const ISUID: usize = 0b100000000000; //  (04000)  set-user-ID (set process effective user ID on execve(2))
    pub const ISGID: usize = 0b10000000000; // (02000)  set-group-ID (set process effective group ID on execve(2); mandatory locking, as described in fcntl(2); take a new file's group from parent directory, as described in chown(2) and mkdir(2))
    pub const ISVTX: usize = 0b1000000000; // (01000)  sticky bit (restricted deletion flag, as described in unlink(2))
    pub const IRUSR: usize = 0b100000000; //  (00400)  read by owner
    pub const IWUSR: usize = 0b10000000; // (00200)  write by owner
    pub const IXUSR: usize = 0b1000000; // (00100)  execute/search by owner
    pub const IRGRP: usize = 0b100000; // (00040)  read by group
    pub const IWGRP: usize = 0b10000; // (00020)  write by group
    pub const IXGRP: usize = 0b1000; // (00010)  execute/search by group
    pub const IROTH: usize = 0b100; // (00004)  read by others
    pub const IWOTH: usize = 0b10; //  (00002)  write by others
    pub const IXOTH: usize = 0b1; //  (00001)  execute/search by others
}
