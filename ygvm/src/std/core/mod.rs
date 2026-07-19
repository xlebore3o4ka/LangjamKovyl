pub mod array;
pub mod array_iterator;
pub mod bool;
pub mod callable;
pub mod exception;
pub mod function;
pub mod f64;
pub mod i64;
pub mod iterator;
pub mod map;
pub mod object;
pub mod string;
pub mod throwable;

use crate::napi::control::exit_err;
use crate::napi::control::exit_ok;
use crate::napi::convert::{bool_to_native, i64_to_native, string_to_native};
use crate::napi::module::{ClassDef, FunctionBodyDef, FunctionDef, ModuleDef};
use crate::napi::ptr::ObjectSmartRef;
use crate::napi_try_or_exit;
use crate::std::core::array::{_array_eq, _array_from_json, _array_get, _array_get_sliced, _array_init, _array_insert, _array_iter, _array_len, _array_mark, _array_pop, _array_push, _array_remove, _array_remove_element, _array_set, _array_to_json, _array_to_string, _array_uninit};
use crate::std::core::array_iterator::{_array_iterator_has_next, _array_iterator_next};
use crate::std::core::bool::{_bool_and, _bool_eq, _bool_from_json, _bool_not, _bool_or, _bool_to_bool, _bool_to_json, _bool_to_string};
use crate::std::core::callable::{_callable_call, _callable_eq};
use crate::std::core::exception::_exception_to_string;
use crate::std::core::f64::{_f64_add, _f64_div, _f64_eq, _f64_from_json, _f64_ge, _f64_gt, _f64_le, _f64_lt, _f64_mul, _f64_neg, _f64_sub, _f64_to_f64, _f64_to_i64, _f64_to_json, _f64_to_string};
use crate::std::core::function::{_function_call, _function_init, _function_mark, _function_to_json, _function_uninit};
use crate::std::core::i64::{_i64_add, _i64_div, _i64_eq, _i64_from_json, _i64_ge, _i64_gt, _i64_le, _i64_lt, _i64_mul, _i64_neg, _i64_sub, _i64_to_f64, _i64_to_i64, _i64_to_json, _i64_to_string};
use crate::std::core::iterator::{_iterator_has_next, _iterator_next};
use crate::std::core::map::{_map_eq, _map_from_json, _map_get, _map_init, _map_mark, _map_set, _map_to_json, _map_to_string, _map_uninit};
use crate::std::core::object::{_object_eq, _object_from_json, _object_hash, _object_init, _object_neq, _object_to_json, _object_to_string};
use crate::std::core::string::{_string_add, _string_eq, _string_from_json, _string_init, _string_to_f64, _string_to_i64, _string_to_json, _string_to_string, _string_uninit};
use crate::std::core::throwable::_throwable_eq;
use crate::utils::alloc::Array;
use crate::utils::map::Map;
use crate::vm::heap::{ObjectRef, VMHeapGC};
use crate::vm::module::{Function, VMModuleManager};
use crate::vm::thread::{VMStackFrameRef, VMThreadRef};
use crate::vm::{VMError, VMRef};

pub fn load(vm: VMRef) -> Result<(), VMError> {
    VMModuleManager::load_napi_module(vm, &ModuleDef {
        path: "std/core".to_owned(),
        uses: vec![],
        functions: vec![
            FunctionDef {
                name: "gc".to_owned(),
                params: vec![],
                body: FunctionBodyDef::Native(_gc)
            }
        ],
        classes: vec![
            ClassDef {
                name: "Object".to_string(),
                extends: vec![],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_object_init)
                    },
                    FunctionDef {
                        name: "__hash__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_object_hash)
                    },
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_object_eq)
                    },
                    FunctionDef {
                        name: "__neq__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_object_neq)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_object_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_object_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_object_from_json)
                    },
                ],
                allocation: 0
            },
            ClassDef {
                name: "Bool".to_string(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__to_bool__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_bool_to_bool)
                    },
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_bool_eq)
                    },
                    FunctionDef {
                        name: "__and__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_bool_and)
                    },
                    FunctionDef {
                        name: "__or__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_bool_or)
                    },
                    FunctionDef {
                        name: "__not__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_bool_not)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_bool_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_bool_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_bool_from_json)
                    },
                ],
                allocation: 0
            },
            ClassDef {
                name: "I64".to_string(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_i64_eq)
                    },
                    FunctionDef {
                        name: "__add__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_add)
                    },
                    FunctionDef {
                        name: "__sub__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_sub)
                    },
                    FunctionDef {
                        name: "__mul__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_mul)
                    },
                    FunctionDef {
                        name: "__div__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_div)
                    },
                    FunctionDef {
                        name: "__neg__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_neg)
                    },
                    FunctionDef {
                        name: "__lt__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_lt)
                    },
                    FunctionDef {
                        name: "__le__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_le)
                    },
                    FunctionDef {
                        name: "__gt__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_gt)
                    },
                    FunctionDef {
                        name: "__ge__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_i64_ge)
                    },
                    FunctionDef {
                        name: "__to_i64__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_i64_to_i64)
                    },
                    FunctionDef {
                        name: "__to_f64__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_i64_to_f64)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_i64_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_i64_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_i64_from_json)
                    },
                ],
                allocation: size_of::<i64>()
            },
            ClassDef {
                name: "F64".to_string(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_eq)
                    },
                    FunctionDef {
                        name: "__add__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_add)
                    },
                    FunctionDef {
                        name: "__sub__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_sub)
                    },
                    FunctionDef {
                        name: "__mul__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_mul)
                    },
                    FunctionDef {
                        name: "__div__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_div)
                    },
                    FunctionDef {
                        name: "__neg__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_neg)
                    },
                    FunctionDef {
                        name: "__lt__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_lt)
                    },
                    FunctionDef {
                        name: "__le__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_le)
                    },
                    FunctionDef {
                        name: "__gt__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_gt)
                    },
                    FunctionDef {
                        name: "__ge__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_f64_ge)
                    },
                    FunctionDef {
                        name: "__to_i64__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_f64_to_i64)
                    },
                    FunctionDef {
                        name: "__to_f64__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_f64_to_f64)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_f64_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_f64_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_f64_from_json)
                    },
                ],
                allocation: size_of::<f64>()
            },
            ClassDef {
                name: "String".to_string(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_string_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_string_uninit)
                    },
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_string_eq)
                    },
                    FunctionDef {
                        name: "__add__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_string_add)
                    },
                    FunctionDef {
                        name: "__to_i64__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_string_to_i64)
                    },
                    FunctionDef {
                        name: "__to_f64__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_string_to_f64)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_string_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_string_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_string_from_json)
                    },
                ],
                allocation: size_of::<String>()
            },
            ClassDef {
                name: "Array".to_string(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_uninit)
                    },
                    FunctionDef {
                        name: "__mark__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_mark)
                    },
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_array_eq)
                    },
                    FunctionDef {
                        name: "len".to_owned(),
                        params: vec!["index".to_owned()],
                        body: FunctionBodyDef::Native(_array_len)
                    },
                    FunctionDef {
                        name: "get_sliced".to_owned(),
                        params: vec!["from".to_owned(), "to".to_owned()],
                        body: FunctionBodyDef::Native(_array_get_sliced)
                    },
                    FunctionDef {
                        name: "remove_element".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_array_remove_element)
                    },
                    FunctionDef {
                        name: "push".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_array_push)
                    },
                    FunctionDef {
                        name: "pop".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_pop)
                    },
                    FunctionDef {
                        name: "insert".to_owned(),
                        params: vec!["index".to_owned(), "value".to_owned()],
                        body: FunctionBodyDef::Native(_array_insert)
                    },
                    FunctionDef {
                        name: "remove".to_owned(),
                        params: vec!["index".to_owned()],
                        body: FunctionBodyDef::Native(_array_remove)
                    },
                    FunctionDef {
                        name: "__set__".to_owned(),
                        params: vec!["index".to_owned(), "value".to_owned()],
                        body: FunctionBodyDef::Native(_array_set)
                    },
                    FunctionDef {
                        name: "__get__".to_owned(),
                        params: vec!["index".to_owned()],
                        body: FunctionBodyDef::Native(_array_get)
                    },
                    FunctionDef {
                        name: "__iter__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_iter)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec!["index".to_owned()],
                        body: FunctionBodyDef::Native(_array_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_array_from_json)
                    },
                ],
                allocation: size_of::<Vec<ObjectRef>>()
            },
            ClassDef {
                name: "Map".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_map_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_map_uninit)
                    },
                    FunctionDef {
                        name: "__mark__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_map_mark)
                    },
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec!["other".to_owned()],
                        body: FunctionBodyDef::Native(_map_eq)
                    },
                    FunctionDef {
                        name: "__set__".to_owned(),
                        params: vec!["index".to_owned(), "value".to_owned()],
                        body: FunctionBodyDef::Native(_map_set)
                    },
                    FunctionDef {
                        name: "__get__".to_owned(),
                        params: vec!["index".to_owned()],
                        body: FunctionBodyDef::Native(_map_get)
                    },
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec!["index".to_owned()],
                        body: FunctionBodyDef::Native(_map_to_string)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_map_to_json)
                    },
                    FunctionDef {
                        name: "__from_json__".to_owned(),
                        params: vec!["value".to_owned()],
                        body: FunctionBodyDef::Native(_map_from_json)
                    },
                ],
                allocation: size_of::<Map>()
            },
            ClassDef {
                name: "Iterator".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "has_next".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_iterator_has_next)
                    },
                    FunctionDef {
                        name: "next".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_iterator_next)
                    },
                ],
                allocation: size_of::<(&'static mut Vec<ObjectRef>, usize)>()
            },
            ClassDef {
                name: "ArrayIterator".to_owned(),
                extends: vec!["std/core/Iterator".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "has_next".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_iterator_has_next)
                    },
                    FunctionDef {
                        name: "next".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_array_iterator_next)
                    },
                ],
                allocation: size_of::<(&'static mut Vec<ObjectRef>, usize)>()
            },
            ClassDef {
                name: "Callable".to_owned(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_callable_eq)
                    },
                    FunctionDef {
                        name: "__call__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_callable_call)
                    },
                ],
                allocation: 0
            },
            ClassDef {
                name: "Function".to_owned(),
                extends: vec!["std/core/Callable".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__init__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_function_init)
                    },
                    FunctionDef {
                        name: "__uninit__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_function_uninit)
                    },
                    FunctionDef {
                        name: "__mark__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_function_mark)
                    },
                    FunctionDef {
                        name: "__call__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_function_call)
                    },
                    FunctionDef {
                        name: "__to_json__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_function_to_json)
                    },
                ],
                allocation: size_of::<(Function, Array<(String, ObjectRef)>)>()
            },
            ClassDef {
                name: "Throwable".to_string(),
                extends: vec!["std/core/Object".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__eq__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_throwable_eq)
                    },
                ],
                allocation: 0
            },
            ClassDef {
                name: "Exception".to_string(),
                extends: vec!["std/core/Throwable".to_owned()],
                methods: vec![
                    FunctionDef {
                        name: "__to_string__".to_owned(),
                        params: vec![],
                        body: FunctionBodyDef::Native(_exception_to_string)
                    }
                ],
                allocation: 0
            },
        ],
        objects: vec![
            ClassDef {
                name: "True".to_owned(),
                extends: vec!["std/core/Bool".to_owned()],
                methods: vec![],
                allocation: 0
            },
            ClassDef {
                name: "False".to_owned(),
                extends: vec!["std/core/Bool".to_owned()],
                methods: vec![],
                allocation: 0
            },
        ],
    })
}

pub fn unload(_vm: VMRef) -> Result<(), VMError> {
    Ok(())
}

unsafe extern "C" fn _gc(thread: VMThreadRef, frame: VMStackFrameRef) -> *mut Result<(), VMError> {
    napi_try_or_exit!(VMHeapGC::gc(thread.vm, Some(thread)));
    exit_ok(frame, &ObjectSmartRef::null())
}

pub fn call_to_string_or_text_null(mut thread: VMThreadRef, object: ObjectSmartRef) -> Result<String, VMError> {
    if let Some(object) = object.try_deref() {
        let value = thread.call_obj(&object, "__to_string__", &[])?;
        let value = string_to_native(thread, value)?;
        Ok(value)
    } else {
        Ok("null".to_string())
    }
}

pub fn call_hash_or_nil(mut thread: VMThreadRef, object: ObjectSmartRef) -> Result<i64, VMError> {
    if let Some(field) = object.try_deref() {
        let value = thread.call_obj(&field, "__hash__", &[])?;
        let value = i64_to_native(thread, value)?;
        Ok(value)
    } else {
        Ok(0)
    }
}

pub fn call_eq_or_eq(mut thread: VMThreadRef, value: ObjectSmartRef, other: ObjectSmartRef) -> Result<bool, VMError> {
    if let Some(object) = value.try_deref() {
        let value = thread.call_obj(&object, "__eq__", &[other])?;
        let value = bool_to_native(thread, value)?;
        Ok(value)
    } else {
        Ok(other.is_null())
    }
}