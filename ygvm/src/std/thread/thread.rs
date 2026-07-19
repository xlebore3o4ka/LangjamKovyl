use crate::napi::control::{exit_err, exit_ok, exit_throw};
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::napi_try_or_exit;
use crate::std::core::exception::alloc_exception;
use crate::vm::heap::VMHeap;
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::VMError;
use std::sync::atomic::{fence, Ordering};
use std::time::Duration;

pub fn alloc_thread(mut thread: VMThreadRef, value: VMThreadRef) -> Result<ObjectSmartRefNN, VMError> {
    let class = VMModuleManager::find_class(thread.vm, "std/thread/Thread")?;
    let object = VMHeap::alloc(thread.vm, class)?;
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = object.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut VMThreadRef;
        std::ptr::write(ptr, value);
    }
    let init = class.find_method("__init__")?;
    let object = object.into();
    let object = thread.call_func(&object, init, &[])?;
    let object = object.deref()?;
    Ok(object)
}

pub unsafe extern "C" fn _thread_start(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let thr = thread_native_data(&this);
    if thr.owner.try_catch_owning() {
        if thr.flags.state.is_live() && !thr.flags.state.is_work() {
            hard_fence();
            let _lock = thread.vm.threads.lock.lock();
            thread.vm.threads.pool.execute(move || {
                hard_fence();
                thr.owner.change_owning();
                thread.vm.call_exec(thr).unwrap();
            });
        }
    }
    exit_ok(frame, &ObjectSmartRef::null())
}

pub unsafe extern "C" fn _thread_join(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let thr = thread_native_data(&this);
    if thr.owner.try_catch_owning() { return exit_ok(frame, &ObjectSmartRef::null()) }
    let mut result = Ok(ObjectSmartRef::null());
    if thr.flags.state.is_live() {
        hard_fence();
        result = thread.vm.call_exec(thr);
        hard_fence();
    }
    let result = napi_try_or_exit!(result);
    exit_ok(frame, &result)
}

pub unsafe extern "C" fn _thread_set_catcher(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let new_value = frame.locals.get_global("catcher");
    let mut thr = thread_native_data(&this);
    let old_value = thr.cather;
    let old_value = ObjectSmartRef::new(old_value);
    thr.cather = new_value;
    exit_ok(frame, &old_value)
}

pub unsafe extern "C" fn _thread_get_catcher(_thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = ObjectSmartRef::new(this);
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let thr = thread_native_data(&this);
    let value = thr.cather;
    let value = ObjectSmartRef::new(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _thread_create_wrapper(mut thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let this = frame.locals.get_global("this");
    let this = this.deref();
    let this = napi_try_or_exit!(this);
    let func = this.fields.get("__thread_create_func__");
    let func = func.map_or_else(ObjectSmartRef::null, ObjectSmartRef::from);
    let func = func.deref();
    let func = napi_try_or_exit!(func);
    let value = thread.call_obj(&func, "__call__", &[]);
    let value = napi_try_or_exit!(value);
    exit_ok(frame, &value)
}

pub unsafe extern "C" fn _thread_to_json(thread: VMThreadRef, _frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let exception = alloc_exception(thread, "Thread not support json serialization".to_owned());
    let exception = napi_try_or_exit!(exception);
    exit_throw(exception)
}


pub fn thread_native_data(this: &ObjectSmartRefNN) -> VMThreadRef {
    // SAFETY: Гарантия стандарта.
    unsafe {
        let ptr = this.as_raw().0.as_ptr().offset(1);
        let ptr = ptr as *mut VMThreadRef;
        let ptr = *ptr;
        ptr
    }
}

pub fn hard_fence() {
    fence(Ordering::SeqCst);
    std::thread::sleep(Duration::from_nanos(1));
    fence(Ordering::SeqCst);
}