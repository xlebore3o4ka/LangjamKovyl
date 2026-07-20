use crate::napi::alloc::{alloc_bool, alloc_string};
use crate::napi::control::{exit_err, exit_ok, exit_throw};
use crate::napi::convert::{bool_to_native, i64_to_native};
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::napi_try_or_exit;
use crate::std::core::array_iterator::alloc_array_iterator;
use crate::std::core::exception::alloc_exception;
use crate::std::core::{call_eq_or_eq, call_to_string_or_text_null};
use crate::std::json::json_element::{alloc_json_element, json_element_to_native, json_element_to_object};
use crate::vm::heap::{ObjectRef, VMHeap, VMHeapGC};
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::VMError;
use serde_json::Value;
use crate::std::core::i64::alloc_i64;

pub fn alloc_array(mut thread: VMThreadRef, value: Vec<ObjectSmartRef>) -> Result<ObjectSmartRefNN, VMError> {
    let class = VMModuleManager::find_class(thread.vm, "std/core/Array")?;
    let object = VMHeap::alloc(thread.vm, class)?;
    let value = value.iter().map(|x| x.as_raw()).collect::<Vec<_>>();
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = object.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Vec<ObjectRef>;
        std::ptr::write(ptr, value);
    }
    let init = class.find_method("__init__")?;
    let object = object.into();
    let object = thread.call_func(&object, init, &[])?;
    let object = object.deref()?;
    Ok(object)
}

pub fn array_to_native(mut thread: VMThreadRef, value: ObjectSmartRef) -> Result<Vec<ObjectSmartRef>, VMError> {
    let object = value.deref()?;
    if object.class.owner.path == "std/core" && object.class.name == "Array" {
        // SAFETY: Гарантия стандарта.
        let array =
            unsafe {
                let ptr = object.as_raw().0.as_ptr().offset(1);
                let ptr = ptr as *mut Vec<ObjectRef>;
                let ptr = &*ptr;
                ptr
            };
        Ok(array.iter().map(ObjectSmartRef::from).collect())
    } else {
        let value = thread.call_obj(&object, "__to_array__", &[])?;
        array_to_native(thread, value)
    }
}

pub unsafe extern "C" fn _array_init(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = thread.call_class("std/core/Object", "__init__", &[this]);
    let this = napi_try_or_exit!(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    this.flags.mark_uninit();
    this.flags.mark_marker();
    let this = this.into();
    exit_ok(frame, &this)
}

pub unsafe extern "C" fn _array_uninit(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.0.as_ptr().offset(1);
        let ptr = ptr as *mut Vec<ObjectRef>;
        std::ptr::drop_in_place(ptr);
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _array_mark(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    for object in array {
        if let Some(object) = object.try_deref() {
            let object = ObjectSmartRefNN::new(object);
            if let Err(err) = VMHeapGC::gc_mark(thread.into(), &object) {
                return exit_err(err);
            }
        }
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _array_eq(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let other = frame.locals.get_global("other");
    let other = ObjectSmartRef::new(other);
    let value = thread.call_class("std/core/Object", "__eq__", &[this.clone(), other.clone()]);
    let value = napi_try_or_exit!(value);
    let value = bool_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    let value =
        if !value {
            false
        } else if this.is_null() {
            true
        } else {
            // SAFETY: Проверка is_null.
            let this = unsafe { this.deref().unwrap_unchecked() };
            let this = array_native_data(&this);
            // SAFETY: Проверка is_null + вызов __eq__.
            let other = unsafe { other.deref().unwrap_unchecked() };
            let other = array_native_data(&other);
            if this.len() == other.len() {
                let mut result = true;
                for i in 0..this.len() {
                    let this = this[i];
                    let this = ObjectSmartRef::new(this);
                    let other = other[i];
                    let other = ObjectSmartRef::new(other);
                    match call_eq_or_eq(thread, this, other) {
                        Ok(value) => if !value {
                            result = false;
                            break;
                        }
                        Err(err) => return exit_err(err)
                    }
                }
                result
            } else {
                false
            }
        };
    let value = alloc_bool(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_push(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let value = frame.locals.get_global("value");
    array.push(value);
    let this = Into::<ObjectSmartRef>::into(this);
    exit_ok(frame, &this)
}

pub unsafe extern "C" fn _array_pop(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let value = array.pop();
    let value = value.map_or_else(ObjectSmartRef::null, ObjectSmartRef::from);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_insert(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let index = frame.locals.get_global("index");
    let index = ObjectSmartRef::new(index);
    let index = i64_to_native(thread, index);
    let index = napi_try_or_exit!(index);
    let value = frame.locals.get_global("value");
    let index = if index < 0 { array.len() as i64 + index } else { index };
    array.insert(index as usize, value);
    let value = ObjectSmartRef::new(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_remove_element(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let value = frame.locals.get_global("value");
    let value = ObjectSmartRef::new(value);
    let array = array_native_data(&this);
    let mut error = Ok(());
    let position =
        array.iter().position(|x| {
            match call_eq_or_eq(thread, value.clone(), ObjectSmartRef::new(x.clone())) {
                Ok(value) => value,
                Err(err) => {
                    error = Err(err);
                    false
                }
            }
        });
    napi_try_or_exit!(error);
    let value =
        if let Some(position) = position {
            let element = array.remove(position);
            let element = ObjectSmartRef::new(element);
            element
        } else {
            ObjectSmartRef::null()
        };
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_remove(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let index = frame.locals.get_global("index");
    let index = ObjectSmartRef::new(index);
    let index = i64_to_native(thread, index);
    let index = napi_try_or_exit!(index);
    let index = if index < 0 { array.len() as i64 + index } else { index } as usize;
    let value =
        if index < array.len() {
            let value = array.remove(index);
            let value = ObjectSmartRef::new(value);
            value
        } else {
            ObjectSmartRef::null()
        };
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_set(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let index = frame.locals.get_global("index");
    let index = ObjectSmartRef::new(index);
    let index = i64_to_native(thread, index);
    let index = napi_try_or_exit!(index);
    let value = frame.locals.get_global("value");
    let index = if index < 0 { array.len() as i64 + index } else { index };
    if let Some(access) = array.get_mut(index as usize) {
        *access = value;
    } else {
        array.push(value);
    }
    let value = ObjectSmartRef::new(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_get_sliced(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let from = frame.locals.get_global("from");
    let from = ObjectSmartRef::new(from);
    let from = i64_to_native(thread, from);
    let from = napi_try_or_exit!(from);
    let mut from = from as isize;
    let to = frame.locals.get_global("to");
    let to = ObjectSmartRef::new(to);
    let to = i64_to_native(thread, to);
    let to = napi_try_or_exit!(to);
    let mut to = to as isize;
    let array = array_native_data(&this);
    if from < 0 { from = 0 } else if from > array.len() as isize { from = array.len() as isize }
    if to < 0 { to = 0 } else if to > array.len() as isize { to = array.len() as isize }
    let value = array[from as usize..to as usize].iter().map(ObjectSmartRef::from).collect::<Vec<_>>();
    let value = alloc_array(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_get(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let index = frame.locals.get_global("index");
    let index = ObjectSmartRef::new(index);
    let index = i64_to_native(thread, index);
    let index = napi_try_or_exit!(index);
    let index = if index < 0 { array.len() as i64 + index } else { index };
    let value = array.get(index as usize);
    let value = value.map_or_else(ObjectSmartRef::null, ObjectSmartRef::from);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_len(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let array = array_native_data(&this);
    let value = array.len();
    let value = value as i64;
    let value = alloc_i64(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_iter(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let value = alloc_array_iterator(thread, &this);
    let value = napi_try_or_exit!(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_to_string(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let value = array_native_data(&this);
    let mut stringified = Vec::new();
    for element in value {
        let element = ObjectSmartRef::new(element.clone());
        let value = call_to_string_or_text_null(thread, element);
        let value = napi_try_or_exit!(value);
        stringified.push(value);
    }
    let text = "[".to_owned() + stringified.join(", ").as_str() + "]";
    let text = alloc_string(thread, text);
    let text = napi_try_or_exit!(text);
    let text = Into::<ObjectSmartRef>::into(text);
    exit_ok(frame, &text)
}

pub unsafe extern "C" fn _array_to_json(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRefNN::deref(this);
    let this = napi_try_or_exit!(this);
    let value = array_native_data(&this);
    let mut array = Vec::new();
    for element in value {
        let element = ObjectSmartRef::new(element.clone());
        let element = json_element_to_native(thread, element);
        let element = napi_try_or_exit!(element);
        array.push(element);
    }
    let value = Value::Array(array);
    let value = alloc_json_element(thread, "std/core/Array".to_owned(), value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_from_json(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = frame.locals.get_global("value");
    let value = ObjectSmartRef::new(value);
    let value = json_element_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    let value =
        match value {
            Value::Array(value) => value,
            _ => {
                let exception = alloc_exception(thread, "Float from json parsing error".to_owned());
                let exception = napi_try_or_exit!(exception);
                return exit_throw(exception)
            }
        };
    let mut array = Vec::new();
    for element in value {
        let element = json_element_to_object(thread, element);
        let element = napi_try_or_exit!(element);
        array.push(element);
    }
    let value = alloc_array(thread, array);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub fn array_native_data(this: &ObjectSmartRefNN) -> &'static mut Vec<ObjectRef> {
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Vec<ObjectRef>;
        let ptr = &mut *ptr;
        ptr
    }
}