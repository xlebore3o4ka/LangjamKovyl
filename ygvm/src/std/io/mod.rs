pub mod server;
pub mod connection;
pub mod client;

use crate::napi::alloc::{alloc_bool, alloc_string};
use crate::napi::control::{exit_err, exit_ok};
use crate::napi::convert::{array_to_native, string_to_native};
use crate::napi::module::{ClassDef, FunctionBodyDef, FunctionDef, ModuleDef};
use crate::napi::ptr::ObjectSmartRef;
use crate::napi_try_or_exit;
use crate::std::core::exception::alloc_exception;
use crate::std::io::client::{_client_close, _client_init, _client_recv, _client_send, _client_uninit, alloc_client};
use crate::std::io::connection::{_connection_addr, _connection_close, _connection_init, _connection_recv, _connection_send, _connection_uninit};
use crate::std::io::server::{_server_accept, _server_init, _server_uninit, alloc_server};
use crate::utils::socket::client::Client;
use crate::utils::socket::server::{Connection, Server};
use crate::vm::module::VMModuleManager;
use crate::vm::thread::{VMStackFrameRef, VMThreadRef, VMThreadState};
use crate::vm::{VMError, VMRef};
use std::fs;
use std::io::Write;

pub fn load(vm: VMRef) -> Result<(), VMError> {
    VMModuleManager::load_napi_module(vm, &ModuleDef {
        path: "std/io".to_owned(),
        uses: vec![],
        functions: vec![
            FunctionDef {
                name: "readline".to_owned(),
                params: vec![],
                body: FunctionBodyDef::Native(_readline),
            },
            FunctionDef {
                name: "print".to_owned(),
                params: vec![],
                body: FunctionBodyDef::Native(_print),
            },
            FunctionDef {
                name: "println".to_owned(),
                params: vec![],
                body: FunctionBodyDef::Native(_println),
            },
            FunctionDef {
                name: "file_exits".to_owned(),
                params: vec!["file".to_owned()],
                body: FunctionBodyDef::Native(_file_exits),
            },
            FunctionDef {
                name: "file_read".to_owned(),
                params: vec!["file".to_owned()],
                body: FunctionBodyDef::Native(_file_read),
            },
            FunctionDef {
                name: "file_write".to_owned(),
                params: vec!["file".to_owned(), "value".to_owned()],
                body: FunctionBodyDef::Native(_file_write),
            },
            FunctionDef {
                name: "open_server".to_owned(),
                params: vec!["addr".to_owned()],
                body: FunctionBodyDef::Native(_open_server)
            },
            FunctionDef {
                name: "open_client".to_owned(),
                params: vec!["addr".to_owned()],
                body: FunctionBodyDef::Native(_open_client)
            },
        ],
        classes: vec![
            ClassDef {
                name: "ServerSocket".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_server_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_server_uninit)
                    },
                    FunctionDef {
                        name: "accept".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_server_accept)
                    },
                ],
                allocation: size_of::<Server>()
            },
            ClassDef {
                name: "Connection".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_connection_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_connection_uninit)
                    },
                    FunctionDef {
                        name: "addr".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_connection_addr)
                    },
                    FunctionDef {
                        name: "send".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_connection_send)
                    },
                    FunctionDef {
                        name: "recv".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_connection_recv)
                    },
                    FunctionDef {
                        name: "close".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_connection_close)
                    },
                ],
                allocation: size_of::<(Connection, (u8, u8, u8, u8), String)>()
            },
            ClassDef {
                name: "ClientSocket".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_client_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_client_uninit)
                    },
                    FunctionDef {
                        name: "send".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_client_send)
                    },
                    FunctionDef {
                        name: "recv".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_client_recv)
                    },
                    FunctionDef {
                        name: "close".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_client_close)
                    },
                ],
                allocation: size_of::<Client>()
            },
        ],
        objects: vec![],
    })
}

pub fn unload(_vm: VMRef) -> Result<(), VMError> {
    Ok(())
}

unsafe extern "C" fn _readline(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let saved_state = thread.flags.state;
    VMThreadState::change(thread, VMThreadState::RunningNativeClean);
    let mut text = String::new();
    let result = std::io::stdin().read_line(&mut text);
    napi_try_or_exit!(map_std_io_err_to_vm_throw(thread, result));
    VMThreadState::change(thread, saved_state);
    if text.ends_with('\n') { text.pop(); }
    let text = alloc_string(thread, text);
    let text = napi_try_or_exit!(text);
    let text = text.into();
    exit_ok(frame, &text)
}

unsafe extern "C" fn _print(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let args = frame.locals.get_global("__args__");
    let args = ObjectSmartRef::new(args);
    let args = array_to_native(thread, args);
    let args = napi_try_or_exit!(args);
    let mut text = String::new();
    for arg in args {
        let value = string_to_native(thread, arg);
        let value = napi_try_or_exit!(value);
        text.push_str(&value);
    }
    print!("{}", text);
    std::io::stdout().flush().unwrap();
    exit_ok(frame, &ObjectSmartRef::null())
}

unsafe extern "C" fn _println(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let args = frame.locals.get_global("__args__");
    let args = ObjectSmartRef::new(args);
    let args = array_to_native(thread, args);
    let args = napi_try_or_exit!(args);
    let mut text = String::new();
    for arg in args {
        let value = string_to_native(thread, arg);
        let value = napi_try_or_exit!(value);
        text.push_str(&value);
    }
    println!("{}", text);
    std::io::stdout().flush().unwrap();
    exit_ok(frame, &ObjectSmartRef::null())
}

unsafe extern "C" fn _file_exits(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let file = frame.locals.get_global("file");
    let file = ObjectSmartRef::new(file);
    let file = string_to_native(thread, file);
    let file = napi_try_or_exit!(file);
    let value = fs::exists(file);
    let value = map_std_io_err_to_vm_throw(thread, value);
    let value = napi_try_or_exit!(value);
    let value = alloc_bool(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}

unsafe extern "C" fn _file_read(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let file = frame.locals.get_global("file");
    let file = ObjectSmartRef::new(file);
    let file = string_to_native(thread, file);
    let file = napi_try_or_exit!(file);
    let value = fs::read_to_string(file);
    let value = map_std_io_err_to_vm_throw(thread, value);
    let value = napi_try_or_exit!(value);
    let value = alloc_string(thread, value);
    let value = napi_try_or_exit!(value);
    let value = value.into();
    exit_ok(frame, &value)
}


unsafe extern "C" fn _file_write(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let file = frame.locals.get_global("file");
    let file = ObjectSmartRef::new(file);
    let file = string_to_native(thread, file);
    let file = napi_try_or_exit!(file);
    let value = frame.locals.get_global("value");
    let value = ObjectSmartRef::new(value);
    let value = string_to_native(thread, value);
    let value = napi_try_or_exit!(value);
    napi_try_or_exit!(map_std_io_err_to_vm_throw(thread, fs::write(file, value)));
    exit_ok(frame, &ObjectSmartRef::null())
}

unsafe extern "C" fn _open_server(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let addr = frame.locals.get_global("addr");
    let addr = ObjectSmartRef::new(addr);
    let addr = string_to_native(thread, addr);
    let addr = napi_try_or_exit!(addr);
    let server = alloc_server(thread, addr);
    let server = napi_try_or_exit!(server);
    let server = server.into();
    exit_ok(frame, &server)
}

unsafe extern "C" fn _open_client(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    let addr = frame.locals.get_global("addr");
    let addr = ObjectSmartRef::new(addr);
    let addr = string_to_native(thread, addr);
    let addr = napi_try_or_exit!(addr);
    let client = alloc_client(thread, addr);
    let client = napi_try_or_exit!(client);
    let client = client.into();
    exit_ok(frame, &client)
}

pub fn map_std_io_err_to_vm_throw<T>(thread: VMThreadRef, result: Result<T, std::io::Error>) -> Result<T, VMError> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(VMError::__Throwing__(alloc_exception(thread, err.to_string())?))
    }
}