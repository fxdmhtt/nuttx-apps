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

use crate::{binding::lvgl::*, runtime::lvgl::event};

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

#[derive(Error, Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[error("The LVGL object expected is a null pointer actually!")]
pub struct NullPtr;

#[derive(Error, Debug)]
pub enum LVGLError {
    #[error(transparent)]
    Deleted(#[from] Deleted),
    #[error(transparent)]
    NullPtr(#[from] NullPtr),
}

#[derive(Debug)]
pub struct LvObj(NonNull<lv_obj_t>);

impl TryFrom<*mut lv_obj_t> for LvObj {
    type Error = LVGLError;

    fn try_from(value: *mut lv_obj_t) -> Result<Self, Self::Error> {
        NonNull::new(value)
            .ok_or(NullPtr)
            .map(LvObj)
            .map_err(Into::into)
    }
}

impl From<LvObj> for NonNull<lv_obj_t> {
    fn from(value: LvObj) -> Self {
        value.0
    }
}

impl From<LvObj> for *mut lv_obj_t {
    fn from(value: LvObj) -> Self {
        Into::<NonNull<_>>::into(value).as_ptr()
    }
}

impl From<&LvObj> for NonNull<lv_obj_t> {
    fn from(value: &LvObj) -> Self {
        value.0
    }
}

impl From<&LvObj> for *mut lv_obj_t {
    fn from(value: &LvObj) -> Self {
        Into::<NonNull<_>>::into(value).as_ptr()
    }
}

impl LvObj {
    pub fn from(obj: *mut lv_obj_t) -> LvObjHandle {
        match unsafe { LVOBJ_TABLE.get(&obj) } {
            Some(owner) => LvObjHandle(Rc::downgrade(owner)),
            None => LvObjHandle::try_from(obj).unwrap(),
        }
    }
}

#[derive(Debug, Default)]
pub struct LvObjHandle(Weak<LvObj>);

impl Clone for LvObjHandle {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl TryFrom<*mut lv_obj_t> for LvObjHandle {
    type Error = LVGLError;

    fn try_from(value: *mut lv_obj_t) -> Result<Self, Self::Error> {
        #[cfg(debug_assertions)]
        {
            // This is a simple stress test to check the reliability of `HashMap`.
            // This code should be disabled in production environments.
            #[cfg(target_feature = "sse2")]
            let mut map = BTreeMap::new();
            #[cfg(not(target_feature = "sse2"))]
            let mut map = HashMap::new();

            for _ in 0..0xff {
                let obj = unsafe { lv_obj_create(lv_screen_active()) };
                unsafe { lv_obj_delete(obj) };
                let owner = Rc::new(NonNull::new(obj).unwrap());
                map.insert(obj, owner);
            }
        }

        let owner = Rc::new(value.try_into()?);
        let weakref = Rc::downgrade(&owner);
        #[cfg(debug_assertions)]
        {
            assert!(unsafe { LVOBJ_TABLE.insert(value, owner) }.is_none());
        }
        #[cfg(not(debug_assertions))]
        {
            unsafe { LVOBJ_TABLE.insert(value, owner) };
        }

        let obj = LvObjHandle(weakref);

        #[cfg(debug_assertions)]
        {
            let evt = event::add(&obj, LV_EVENT_DELETE, move |e| {
                debug_assert_eq!(value, unsafe { lv_event_get_target(e) });
                if let Some(obj) = unsafe { LVOBJ_TABLE.remove(&lv_event_get_target(e)) } {
                    debug_assert_eq!(Rc::strong_count(&obj), 1);
                } else {
                    panic!();
                }
            });

            let obj = obj.try_get().unwrap();
            let cnt = unsafe { lv_obj_get_event_count(obj) };
            debug_assert!(cnt >= 2);
            debug_assert_eq!(unsafe { lv_obj_get_event_dsc(obj, cnt - 2) }, evt);
            debug_assert_eq!(
                unsafe { lv_event_dsc_get_user_data(lv_obj_get_event_dsc(obj, cnt - 1)) },
                std::ptr::null_mut()
            );
        }

        #[cfg(not(debug_assertions))]
        let _ = event::add(&obj, LV_EVENT_DELETE, |e| {
            unsafe { LVOBJ_TABLE.remove(&lv_event_get_target(e)) };
        });

        Ok(obj)
    }
}

impl From<LvObjHandle> for Weak<LvObj> {
    fn from(value: LvObjHandle) -> Self {
        value.0
    }
}

impl TryFrom<LvObjHandle> for Rc<LvObj> {
    type Error = LVGLError;

    fn try_from(value: LvObjHandle) -> Result<Self, Self::Error> {
        Into::<Weak<_>>::into(value)
            .upgrade()
            .ok_or(Deleted)
            .map_err(Into::into)
    }
}

impl TryFrom<LvObjHandle> for NonNull<lv_obj_t> {
    type Error = LVGLError;

    fn try_from(value: LvObjHandle) -> Result<Self, Self::Error> {
        TryFrom::try_from(value).map(|v: Rc<LvObj>| v.0)
    }
}

impl TryFrom<LvObjHandle> for *mut lv_obj_t {
    type Error = LVGLError;

    fn try_from(value: LvObjHandle) -> Result<Self, Self::Error> {
        TryFrom::try_from(value).map(NonNull::as_ptr)
    }
}

impl From<&LvObjHandle> for Weak<LvObj> {
    fn from(value: &LvObjHandle) -> Self {
        value.clone().into()
    }
}

impl TryFrom<&LvObjHandle> for Rc<LvObj> {
    type Error = LVGLError;

    fn try_from(value: &LvObjHandle) -> Result<Self, Self::Error> {
        Into::<Weak<_>>::into(value)
            .upgrade()
            .ok_or(Deleted)
            .map_err(Into::into)
    }
}

impl TryFrom<&LvObjHandle> for NonNull<lv_obj_t> {
    type Error = LVGLError;

    fn try_from(value: &LvObjHandle) -> Result<Self, Self::Error> {
        TryFrom::try_from(value).map(|v: Rc<LvObj>| v.0)
    }
}

impl TryFrom<&LvObjHandle> for *mut lv_obj_t {
    type Error = LVGLError;

    fn try_from(value: &LvObjHandle) -> Result<Self, Self::Error> {
        TryFrom::try_from(value).map(NonNull::as_ptr)
    }
}

impl LvObjHandle {
    /// Try to get the underlying `lv_obj_t *`.
    ///
    /// Returns `Err(LVGLError::Deleted)` if the object has been deleted.
    pub fn try_get(&self) -> Result<*mut lv_obj_t, LVGLError> {
        self.try_into()
    }
}
