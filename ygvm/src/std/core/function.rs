use crate::napi::control::{exit_err, exit_ok, exit_throw};
use crate::napi::convert::array_to_native;
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::napi::thread::{NativeFunction, NativeUserdata};
use crate::napi_try_or_exit;
use crate::std::core::exception::alloc_exception;
use crate::syntax::parser::Block;
use crate::utils::alloc::Array;
use crate::vm::heap::{ObjectRef, VMHeap, VMHeapGC};
use crate::vm::module::{Function, VMModuleManager};
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::VMError;

pub fn alloc_func_ast(thread: VMThreadRef, params: Array<String>, captures: Array<(String, ObjectSmartRef)>, body: Block) -> Result<ObjectSmartRefNN, VMError> {
    let function = Function::VM {
        name: "<call>".to_owned(),
        params,
        body
    };
    alloc_fn(thread, captures, function)
}

pub fn alloc_func_native(thread: VMThreadRef, params: Array<String>, captures: Array<(String, ObjectSmartRef)>, function: NativeFunction, userdata: NativeUserdata) -> Result<ObjectSmartRefNN, VMError> {
    let function = Function::Native {
        name: "<call>".to_owned(),
        params,
        function,
        userdata
    };
    alloc_fn(thread, captures, function)
}

fn alloc_fn(mut thread: VMThreadRef, captures: Array<(String, ObjectSmartRef)>, function: Function) -> Result<ObjectSmartRefNN, VMError> {
    let class = VMModuleManager::find_class(thread.vm, "std/core/Function")?;
    let object = VMHeap::alloc(thread.vm, class)?;
    let captures = captures.into_iter().map(|(k, v)| (k.to_owned(), v.as_raw())).collect();
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = object.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Function;
        std::ptr::write(ptr, function);
        let ptr = ptr.offset(1);
        let ptr = ptr as *mut Array<(String, ObjectRef)>;
        std::ptr::write(ptr, captures);
    }
    let init = class.find_method("__init__")?;
    let object = object.into();
    let object = thread.call_func(&object, init, &[])?;
    let object = object.deref()?;
    Ok(object)
}

pub unsafe extern "C" fn _function_init(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
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

pub extern "C" fn _function_uninit(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let (func, captures) = function_native_data(&this);
    // SAFETY: Гарантия стандарта.
    unsafe {
        std::ptr::drop_in_place(func);
        std::ptr::drop_in_place(captures);
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub extern "C" fn _function_mark(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let (_, captures) = function_native_data(&this);
    for (_, value) in captures.iter() {
        if let Some(value) = value.try_deref() {
            let value = ObjectSmartRefNN::new(value);
            if let Err(err) = VMHeapGC::gc_mark(thread.into(), &value) {
                return exit_err(err);
            }
        }
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _function_call(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let (func, captures) = function_native_data(&this);
    let args = frame.locals.get_global("__args__");
    let args = ObjectSmartRef::new(args);
    let args =
        if let Some(args) = args.try_deref() {
            let args = args.into();
            let args = array_to_native(thread, args);
            let args = napi_try_or_exit!(args);
            args
        } else {
            Vec::new()
        };
    let mut new_frame = thread.frame_new(&func);
    for (name, value) in captures.iter() {
        new_frame.locals.set_global(name, *value)
    }
    napi_try_or_exit!(thread.frame_init_args(new_frame, func.params(), &args));
    let result =
        match thread.frame_exec(new_frame) {
            Ok(value) => exit_ok(frame, &value),
            Err(err) => exit_err(err)
        };
    thread.frames.pop();
    result
}

pub unsafe extern "C" fn _function_to_json(thread: VMThreadRef, _frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let exception = alloc_exception(thread, "Function not support json serialization".to_owned());
    let exception = napi_try_or_exit!(exception);
    exit_throw(exception)
}


fn function_native_data(this: &ObjectSmartRefNN) -> (&'static mut Function, &'static mut Array<(String, ObjectRef)>) {
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut Function;
        let func = &mut *ptr;
        let ptr = ptr.offset(1);
        let ptr = ptr as *mut Array<(String, ObjectRef)>;
        let captures = &mut *ptr;
        (func, captures)
    }
}