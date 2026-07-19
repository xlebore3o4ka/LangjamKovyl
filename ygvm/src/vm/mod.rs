pub mod heap;
pub mod module;
pub mod thread;

use crate::napi::ptr::{ObjectSmartRef, ObjectSmartRefNN};
use crate::ownership_hack_mut;
use crate::vm::heap::{ObjectRef, VMHeap, VMHeapGC};
use crate::vm::module::{Function, VMModuleManager};
use crate::vm::thread::{VMStackFrameRef, VMThreadManager, VMThreadRef, VMThreadState};
use crate::utils::alloc::Boxed;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum VMError {
    #[error("Out of memory error")]
    OutOfMemory,

    #[error("Null pointer exception")]
    NullPointer,

    #[error("FFI error: {0}")]
    FFI(String),

    #[error("Alias not found: {0}")]
    AliasNotFound(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Class not found: {0}")]
    ClassNotFound(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Unhandled exception: {0}")]
    UnhandledException(String),

    #[error("(unreachable)")]
    __Throwing__(ObjectSmartRef),

    #[error("(unreachable)")]
    __Return__,
}

pub struct VM {
    pub flags: VMFlags,
    pub heap: VMHeap,
    pub threads: VMThreadManager,
    pub modules: VMModuleManager,
    pub catcher: ObjectRef
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct VMRef(NonNull<VM>);

pub struct VMFlags(AtomicU32);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum VMState {
    Running = 0b0000_0000_0000_0000,
    GC      = 0b0000_0000_0000_0001,
    HotSwap = 0b0000_0000_0000_0010,
    Stopped = 0b0000_0000_0000_0011,
}

impl VM {
    pub fn new() -> Result<Boxed<Self>, VMError> {
        let mut this = Boxed::new(
            Self {
                flags: VMFlags::new(),
                heap: VMHeap::new(),
                threads: VMThreadManager::new(),
                modules: VMModuleManager::new(),
                catcher: ObjectRef::null(),
            }
        );
        // SAFETY: Гарантии структуры.
        let vm = this.deref_mut().into();
        crate::std::core::load(vm)?;
        crate::std::thread::load(vm)?;
        crate::std::utils::load(vm)?;
        crate::std::io::load(vm)?;
        crate::std::json::load(vm)?;
        Ok(this)
    }

    pub fn call_obj(&mut self, instance: &ObjectSmartRefNN, func: &str, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        let mut thread = VMThreadManager::new_thread(self.into());
        VMThreadState::change(thread, VMThreadState::RunningVM);
        let result = thread.call_obj(instance, func, args);
        let result = Self::process_call_result(thread, result);
        result
    }

    pub fn call_class(&mut self, class: &str, func: &str, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        let mut thread = VMThreadManager::new_thread(self.into());
        VMThreadState::change(thread, VMThreadState::RunningVM);
        let result = thread.call_class(class, func, args);
        let result = Self::process_call_result(thread, result);
        result
    }

    pub fn call_module(&mut self, module: &str, func: &str, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        let mut thread = VMThreadManager::new_thread(self.into());
        VMThreadState::change(thread, VMThreadState::RunningVM);
        let result = thread.call_module(module, func, args);
        let result = Self::process_call_result(thread, result);
        result
    }

    pub fn call_func(&mut self, instance: &ObjectSmartRef, func: &Function, args: &[ObjectSmartRef]) -> Result<ObjectSmartRef, VMError> {
        let mut thread = VMThreadManager::new_thread(self.into());
        VMThreadState::change(thread, VMThreadState::RunningVM);
        let result = thread.call_func(instance, func, args);
        let result = Self::process_call_result(thread, result);
        result
    }

    pub fn call_exec(&mut self, mut thread: VMThreadRef) -> Result<ObjectSmartRef, VMError> {
        thread.owner.catch_owning();
        VMThreadState::change(thread, VMThreadState::RunningVM);
        let mut result = Ok(ObjectSmartRef::null());
        loop {
            if thread.frames.is_empty() || result.is_err() {
                let result = Self::process_call_result(thread, result);
                thread.owner.drop_owning();
                return result;
            }
            let frame = thread.frames.last_mut();
            // SAFETY: Проверка is_empty выше.
            let frame = unsafe { frame.unwrap_unchecked() };
            let frame = VMStackFrameRef(frame.as_raw());
            result = thread.clone().frame_exec(frame);
            thread.frames.pop();
        }
    }

    pub fn stop(&mut self, hard: bool) -> Result<(), VMError> {
        if self.flags.get_state() == VMState::Stopped { return Ok(()) }
        if hard { self.flags.set_allow_execution(false); }
        while VMThreadManager::check_threads_is_work(self.into()) {
            std::thread::yield_now();
            std::hint::spin_loop();
        }
        self.threads.pool.join();
        self.flags.set_state(VMState::Stopped, true);
        self.stop0()?;
        Ok(())
    }

    pub fn try_stop(&mut self) -> Result<bool, VMError> {
        if self.flags.get_state() == VMState::Stopped || VMThreadManager::check_threads_is_work(self.into()) { return Ok(false) }
        self.flags.set_state(VMState::Stopped, true);
        self.stop0()?;
        Ok(true)
    }

    fn process_call_result(mut thread: VMThreadRef, result: Result<ObjectSmartRef, VMError>) -> Result<ObjectSmartRef, VMError> {
        match result {
            Ok(value) => {
                VMThreadState::change(thread, VMThreadState::Complete);
                Ok(value)
            },
            Err(VMError::__Throwing__(exception)) => {
                match thread.catch_unhandled(exception) {
                    Ok(value) => {
                        VMThreadState::change(thread, VMThreadState::Complete);
                        Ok(value)
                    },
                    Err(err) => {
                        VMThreadState::change(thread, VMThreadState::Died);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                VMThreadState::change(thread, VMThreadState::Died);
                Err(err)
            }
        }
    }

    fn stop0(&mut self) -> Result<(), VMError> {
        // Очистка потоков
        let _lock = self.threads.lock.lock();
        self.threads.threads.clear();
        drop(_lock);
        // Запуск деинициализации модулей
        let modules_count = self.modules.modules.read().len();
        for i in 0..modules_count {
            let module = &self.this().modules.modules.read()[i];
            if let Some(uninit) = module.try_find_method("__uninit__") {
                self.call_func(&ObjectSmartRef::null(), &uninit, &[])?;
            }
        }
        // Вызов сборки мусора
        VMHeapGC::gc(self.into(), None)?;
        // Очистка несистемных модулей
        self.modules.modules.write().retain(|module| module.path.starts_with("std/"));
        // Выгрузка системных модулей
        crate::std::core::unload(self.into())?;
        crate::std::thread::unload(self.into())?;
        crate::std::utils::unload(self.into())?;
        crate::std::io::unload(self.into())?;
        crate::std::json::unload(self.into())?;
        // Очистка остаточных объектов
        while let Some(object) = self.heap.objects.lock().pop() {
            let layout = object.class.layout;
            let object = object.0.as_ptr() as *mut u8;
            // SAFETY: Heap не хранит невалидные ссылки.
            unsafe { std::alloc::dealloc(object, layout); }
        }
        // Очистка системных модулей
        self.modules.modules.write().clear();
        // Конец остановки
        self.flags.set_state(VMState::Stopped, false);
        Ok(())
    }

    pub fn this<'a, 'b>(&'a mut self) -> &'b mut Self {
        ownership_hack_mut(self)
    }
}

impl From<&mut VM> for VMRef {
    fn from(value: &mut VM) -> Self {
        Self(NonNull::from(value))
    }
}

impl From<&mut Boxed<VM>> for VMRef {
    fn from(value: &mut Boxed<VM>) -> Self {
        // SAFETY: Boxed всегда содержит валидный указатель.
        Self(unsafe { NonNull::new_unchecked(value.as_raw()) })
    }
}

impl Deref for VMRef {
    type Target = VM;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { &*self.0.as_ptr() }
    }
}

impl DerefMut for VMRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl VMFlags {
    const STATE_MASK: u32 = 0b0000_0000_0000_0011;
    const EXEC_MASK: u32  = 0b0000_0000_0000_0100;

    pub fn new() -> Self {
        let this = Self(AtomicU32::new(0));
        this.set_allow_execution(true);
        this
    }

    // ---------- State management ----------
    pub fn set_state(&self, state: VMState, exec: bool) {
        let state_bits = state as u32 & Self::STATE_MASK;
        let exec_bit = if exec { Self::EXEC_MASK } else { 0 };
        let new_bits = state_bits | exec_bit;
        self.0.try_update(Ordering::Release, Ordering::Acquire, |old| { Some((old & !(Self::STATE_MASK | Self::EXEC_MASK)) | new_bits) }).unwrap();
    }

    pub fn wait_state(&self, state: VMState) {
        while self.get_state() != state {
            std::thread::yield_now();
            std::hint::spin_loop();
        }
    }

    pub fn get_state(&self) -> VMState {
        VMState::from(self.0.load(Ordering::Acquire) & Self::STATE_MASK)
    }

    // ---------- Exec mark ----------
    pub fn wait_allow_execution_and_no_gc(&self) {
        while !self.is_allow_execution() && self.get_state() != VMState::GC {
            std::thread::yield_now();
            std::hint::spin_loop();
        }
    }

    pub fn wait_allow_execution(&self) {
        while !self.is_allow_execution() {
            std::thread::yield_now();
            std::hint::spin_loop();
        }
    }

    pub fn set_allow_execution(&self, exec: bool) {
        if exec {
            self.0.fetch_or(Self::EXEC_MASK, Ordering::Release);
        } else {
            self.0.fetch_and(!Self::EXEC_MASK, Ordering::Release);
        }
    }

    pub fn is_allow_execution(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::EXEC_MASK != 0
    }
}

impl VMState {
    pub fn is_allow_gc(&self) -> bool {
        match self {
            VMState::Running | VMState::Stopped => true,
            VMState::GC | VMState::HotSwap => false,
        }
    }
}

impl From<u32> for VMState {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl Into<u32> for VMState {
    fn into(self) -> u32 {
        unsafe { std::mem::transmute(self) }
    }
}