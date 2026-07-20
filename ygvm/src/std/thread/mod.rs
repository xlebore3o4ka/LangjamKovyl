pub mod mutex;
pub mod thread;

use crate::napi::control::{exit_err, exit_ok};
use crate::napi::convert::i64_to_native;
use crate::napi::module::{ClassDef, FunctionBodyDef, FunctionDef, ModuleDef};
use crate::napi::ptr::ObjectSmartRef;
use crate::std::thread::mutex::{_mutex_from_json, _mutex_get, _mutex_init, _mutex_lock, _mutex_mark, _mutex_set, _mutex_to_json, _mutex_try_lock, _mutex_try_with_lock, _mutex_uninit, _mutex_unlock, _mutex_with_lock, alloc_mutex};
use crate::utils::mutex::Mutex;
use crate::std::thread::thread::{_thread_create_wrapper, _thread_get_catcher, _thread_join, _thread_set_catcher, _thread_start, _thread_to_json, alloc_thread, thread_native_data};
use crate::vm::heap::ObjectRef;
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadManager, VMThreadRef, VMThreadState};
use crate::vm::{VMError, VMRef};
use crate::napi_try_or_exit;

pub fn load(vm: VMRef) -> Result<(), VMError> {
    VMModuleManager::load_napi_module(vm, &ModuleDef {
        path: "std/thread".to_owned(),
        uses: vec![],
        functions: vec![
            FunctionDef {
                name: "sleep".to_owned(),
                params: vec!["time".to_owned()],
                body: FunctionBodyDef::Native(_sleep),
            },
            FunctionDef {
                name: "current".to_owned(),
                params: vec![],
                body: FunctionBodyDef::Native(_current),
            },
            FunctionDef {
                name: "create".to_owned(),
                params: vec!["func".to_owned()],
                body: FunctionBodyDef::Native(_create),
            },
            FunctionDef {
                name: "mutex".to_owned(),
                params: vec!["value".to_owned()],
                body: FunctionBodyDef::Native(_mutex),
            }
        ],
        classes: vec![
            ClassDef {
                name: "Thread".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "start".to_owned(),
                        params: vec!["catcher".to_owned()],
                        body: FunctionBodyDef::Native(_thread_start),
                    },
                    FunctionDef {
                        name: "join".to_owned(),
                        params: vec!["catcher".to_owned()],
                        body: FunctionBodyDef::Native(_thread_join),
                    },
                    FunctionDef {
                        name: "set_catcher".to_owned(),
                        params: vec!["catcher".to_owned()],
                        body: FunctionBodyDef::Native(_thread_set_catcher),
                    },
                    FunctionDef {
                        name: "get_catcher".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_thread_get_catcher),
                    },
                    FunctionDef {
                        name: "__thread_create_wrapper__".to_owned(),
                        params: vec!["catcher".to_owned()],
                        body: FunctionBodyDef::Native(_thread_create_wrapper),
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_thread_to_json)
                    },
                ],
                allocation: size_of::<VMThreadRef>()
            },
            ClassDef {
                name: "Mutex".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_mutex_init),
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_mutex_uninit),
                    },
                    FunctionDef {
                        name: "__mark__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_mutex_mark),
                    },
                    FunctionDef {
                        name: "try_with_lock".to_owned(),
                        params: vec!["func".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_try_with_lock),
                    },
                    FunctionDef {
                        name: "with_lock".to_owned(),
                        params: vec!["func".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_with_lock),
                    },
                    FunctionDef {
                        name: "try_lock".to_owned(),
                        params: vec!["func".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_try_lock),
                    },
                    FunctionDef {
                        name: "lock".to_owned(),
                        params: vec!["func".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_lock),
                    },
                    FunctionDef {
                        name: "unlock".to_owned(),
                        params: vec!["func".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_unlock),
                    },
                    FunctionDef {
                        name: "unlock".to_owned(),
                        params: vec!["func".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_unlock),
                    },
                    FunctionDef {
                        name: "set".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_set),
                    },
                    FunctionDef {
                        name: "get".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_mutex_get),
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_mutex_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_mutex_from_json)
                    },
                ],
                allocation: size_of::<Mutex<ObjectRef>>()
            },
        ],
        objects: vec![],
    })
}

pub fn unload(_vm: VMRef) -> Result<(), VMError> {
    Ok(())
}

unsafe extern "C" fn _sleep(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = frame.locals.get_global("time");
    let value = ObjectSmartRef::new(value);
    let value = i64_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    let saved_state = thread.flags.state;
    VMThreadState::change(thread, VMThreadState::RunningNativeClean);
    std::thread::sleep(std::time::Duration::from_millis(value as u64));
    VMThreadState::change(thread, saved_state);
    exit_ok(frame, &ObjectSmartRef::null())
}

unsafe extern "C" fn _current(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = alloc_thread(thread, thread);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

unsafe extern "C" fn _create(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = VMThreadManager::new_thread(thread.vm);
    let value = alloc_thread(thread, value);
    let mut value = napi_try_or_exit!(value);
    let mut thr = thread_native_data(&value);
    let func = frame.locals.get_global("func");
    let func = func.deref();
    let func = napi_try_or_exit!(func);
    value.fields.insert("__thread_create_func__".to_owned(), func.into());
    let wrapper_func = value.class.find_method("__thread_create_wrapper__");
    let wrapper_func = napi_try_or_exit!(wrapper_func);
    let mut new_frame = thr.frame_new(wrapper_func);
    new_frame.locals.set_global("this", value.as_raw().into());
    thr.owner.drop_owning();
    let value = value.into();
    exit_ok(frame, &value)
}

unsafe extern "C" fn _mutex(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let value = frame.locals.get_global("value");
    let value = ObjectSmartRef::new(value);
    let value = alloc_mutex(thread, &value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}