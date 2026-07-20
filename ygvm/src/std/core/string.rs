use crate::napi::control::{exit_err, exit_ok, exit_throw};
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::napi_try_or_exit;
use crate::std::core::exception::alloc_exception;
use crate::std::core::call_to_string_or_text_null;
use crate::std::json::json_element::{alloc_json_element, json_element_to_native};
use crate::vm::heap::VMHeap;
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::VMError;
use serde_json::Value;
use std::str::FromStr;
use crate::napi::alloc::{alloc_bool, alloc_f64, alloc_i64};

pub fn alloc_string(mut thread: VMThreadRef, value: String) -> Result<ObjectSmartRefNN, VMError> {
    let class = VMModuleManager::find_class(thread.vm, "std/core/String")?;
    let object = VMHeap::alloc(thread.vm, class)?;
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = object.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut String;
        std::ptr::write(ptr, value);
    }
    let init = class.find_method("__init__")?;
    let object = object.into();
    let object = thread.call_func(&object, init, &[])?;
    let object = object.deref()?;
    Ok(object)
}

pub fn string_to_native(mut thread: VMThreadRef, value: ObjectSmartRef) -> Result<String, VMError> {
    if let Some(object) = value.try_deref() {
        if object.class.owner.path == "std/core" && object.class.name == "String" {
            // SAFETY: Гарантия стандарта.
            unsafe {
                let ptr = object.as_raw().0.as_ptr().offset(1);
                let ptr = ptr as *mut String;
                let ptr = &*ptr;
                Ok(ptr.to_owned())
            }
        } else {
            let value = thread.call_obj(&object, "__to_string__", &[])?;
            string_to_native(thread, value)
        }
    } else {
        Ok("null".to_owned())
    }
}

pub unsafe extern "C" fn _string_init(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = thread.call_class("std/core/Object", "__init__", &[this]);
    let this = napi_try_or_exit!(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    this.flags.mark_uninit();
    let this = Into::<ObjectSmartRef>::into(this);
    exit_ok(frame, &this)
}

pub unsafe extern "C" fn _string_uninit(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.0.as_ptr().offset(1);
        let ptr = ptr as *mut String;
        std::ptr::drop_in_place(ptr);
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _string_eq(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let other = frame.locals.get_global("other");
    let value =
        if this.0 == other.0 {
            true
        } else {
            let this = ObjectSmartRef::new(this);
            let this = string_to_native(thread, this);
            let this = napi_try_or_exit!(this);
            let other = ObjectSmartRef::new(other);
            let other = string_to_native(thread, other);
            let other = napi_try_or_exit!(other);
            this == other
        };
    let value = alloc_bool(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _string_add(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = string_to_native(thread, this);
    let this = napi_try_or_exit!(this);
    let other = frame.locals.get_global("other");
    let other = ObjectSmartRef::new(other);
    let other = call_to_string_or_text_null(thread, other);
    let other = napi_try_or_exit!(other);
    let value = this + other.as_str();
    let value = alloc_string(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _string_to_i64(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = string_to_native(thread, this);
    let this = napi_try_or_exit!(this);
    let value = i64::from_str(&this);
    let value = match value {
        Ok(value) => value,
        Err(err) => {
            let exception = alloc_exception(thread, err.to_string());
            let exception = napi_try_or_exit!(exception);
            return exit_throw(exception);
        }
    };
    let value = alloc_i64(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _string_to_f64(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = string_to_native(thread, this);
    let this = napi_try_or_exit!(this);
    let value = f64::from_str(&this);
    let value = match value {
        Ok(value) => value,
        Err(err) => {
            let exception = alloc_exception(thread, err.to_string());
            let exception = napi_try_or_exit!(exception);
            return exit_throw(exception);
        }
    };
    let value = alloc_f64(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _string_to_string(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let text =
        if let Some(this) = this.try_deref() {
            this
        } else {
            let text = alloc_string(thread, "null".to_string());
            let text = napi_try_or_exit!(text);
            text
        };
    let text = Into::<ObjectSmartRef>::into(text);
    exit_ok(frame, &text)
}

pub unsafe extern "C" fn _string_to_json(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let value = string_to_native(thread, this);
    let value = napi_try_or_exit!(value);
    let value = Value::String(value);
    let value = alloc_json_element(thread, "std/core/String".to_owned(), value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _string_from_json(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = frame.locals.get_global("value");
    let value = ObjectSmartRef::new(value);
    let value = json_element_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    let value =
        match value {
            Value::String(value) => value,
            _ => {
                let exception = alloc_exception(thread, "String from json parsing error".to_owned());
                let exception = napi_try_or_exit!(exception);
                return exit_throw(exception)
            }
        };
    let value = alloc_string(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}