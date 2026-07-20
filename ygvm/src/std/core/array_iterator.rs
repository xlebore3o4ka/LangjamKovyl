use crate::napi::alloc::alloc_bool;
use crate::napi::control::{exit_err, exit_ok, exit_throw};
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::napi_try_or_exit;
use crate::std::core::array::array_native_data;
use crate::std::core::exception::alloc_exception;
use crate::vm::heap::{ObjectRef, ObjectRefNN, VMHeap};
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::VMError;

pub fn alloc_array_iterator(mut thread: VMThreadRef, array: &ObjectSmartRefNN) -> Result<ObjectSmartRef, VMError> {
    let class = VMModuleManager::find_class(thread.vm, "std/core/ArrayIterator")?;
    let mut object = VMHeap::alloc(thread.vm, class)?;
    object.fields.insert("array".to_owned(), array.as_raw().into());
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = object.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut &'static mut Vec<ObjectRef>;
        let array = array_native_data(array);
        std::ptr::write(ptr, array);
        let ptr = ptr.offset(1);
        let ptr = ptr as *mut usize;
        std::ptr::write(ptr, 0);
    }
    let init = class.find_method("__init__")?;
    let object = object.into();
    let object = thread.call_func(&object, init, &[])?;
    Ok(object)
}

pub unsafe extern "C" fn _array_iterator_has_next(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let (array, offset) = array_iterator_native_data(this);
    let value = *offset < array.len();
    let value = alloc_bool(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_iterator_next(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let (array, offset) = array_iterator_native_data(this);
    let value = array.get(*offset);
    let value = value.map_or_else(ObjectSmartRef::null, ObjectSmartRef::from);
    *offset += 1;
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _array_iterator_to_json(thread: VMThreadRef, _frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let exception = alloc_exception(thread, "Array iterator not support json serialization".to_owned());
    let exception = napi_try_or_exit!(exception);
    exit_throw(exception)
}


fn array_iterator_native_data(this: ObjectRefNN) -> (&'static mut Vec<ObjectRef>, &'static mut usize) {
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.0.as_ptr().offset(1);
        let ptr = ptr as *mut &'static mut Vec<ObjectRef>;
        let array = &mut **ptr;
        let ptr = ptr.offset(1);
        let ptr = ptr as *mut usize;
        let offset = &mut *ptr;
        (array, offset)
    }
}