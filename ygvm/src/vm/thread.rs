use crate::napi::alloc::{alloc_array, alloc_bool, alloc_f64, alloc_func_ast, alloc_i64, alloc_map, alloc_string};
use crate::napi::convert::{bool_to_native, string_to_native};
use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::std::thread::thread::hard_fence;
use crate::syntax::parser::{AssignOp, AssignTarget, BinaryOp, Block, Expr, Literal, Statement, UnaryOp};
use crate::utils::alloc::{Array, Boxed};
use crate::vm::heap::{ObjectRef, VMHeap};
use crate::vm::module::VMModuleManager;
use crate::vm::{Function, VMError, VMRef, VMState};
use crate::{napi, ownership_hack_mut};
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use threadpool::ThreadPool;

pub struct VMThreadManager {
    pub lock: ReentrantMutex<()>,
    pub pool: ThreadPool,
    pub threads: Vec<Boxed<VMThread>>
}

pub struct VMThread {
    pub vm: VMRef,
    pub owner: VMThreadOwner,
    pub flags: VMThreadFlags,
    pub frames: Vec<Boxed<VMStackFrame>>,
    pub cather: ObjectRef
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct VMThreadRef(pub *mut VMThread);

pub struct VMThreadOwner(AtomicUsize);

pub struct VMThreadFlags {
    pub gc: bool,
    pub state: VMThreadState,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VMThreadState {
    /// Готов к запуску.
    Ready,
    /// Запущен на VM.
    RunningVM,
    /// Запущен вне VM, взаимодействует с VM.
    RunningNativeLock,
    /// Запущен вне VM, не взаимодействует с VM.
    RunningNativeClean,
    /// Приостановлен.
    Suspended,
    /// Приостановлен для GC.
    SuspendedGC,
    /// Простаивает на VM.
    Yielding,
    /// Простаивает вне VM.
    YieldingNative,
    /// Завершен успешно.
    Complete,
    /// Завершен с ошибкой.
    Died,
    /// Убит.
    Killed,
}

pub struct VMStackFrame {
    pub function: *const Function,
    pub locals: VMStackFrameLocals
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct VMStackFrameRef(pub *mut VMStackFrame);

pub struct VMStackFrameLocals {
    pub blocks: Vec<HashMap<String, ObjectRef>>
}

impl VMThreadManager {
    pub fn new() -> Self {
        // SAFETY: 1 всегда больше 0.
        let available_processors = std::thread::available_parallelism().unwrap_or_else(|_| unsafe { NonZeroUsize::new_unchecked(1) }).get();
        Self {
            lock: ReentrantMutex::new(()),
            pool: ThreadPool::new(available_processors),
            threads: Vec::new(),
        }
    }

    pub fn new_thread(mut vm: VMRef) -> VMThreadRef {
        let tid = thread_id::get();
        let thread = Boxed::new(
            VMThread {
                vm,
                owner: VMThreadOwner(AtomicUsize::new(tid)),
                flags: VMThreadFlags {
                    gc: vm.flags.get_state() == VMState::GC,
                    state: VMThreadState::Ready
                },
                frames: vec![],
                cather: ObjectRef::null(),
            }
        );
        let ref_thread = thread.as_raw();
        hard_fence();
        let lock = vm.this().threads.lock.lock();
        vm.threads.threads.push(thread);
        drop(lock);
        hard_fence();
        VMThreadRef(ref_thread)
    }

    pub fn check_threads_is_work(vm: VMRef) -> bool {
        let _lock = vm.threads.lock.lock();
        vm.threads.threads.iter().any(|x| x.flags.state.is_work())
    }

    pub fn check_threads_is_allow_gc(vm: VMRef) -> bool {
        let _lock = vm.threads.lock.lock();
        vm.threads.threads.iter().all(|x| x.flags.state.is_gc_allow())
    }
}

impl VMThread {
    pub fn obj_set(&mut self, instance: &mut ObjectSmartRefNN, name: &str, value: ObjectSmartRef) -> Result<(), VMError> {
        VMThreadState::change(self.into(), VMThreadState::RunningVM);
        if instance.flags.check_proxy() {
            let name = alloc_string(self.into(), name.to_owned())?;
            self.call_obj(instance, "__set__", &[name.into(), value])?;
        } else {
            instance.fields.insert(name.to_owned(), value.as_raw());
        }
        Ok(())
    }

    pub fn obj_get(&mut self, instance: &ObjectSmartRefNN, name: &str) -> Result<ObjectSmartRef, VMError> {
        VMThreadState::change(self.into(), VMThreadState::RunningVM);
        if instance.flags.check_proxy() {
            let name = alloc_string(self.into(), name.to_owned())?;
            self.call_obj(instance, "__set__", &[name.into()])
        } else {
            Ok(instance.fields.get(name).map_or_else(ObjectSmartRef::null, ObjectSmartRef::from))
        }
    }

    pub fn call_obj(&mut self, instance: &ObjectSmartRefNN, func: &str, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        if instance.flags.check_proxy() {
            let name = alloc_string(self.into(), func.to_owned())?;
            let args = alloc_array(self.into(), args.to_owned())?;
            self.call_obj(instance, "__call__", &[name.into(), args.into()])
        } else if let Some(method) = instance.class.try_find_method(func) {
            let instance = instance.clone();
            let instance = Into::<ObjectSmartRef>::into(instance);
            self.call_func(&instance, method, args)
        } else if let Some(field) = instance.fields.get(func) && let Some(value) = field.try_deref() {
            let value = ObjectSmartRefNN::new(value);
            self.call_obj(&value, "__call__", args)
        } else {
            Err(VMError::MethodNotFound(func.to_owned()))
        }
    }

    pub fn call_class(&mut self, class: &str, func: &str, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        if let Some(class) = VMModuleManager::try_find_class(self.vm, class)? {
            let method = class.find_method(func)?;
            let (object, args) =
                if args.is_empty() || args.len() <= method.params().len() {
                    (ObjectSmartRef::null(), args)
                } else {
                    (args[0].clone(), &args[1..])
                };
            self.call_func(&object, method, args)
        } else if let Some(object) = VMModuleManager::try_find_object(self.vm, class)? {
            let method = object.class.find_method(func)?;
            let object = object.into();
            self.call_func(&object, method, args)
        } else {
            Err(VMError::ClassNotFound(class.to_owned()))
        }
    }
    
    pub fn call_module(&mut self, module: &str, func: &str, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        let module = VMModuleManager::find_module(self.vm, module)?;
        let method = module.find_method(func)?;
        self.call_func(&ObjectSmartRef::null(), method, args)
    }

    pub fn call_func(&mut self, instance: &ObjectSmartRef, func: &Function, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        let mut frame = self.frame_new(func);
        frame.locals.set_global("this", instance.as_raw());
        self.frame_init_args(frame, func.params(), args)?;
        let result = self.frame_exec(frame);
        self.frames.pop();
        result
    }

    pub fn frame_new(&mut self, func: &Function) -> VMStackFrameRef {
        let frame = Boxed::new(VMStackFrame::new(func));
        let ref_frame = frame.as_raw();
        hard_fence();
        self.frames.push(frame);
        hard_fence();
        VMStackFrameRef(ref_frame)
    }

    pub fn frame_init_args(&mut self, mut frame: VMStackFrameRef, params: &Array<String>, args: &[ObjectSmartRef]) -> Result<(), VMError> {
        if params.is_empty() {
            if !args.is_empty() {
                let args = alloc_array(self.into(), args.to_owned())?;
                frame.locals.set_global("__args__", args.as_raw().into());
            }
        } else {
            if args.len() < params.len() {
                for i in 0..args.len() {
                    frame.locals.set_global(&params[i], args[i].as_raw());
                }
                for i in args.len()..params.len() {
                    frame.locals.set_global(&params[i], ObjectRef::null());
                }
            } else {
                for i in 0..params.len() {
                    frame.locals.set_global(&params[i], args[i].as_raw());
                }
                let args = alloc_array(self.into(), args[params.len()..args.len()].to_owned())?;
                frame.locals.set_global("__args__", args.as_raw().into());
            }
        }
        Ok(())
    }

    pub fn frame_exec(&mut self, frame: VMStackFrameRef) -> Result<ObjectSmartRef, VMError> {
        // SAFETY: Гарантия вызывающей стороны.
        match unsafe { &*frame.function } {
            Function::VM { body, .. } => {
                VMThreadState::change(self.into(), VMThreadState::RunningVM);
                match self.exec_stmt_block(frame, body) {
                    Ok(()) | Err(VMError::__Return__) => {}
                    Err(err) => return Err(err)
                }
                let value = frame.locals.get_global("__return__");
                let value = ObjectSmartRef::new(value);
                Ok(value)
            }
            Function::Native { function, .. } => {
                VMThreadState::change(self.into(), VMThreadState::RunningNativeLock);
                // SAFETY: Гарантии стандарта.
                unsafe {
                    let result = function(self.into(), frame.into());
                    let result_ =
                        match &*result {
                            Ok(()) | Err(VMError::__Return__) => {
                                let value = frame.locals.get_global("__return__");
                                let value = ObjectSmartRef::new(value);
                                Ok(value)
                            }
                            Err(err) => Err(err.clone()),
                        };
                    napi::alloc::dealloc_native_function_result(result);
                    result_
                }
            }
        }
    }

    pub fn catch_unhandled(&mut self, exception: ObjectSmartRef) -> Result<ObjectSmartRef, VMError> {
        if let Some(catcher) = self.cather.try_deref() {
            self.call_obj(&ObjectSmartRefNN::new(catcher), "__call__", &[exception])
        } else if let Some(catcher) = self.vm.catcher.try_deref() {
            self.call_obj(&ObjectSmartRefNN::new(catcher), "__call__", &[exception])
        } else {
            let value = string_to_native(self.into(), exception)?;
            Err(VMError::UnhandledException(value))
        }
    }

    fn exec_stmt(&mut self, mut frame: VMStackFrameRef, stmt: &Statement) -> Result<(), VMError> {
        match stmt {
            Statement::Let(r#let) => {
                let value = self.exec_expr(frame, &r#let.value)?;
                frame.locals.let_block(&r#let.name, value.as_raw());
            },
            Statement::Assign(assign) => {
                let value = self.exec_expr(frame, &assign.value)?;
                match &assign.target {
                    AssignTarget::Var(name) => {
                        let target = frame.locals.get_block(name);
                        let target = ObjectSmartRef::new(target);
                        let value = self.exec_stmt_assign(&assign.op, target, value)?;
                        frame.locals.set_block(name, value.as_raw());
                    }
                    AssignTarget::Index { target, index } => {
                        let target = self.exec_expr(frame, target)?;
                        let target = target.deref()?;
                        let index = self.exec_expr(frame, index)?;
                        let value = self.exec_stmt_assign(&assign.op, target.clone().into(), value)?;
                        self.call_obj(&target, "__set__", &[index, value])?;
                    }
                    AssignTarget::Member { target, name } => {
                        let target = self.exec_expr(frame, target)?;
                        let mut target = target.deref()?;
                        let member = self.obj_get(&target, name)?;
                        let value = self.exec_stmt_assign(&assign.op, member, value)?;
                        self.obj_set(&mut target, name, value)?;
                    }
                }

            }
            Statement::If(r#if) => {
                let condition = self.exec_expr(frame, &r#if.cond)?;
                let condition = bool_to_native(self.into(), condition)?;
                if condition {
                    self.exec_stmt_block(frame, &r#if.then_block)?;
                } else if let Some(else_block) = &r#if.else_block {
                    self.exec_stmt_block(frame, else_block)?;
                }
            }
            Statement::While(r#while) => {
                while {
                    let condition = self.exec_expr(frame, &r#while.cond)?;
                    bool_to_native(self.into(), condition)?
                } {
                    self.exec_stmt_block(frame, &r#while.body)?;
                }
            }
            Statement::For(r#for) => {
                let iterable = self.exec_expr(frame, &r#for.iter)?;
                let iterable = iterable.deref()?;
                let iterator = self.call_obj(&iterable, "__iter__", &[])?;
                let iterator = iterator.deref()?;
                while {
                    let has_next = self.call_obj(&iterator, "has_next", &[])?;
                    bool_to_native(self.into(), has_next)?
                } {
                    frame.locals.push_block();
                    let next = self.call_obj(&iterator, "next", &[])?;
                    frame.locals.let_block(&r#for.var, next.as_raw());
                    self.exec_stmt_block(frame, &r#for.body)?;
                    frame.locals.pop_block();
                }
            }
            Statement::Expr(expr) => {
                self.exec_expr(frame, expr)?;
            }
            Statement::Return(r#return) => {
                if let Some(value) = r#return {
                    let value = self.exec_expr(frame, value)?;
                    frame.locals.set_global("__return__", value.as_raw());
                }
                return Err(VMError::__Return__)
            }
            Statement::Block(block) => {
                frame.locals.push_block();
                self.exec_stmt_block(frame, block)?;
                frame.locals.pop_block();
            }
            Statement::TryCatch(try_catch) => {
                match self.exec_stmt_block(frame, &try_catch.try_block) {
                    Ok(()) => {}
                    Err(VMError::__Throwing__(exception)) => {
                        frame.locals.push_block();
                        frame.locals.let_block(&try_catch.catch_param, exception.as_raw());
                        self.exec_stmt_block(frame, &try_catch.catch_block)?;
                        frame.locals.pop_block();
                    }
                    Err(err) => return Err(err)
                }
            }
            Statement::Throw(throw) => {
                let exception = self.exec_expr(frame, throw)?;
                return Err(VMError::__Throwing__(exception))
            }
        };
        Ok(())
    }

    fn exec_stmt_assign(&mut self, assign: &AssignOp, target: ObjectSmartRef, value: ObjectSmartRef) -> Result<ObjectSmartRef, VMError> {
        if *assign == AssignOp::Assign {
            Ok(value)
        } else {
            let operation =
                match &assign {
                    AssignOp::Assign => unreachable!(),
                    AssignOp::PlusEq => "__add__",
                    AssignOp::MinusEq => "__sub__",
                    AssignOp::StarEq => "__mul__",
                    AssignOp::SlashEq => "__div__",
                    AssignOp::PercentEq => "__mod__",
                    AssignOp::AndEq => "__and__",
                    AssignOp::OrEq => "__or__",
                    AssignOp::XorEq => "__xor__",
                    AssignOp::NotEq => "__not__",
                    AssignOp::LtEq => "__lt__",
                    AssignOp::GtEq => "__gt__",
                    AssignOp::LeEq => "__le__",
                    AssignOp::GeEq => "__ge__"
                };
            let target = target.deref()?;
            let value = self.call_obj(&target, operation, &[value])?;
            Ok(value)
        }
    }

    fn exec_stmt_block(&mut self, frame: VMStackFrameRef, block: &Block) -> Result<(), VMError> {
        for stmt in &block.statements {
            self.exec_stmt(frame, stmt)?;
        }
        Ok(())
    }

    fn exec_expr(&mut self, frame: VMStackFrameRef, expr: &Expr) -> Result<ObjectSmartRef, VMError> {
        match expr {
            Expr::Literal(literal) => {
                match literal {
                    Literal::Boolean(value) => alloc_bool(self.into(), *value).map(|x| x.into()),
                    Literal::Integer(value) => alloc_i64(self.into(), *value).map(|x| x.into()),
                    Literal::Double(value) => alloc_f64(self.into(), *value).map(|x| x.into()),
                    Literal::String(value) => alloc_string(self.into(), value.clone()).map(|x| x.into()),
                    Literal::Array(value) => {
                        let mut array = Vec::new();
                        for element in value {
                            array.push(self.exec_expr(frame, element)?);
                        }
                        alloc_array(self.into(), array).map(|x| x.into())
                    },
                    Literal::Map(value) => {
                        let mut map = Vec::new();
                        for (left, right) in value {
                            let left = self.exec_expr(frame, left)?;
                            let right = self.exec_expr(frame, right)?;
                            map.push((left, right));
                        }
                        alloc_map(self.into(), map).map(|x| x.into())
                    },
                }
            }
            Expr::Var(var) => {
                let value = frame.locals.get_block(var);
                let value = ObjectSmartRef::new(value);
                Ok(value)
            }
            Expr::Binary(binary) => {
                let left = self.exec_expr(frame, &binary.left)?;
                let right = self.exec_expr(frame, &binary.right)?;
                let left =
                    match &binary.op {
                        BinaryOp::Eq => {
                            if let Some(left) = left.try_deref() {
                                left
                            } else {
                                let value = right.is_null();
                                let value = alloc_bool(self.into(), value)?;
                                return Ok(value.into());
                            }
                        },
                        BinaryOp::Neq => {
                            if let Some(left) = left.try_deref() {
                                left
                            } else {
                                let value = !right.is_null();
                                let value = alloc_bool(self.into(), value)?;
                                return Ok(value.into());
                            }
                        },
                        _ => left.deref()?
                    };
                let operation =
                    match &binary.op {
                        BinaryOp::Add => "__add__",
                        BinaryOp::Sub => "__sub__",
                        BinaryOp::Mul => "__mul__",
                        BinaryOp::Div => "__div__",
                        BinaryOp::Mod => "__mod__",
                        BinaryOp::And => "__and__",
                        BinaryOp::Or => "__or__",
                        BinaryOp::Xor => "__xor__",
                        BinaryOp::Eq => "__eq__",
                        BinaryOp::Neq => "__neq__",
                        BinaryOp::Lt => "__lt__",
                        BinaryOp::Gt => "__gt__",
                        BinaryOp::Le => "__le__",
                        BinaryOp::Ge => "__ge__"
                    };
                let value = self.call_obj(&left, operation, &[right])?;
                Ok(value)
            }
            Expr::Unary(unary) => {
                let target = self.exec_expr(frame, &unary.expr)?;
                let target = target.deref()?;
                let operation =
                    match &unary.op {
                        UnaryOp::Neg => "__neg__",
                        UnaryOp::Not => "__not__"
                    };
                let value = self.call_obj(&target, operation, &[])?;
                Ok(value)
            }
            Expr::Call(call) => {
                let target = self.exec_expr(frame, &call.callee)?;
                let target = target.deref()?;
                let args = self.exec_arguments(frame, &call.args)?;
                let value = self.call_obj(&target, "__call__", &args)?;
                Ok(value)
            }
            Expr::Index(index) => {
                let target = self.exec_expr(frame, &index.target)?;
                let target = target.deref()?;
                let index = self.exec_expr(frame, &index.index)?;
                let value = self.call_obj(&target, "__get__", &[index])?;
                Ok(value)
            }
            Expr::Member(member) => {
                let target = self.exec_expr(frame, &member.target)?;
                let target = target.deref()?;
                let value = self.obj_get(&target, &member.name)?;
                Ok(value)
            }
            Expr::MethodCallColon(call) => {
                let args = self.exec_arguments(frame, &call.args)?;
                let value = self.call_class(&call.class, &call.method, &args)?;
                Ok(value)
            }
            Expr::MethodCallDot(call) => {
                let target = self.exec_expr(frame, &call.target)?;
                let target = target.deref()?;
                let args = self.exec_arguments(frame, &call.args)?;
                let value = self.call_obj(&target, &call.method, &args)?;
                Ok(value)
            }
            Expr::ModuleCall(call) => {
                let module = VMModuleManager::find_module(self.vm, &call.module)?;
                let module = module.find_method(&call.function)?;
                let args = self.exec_arguments(frame, &call.args)?;
                let value = self.call_func(&ObjectSmartRef::null(), module, &args)?;
                Ok(value)
            }
            Expr::New(new) => {
                let class = VMModuleManager::find_class(self.vm, &new.class)?;
                let instance = VMHeap::alloc(self.vm, class)?;
                let args = self.exec_arguments(frame, &new.args)?;
                let value = self.call_obj(&instance, "__init__", &args)?;
                Ok(value)
            }
            Expr::Function(function) => {
                let params = Array::from(&function.params);
                let captures =
                    if let Some(captures) = &function.captures {
                        captures
                            .iter()
                            .map(|x| (x.to_owned(), ObjectSmartRef::new(frame.locals.get_block(x))))
                            .collect()
                    } else {
                        let mut captures = Vec::new();
                        frame.locals.blocks.iter().for_each(|block| {
                            block.iter().for_each(|(name, value)| {
                                captures.push((name.to_owned(), ObjectSmartRef::new(value.clone())));
                            });
                        });
                        Array::from(captures)
                    };
                let body = function.body.clone();
                let value = alloc_func_ast(self.into(), params, captures, body)?;
                Ok(value.into())
            }
            Expr::ObjectRef(object) => {
                let value = VMModuleManager::find_object(self.vm, object)?;
                Ok(value.into())
            }
        }
    }

    fn exec_arguments(&mut self, frame: VMStackFrameRef, args: &Vec<Expr>) -> Result<Array<ObjectSmartRef>, VMError> {
        let mut list = Vec::new();
        for arg in args {
            list.push(self.exec_expr(frame, &arg)?);
        }
        Ok(Array::from(list))
    }

    pub fn this<'a, 'b>(&'a mut self) -> &'b mut Self {
        ownership_hack_mut(self)
    }
}

impl From<&mut VMThread> for VMThreadRef {
    fn from(value: &mut VMThread) -> Self {
        Self(value)
    }
}

impl Deref for VMThreadRef {
    type Target = VMThread;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { &*self.0 }
    }
}

impl DerefMut for VMThreadRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { &mut *self.0 }
    }
}

unsafe impl Send for VMThreadRef {}
unsafe impl Sync for VMThreadRef {}

impl VMThreadOwner {
    pub fn change_owning(&self) {
        let tid = thread_id::get();
        self.0.store(tid, Ordering::Release);
    }

    pub fn catch_owning(&self) {
        if self.is_owning() { return }
        while !self.try_catch_owning() {
            std::thread::yield_now();
            std::hint::spin_loop();
        }
    }

    pub fn try_catch_owning(&self) -> bool {
        let tid = thread_id::get();
        self.0.compare_exchange(0, tid, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    pub fn is_owning(&self) -> bool {
        let tid = thread_id::get();
        self.0.load(Ordering::Acquire) == tid
    }

    pub fn drop_owning(&self) {
        self.0.store(0, Ordering::Release)
    }
}

impl VMThreadState {
    pub fn change(mut this: VMThreadRef, state: VMThreadState) {
        if state.is_work() {
            if this.flags.gc {
                this.flags.state = VMThreadState::Suspended;
                this.vm.flags.wait_allow_execution();
            } else {
                this.flags.state = VMThreadState::SuspendedGC;
                this.vm.flags.wait_allow_execution_and_no_gc();
            }
        }

        this.flags.state = state;
    }

    pub fn is_gc_allow(&self) -> bool {
        match self {
            VMThreadState::RunningVM |
            VMThreadState::RunningNativeLock => false,
            _ => true
        }
    }

    pub fn is_work(&self) -> bool {
        match self {
            VMThreadState::RunningVM |
            VMThreadState::RunningNativeLock |
            VMThreadState::RunningNativeClean => true,
            _ => false
        }
    }

    pub fn is_live(&self) -> bool {
        match self {
            VMThreadState::Ready |
            VMThreadState::RunningVM |
            VMThreadState::RunningNativeLock |
            VMThreadState::RunningNativeClean |
            VMThreadState::Suspended |
            VMThreadState::SuspendedGC |
            VMThreadState::Yielding |
            VMThreadState::YieldingNative => true,
            _ => false
        }
    }
}

impl VMStackFrame {
    pub fn new(function: &Function) -> Self {
        Self {
            function,
            locals: VMStackFrameLocals::new()
        }
    }
}

impl From<&mut VMStackFrame> for VMStackFrameRef {
    fn from(value: &mut VMStackFrame) -> Self {
        Self(value)
    }
}

impl From<*mut VMStackFrame> for VMStackFrameRef {
    fn from(value: *mut VMStackFrame) -> Self {
        Self(value)
    }
}

impl Deref for VMStackFrameRef {
    type Target = VMStackFrame;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { &*self.0 }
    }
}

impl DerefMut for VMStackFrameRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { &mut *self.0 }
    }
}

impl VMStackFrameLocals {
    pub fn new() -> Self {
        Self {
            blocks: vec![HashMap::new()]
        }
    }

    pub fn set_global(&mut self, name: &str, value: ObjectRef) {
        let global = self.blocks.first_mut();
        // SAFETY: Глобальный блок всегда есть.
        let global = unsafe { global.unwrap_unchecked() };
        global.insert(name.to_string(), value);
    }

    pub fn get_global(&self, name: &str) -> ObjectRef {
        let global = self.blocks.first();
        // SAFETY: Глобальный блок всегда есть.
        let global = unsafe { global.unwrap_unchecked() };
        global.get(name).map_or_else(|| ObjectRef::null(), |x| x.clone())
    }

    pub fn push_block(&mut self) {
        self.blocks.push(HashMap::new())
    }

    pub fn pop_block(&mut self) {
        self.blocks.pop();
    }

    pub fn let_block(&mut self, name: &str, value: ObjectRef) {
        let last = self.blocks.last_mut();
        // SAFETY: Глобальный блок всегда есть.
        let last = unsafe { last.unwrap_unchecked() };
        last.insert(name.to_string(), value);
    }

    pub fn set_block(&mut self, name: &str, value: ObjectRef) {
        for block in self.blocks.iter_mut().rev() {
            if let Some(element) = block.get_mut(name) {
                *element = value;
            }
        }
    }

    pub fn get_block(&self, name: &str) -> ObjectRef {
        for block in self.blocks.iter().rev() {
            if let Some(value) = block.get(name) {
                return value.clone();
            }
        }
        ObjectRef::null()
    }
}