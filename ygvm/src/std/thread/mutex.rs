use crate::napi::alloc::alloc_bool;
use crate::napi::control::{exit_err, exit_ok};
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::napi_try_or_exit;
use crate::std::json::json_element::{alloc_json_element, json_element_to_native, json_element_to_object};
use crate::utils::mutex::Mutex;
use crate::vm::heap::{ObjectRef, VMHeap, VMHeapGC};
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::VMError;
use std::sync::atomic::{fence, Ordering};

pub fn alloc_mutex(mut thread: VMThreadRef, value: &ObjectSmartRef) -> Result<ObjectSmartRefNN, VMError> {
    let class = VMModuleManager::find_class(thread.vm, "std/thread/Mutex")?;
    let object = VMHeap::alloc(thread.vm, class)?;
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = object.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Mutex<ObjectRef>;
        std::ptr::write(ptr, Mutex::new(value.as_raw()));
        fence(Ordering::Release);
    }
    let init = class.find_method("__init__")?;
    let object = object.into();
    let object = thread.call_func(&object, init, &[])?;
    let object = object.deref()?;
    Ok(object)
}

pub unsafe extern "C" fn _mutex_init(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = thread.call_class("std/core/Object", "__init__", &[this]);
    let this = napi_try_or_exit!(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    this.flags.mark_uninit();
    this.flags.mark_marker();
    let this = Into::<ObjectSmartRef>::into(this);
    exit_ok(frame, &this)
}

pub unsafe extern "C" fn _mutex_uninit(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Mutex<ObjectRef>;
        std::ptr::drop_in_place(ptr);
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _mutex_mark(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    // SAFETY: Гарантия стандарта.
    let object = unsafe { std::ptr::read(mutex).into_inner() };
    let object = ObjectSmartRef::new(object);
    if let Some(object) = object.try_deref() {
        napi_try_or_exit!(VMHeapGC::gc_mark(thread, &object));
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _mutex_try_with_lock(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    let func = frame.locals.get_global("func");
    let func = ObjectSmartRef::new(func);
    let func = func.deref();
    let func = napi_try_or_exit!(func);
    if mutex.try_raw_lock(thread.0 as usize) {
        let value = thread.call_obj(&func, "__call__", &[]);
        mutex.unlock(thread.0 as usize);
        let value = napi_try_or_exit!(value);
        exit_ok(frame, &value)
    } else {
        exit_ok(frame, &ObjectSmartRef::null())
    }
}

pub unsafe extern "C" fn _mutex_with_lock(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    let func = frame.locals.get_global("func");
    let func = ObjectSmartRef::new(func);
    let func = func.deref();
    let func = napi_try_or_exit!(func);
    mutex.raw_lock(thread.0 as usize);
    let value = thread.call_obj(&func, "__call__", &[]);
    mutex.unlock(thread.0 as usize);
    let value = napi_try_or_exit!(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _mutex_try_lock(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    let value = mutex.try_raw_lock(thread.0 as usize);
    let value = alloc_bool(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _mutex_lock(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    mutex.raw_lock(thread.0 as usize);
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _mutex_unlock(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    mutex.unlock(thread.0 as usize);
    exit_ok(frame, &ObjectSmartRef::null())
}


pub unsafe extern "C" fn _mutex_set(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    let value = frame.locals.get_global("value");
    fence(Ordering::Acquire);
    // SAFETY: Гарантия стандарта.
    let old_value =
        unsafe {
            let old_value = *mutex.data_ptr();
            let old_value = ObjectSmartRef::new(old_value);
            *mutex.data_ptr() = value;
            old_value
        };
    fence(Ordering::Release);
    exit_ok(frame, &old_value)
}

pub unsafe extern "C" fn _mutex_get(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    let value = mutex.data_ptr();
    fence(Ordering::Acquire);
    // SAFETY: Гарантия стандарта.
    let value = unsafe { *value };
    let value = ObjectSmartRef::new(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _mutex_to_json(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let mutex = mutex_native_data(&this);
    let value = mutex.data_ptr();
    fence(Ordering::Acquire);
    // SAFETY: Гарантия стандарта.
    let value = unsafe { *value };
    let value = ObjectSmartRef::new(value.clone());
    let value = json_element_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    let value = alloc_json_element(thread, "std/thread/Mutex".to_owned(), value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _mutex_from_json(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = frame.locals.get_global("value");
    let value = ObjectSmartRef::new(value);
    let value = json_element_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    let value = json_element_to_object(thread, value.clone());
    let value = napi_try_or_exit!(value);
    let value = alloc_mutex(thread, &value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

fn mutex_native_data(this: &ObjectSmartRefNN) -> &'static mut Mutex<ObjectRef> {
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Mutex<ObjectRef>;
        let ptr = &mut *ptr;
        ptr
    }
}