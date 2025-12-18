#![allow(static_mut_refs)]

#[cfg(target_feature = "sse2")]
use std::collections::BTreeMap;
#[cfg(not(target_feature = "sse2"))]
use std::collections::HashMap;

use std::{
    ptr::NonNull,
    rc::{Rc, Weak},
};

use once_cell::sync::Lazy;
use thiserror::Error;

use crate::runtime::lvgl::{
    event, lv_event_dsc_get_user_data, lv_event_get_target, lv_obj_get_event_count, lv_obj_get_event_dsc, lv_obj_t, LV_EVENT_DELETE,
};

// The best practice is to use `HashMap`,
// but the reason we're not using `HashMap` here is that
// it would cause an alignment exception
// in the `Group::load_aligned` function
// within the `sse2.rs` file of `hashbrown`.
#[cfg(target_feature = "sse2")]
static mut LVOBJ_TABLE: Lazy<BTreeMap<*mut lv_obj_t, Rc<LvObj>>> = Lazy::new(BTreeMap::new);
#[cfg(not(target_feature = "sse2"))]
static mut LVOBJ_TABLE: Lazy<HashMap<*mut lv_obj_t, Rc<LvObj>>> = Lazy::new(HashMap::new);

#[derive(Error, Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[error("The LVGL object has been deleted!")]
pub struct Deleted;

#[derive(Error, Debug)]
pub enum LVGLError {
    #[error(transparent)]
    Deleted(#[from] Deleted),
}

#[derive(Debug)]
pub struct LvObj(NonNull<lv_obj_t>);

impl LvObj {
    fn new(obj: *mut lv_obj_t) -> Self {
        Self(NonNull::new(obj).unwrap())
    }

    pub fn from(obj: *mut lv_obj_t) -> LvObjHandle {
        #[cfg(debug_assertions)]
        {
            // This is a simple stress test to check the reliability of `HashMap`.
            // This code should be disabled in production environments.
            #[cfg(target_feature = "sse2")]
            let mut map = BTreeMap::new();
            #[cfg(not(target_feature = "sse2"))]
            let mut map = HashMap::new();

            for _ in 0..0xff {
                let obj = unsafe { super::lv_obj_create(super::lv_screen_active()) };
                unsafe { super::lv_obj_delete(obj) };
                let owner = Rc::new(NonNull::new(obj).unwrap());
                map.insert(obj, owner);
            }
        }

        let owner = Rc::new(Self::new(obj));
        let weakref = Rc::downgrade(&owner);
        unsafe { LVOBJ_TABLE.insert(obj, owner) };

        let obj = LvObjHandle(weakref);

        {
            let evt = event::add(&obj, LV_EVENT_DELETE, |e| {
                unsafe { LVOBJ_TABLE.remove(&lv_event_get_target(e)) };
            });

            let obj = obj.try_get().unwrap();
            let cnt = unsafe { lv_obj_get_event_count(obj) };
            assert!(cnt >= 2);
            assert_eq!(unsafe { lv_obj_get_event_dsc(obj, cnt - 2) }, evt);
            assert_eq!(
                unsafe { lv_event_dsc_get_user_data(lv_obj_get_event_dsc(obj, cnt - 1)) },
                std::ptr::null_mut()
            );
        }

        obj
    }
}

#[derive(Debug, Clone, Default)]
pub struct LvObjHandle(Weak<LvObj>);

impl LvObjHandle {
    /// Try to get the underlying `lv_obj_t *`.
    ///
    /// Returns `Err(LVGLError::Deleted)` if the object has been deleted.
    pub fn try_get(&self) -> Result<*mut lv_obj_t, LVGLError> {
        let obj = self.0.upgrade().ok_or(Deleted)?;
        Ok(obj.0.as_ptr())
    }

    pub fn is_null(&self) -> bool {
        self.0.upgrade().is_none()
    }
}
