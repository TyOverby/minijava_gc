#![feature(libc)]
extern crate libc;

use defs::*;
use std::ptr;
use std::mem;

mod defs;


static mut gc: Option<*mut Gc> = None; //ptr::null_mut();

#[repr(C)]
pub unsafe extern fn gc_init() {
    let gc_p = libc::malloc(mem::size_of::<Gc>() as libc::size_t) as *mut Gc;
    gc = Some(gc_p);
}

pub unsafe extern fn gc_start_class(id: usize) {
    (*gc.unwrap()).add_class(id);
}

pub unsafe extern fn gc_set_class_size(size: usize) {
    (*gc.unwrap()).get_class_builder().unwrap().set_size(size);
}

pub unsafe extern fn gc_add_class_ptr(offset: usize) {
    (*gc.unwrap()).get_class_builder().unwrap().add_ptr(offset);
}

pub unsafe extern fn gc_alloc(id: usize) {
    (*gc.unwrap()).alloc(id);
}
