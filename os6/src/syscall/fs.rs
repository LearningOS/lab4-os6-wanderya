//! File and filesystem-related syscalls

// use crate::mm::translated_byte_buffer;
// use crate::mm::translated_str;
// use crate::mm::translated_refmut;
// use crate::task::VirtAddr;
// use crate::task::current_user_token;
use crate::task::*;
use crate::fs::open_file;
use crate::fs::OpenFlags;
use crate::fs::Stat;
use crate::mm::*;
use crate::fs::*;
// use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

// YOUR JOB: 扩展 easy-fs 和内核以实现以下三个 syscall
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    let va = VirtAddr::from(_st as usize);
    let pa = translated_va2pa(current_user_token(), va);
    let ptr_st = pa as *mut Stat;

    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();

    // check if _fd exists.
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[_fd].is_none() {
        return -1;
    }
    if let Some(inode) = &inner.fd_table[_fd] {
        // let ino = inode.index_node_id() as u64;
        // let mode = inode.stat_mode();
        // let nlink = inode.nlink();
        // drop(inner); // why error?
        let (ino, mode, nlink) = inode.fstat();
        drop(inner);
        unsafe {
            // *ptr_st = Stat {
            //     dev: 0,
            //     ino,
            //     mode,
            //     nlink,
            //     pad: [0; 7]
            // }
            (*ptr_st).dev   = 0;
            (*ptr_st).ino   = ino as u64;
            (*ptr_st).mode  = mode;
            (*ptr_st).nlink = nlink;
        }
        0
    } else {
        -1
    }
}

pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let new_name = translated_str(token, _new_name);
    // check if they are the same name 
    if old_name == new_name {
        return -1;
    }

    linkat(old_name.as_str(), new_name.as_str());
    0
}

pub fn sys_unlinkat(_name: *const u8) -> isize {
    let token = current_user_token();
    let name = translated_str(token, _name);

    unlinkat(name.as_str());
    0
}
