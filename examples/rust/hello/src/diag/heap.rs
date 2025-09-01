#![allow(dead_code)]
#![allow(static_mut_refs)]
#![allow(unused_imports)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::{c_char, c_int, c_void};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use itertools::Itertools;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Entry {
    pub ptr: *mut u8,
    pub size: usize,
}

unsafe impl Sync for Entry {}

impl Entry {
    pub const fn new() -> Self {
        Self {
            ptr: null_mut(),
            size: 0,
        }
    }
}

fn identity<T>(x: T) -> T {
    x
}

struct CountingAllocator {
    allocated: AtomicUsize,
    traced: AtomicBool,
}

unsafe impl Sync for CountingAllocator {}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null()
            && self.traced.load(Ordering::SeqCst)
            && !match (ptr as usize, layout.size()) {
                // Phase 1
                (0x00007ffff3c163c0, 176) => true, // runnable thread_id thread_local! Thread Inner
                (0x00007ffff3c16478, 16) => true, // runnable thread_id thread_local! Storage try_initialize
                (0x00007ffff3c16498, 504) => true, // async_executor::state.queue.push(runnable)
                (0x00007ffff3c167b0, 1024) => true, // println! LineWriter::with_capacity(1024, inner)
                (0x00007ffff3c16bb8, 72) => true,   // println! write_fmt Mutex.get
                // Phase 2
                (0x00007ffff3c1a558, 64) => true, // EFFECT_STACK.push
                (0x00007ffff3c1a6a8, 64) => true, // MEMO_STACK.push
                (0x00007ffff3c1ab40, 4360) => true, // LruCache::new HashMap::with_capacity
                (0x00007ffff3c1a6f0, 48) => true, // LruCache::new head
                (0x00007ffff3c1a728, 48) => true, // LruCache::new tail
                (0x00007ffff3c1a5a0, 24) => true, // Memo store_in_cache Rc::new(val)
                (0x00007ffff3c1a888, 48) => true, // Memo store_in_cache CACHE.put
                (0x00007ffff3c1c2d0, 16) => true, // event::add evt2.closure
                (_, 504) => true, // rust_delay_wake wake schedule state.queue.push(runnable)
                (_, 144) => true, // tasks_cleanup_in_background
                // reason not founded
                _ => false,
            }
        {
            if let Some(entry) = unsafe { RECORDS.iter_mut() }.find(|x| x.ptr.is_null()) {
                self.allocated.fetch_add(layout.size(), Ordering::SeqCst);
                *entry = Entry {
                    ptr,
                    size: layout.size(),
                };
            } else {
                unreachable!()
            }
        }

        // self.traced.store(false, Ordering::SeqCst);
        // let _result = {
        //     let mut frames: *mut *mut c_char = null_mut();
        //     let size = unsafe { get_backtrace(&mut frames as *mut *mut *mut c_char) };
        //     let result = unsafe { std::slice::from_raw_parts(frames, size as usize) }
        //         .iter()
        //         .map(|&frame| unsafe { CStr::from_ptr(frame).to_string_lossy().into_owned() })
        //         .map(|s| {
        //             {
        //                 if let Some(hex) = s.strip_prefix("0x") {
        //                     usize::from_str_radix(hex, 16)
        //                 } else {
        //                     s.parse()
        //                 }
        //             }
        //             .expect(&s)
        //         })
        //         .collect::<Vec<_>>();
        //     unsafe { free(frames as *mut c_void) };
        //     result
        // };
        // self.traced.store(true, Ordering::SeqCst);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);

        if let Some(entry) = unsafe { RECORDS.iter_mut() }.find(|x| x.ptr == ptr) {
            self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
            *entry = Entry::new();
        }
    }
}

#[global_allocator]
static A: CountingAllocator = CountingAllocator {
    allocated: AtomicUsize::new(0),
    traced: AtomicBool::new(true),
};
static mut RECORDS: [Entry; 0x200] = [Entry::new(); 0x200];

pub fn allocator_stats() -> usize {
    A.allocated.load(Ordering::SeqCst)
}

pub fn allocator_details() -> Vec<&'static Entry> {
    unsafe { RECORDS.iter() }
        .filter(|x| !x.ptr.is_null())
        .collect::<Vec<_>>()
}

pub fn allocator_zero() {
    unsafe { RECORDS = [Entry::new(); 0x200] };
    A.allocated.store(0, Ordering::SeqCst);
}

extern "C" {
    fn free(ptr: *mut c_void);
}

extern "C" {
    fn idle(size: usize);
    fn get_backtrace(out_frames: *mut *mut *mut c_char) -> c_int;
}
