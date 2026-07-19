use crate::napi::ptr::ObjectSmartRefNN;
use crate::utils::alloc::Boxed;
use crate::vm::module::ClassRef;
use crate::vm::thread::{VMThreadManager, VMThreadRef, VMThreadState};
use crate::vm::{VMError, VMRef, VMState};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::std::thread::thread::hard_fence;

pub struct VMHeap {
    pub gc: VMHeapGC,
    pub objects: Mutex<Vec<ObjectRefNN>>,
    pub pin_counters: Mutex<HashMap<ObjectRefNN, AtomicU32>>,
    pub weak_handlers: Mutex<HashMap<ObjectRefNN, Vec<Boxed<ObjectWeakHandle>>>>,
}

pub struct VMHeapGC {
    pub mark_flag: bool,
    pub for_marking: *mut Vec<ObjectRefNN>
}

pub struct Object {
    pub class: ClassRef,
    pub fields: HashMap<String, ObjectRef>,
    pub flags: ObjectFlags,
}

#[derive(PartialEq, Eq)]
pub struct ObjectWeakHandle {
    pub object: ObjectRef
}

pub struct ObjectFlags(AtomicU64);

/// Ссылка на объект (nullable)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ObjectRef(pub *mut Object);
/// Ссылка на объект (not null)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ObjectRefNN(pub NonNull<Object>);

/// Умная ссылка на объект (not null)
#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ObjectSoftLockRef(pub ObjectRefNN);
/// Слабая ссылка на объект (nullable)
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct ObjectWeakRef(pub NonNull<ObjectWeakHandle>);

impl VMHeap {
    pub fn new() -> Self {
        Self {
            gc: VMHeapGC {
                mark_flag: false,
                for_marking: null_mut(),
            },
            objects: Mutex::new(Vec::new()),
            pin_counters: Mutex::new(HashMap::new()),
            weak_handlers: Mutex::new(HashMap::new()),
        }
    }

    pub fn alloc(vm: VMRef, class: ClassRef) -> Result<ObjectSmartRefNN, VMError> {
        // SAFETY: Гарантия вызывающей стороны.
        let object = unsafe { std::alloc::alloc_zeroed(class.layout) as *mut Object };
        if object.is_null() { return Err(VMError::OutOfMemory) }
        // SAFETY: Проверка is_null.
        let mut object = unsafe { ObjectRefNN::new_unchecked(object) };
        object.class = class;
        object.fields = HashMap::new();
        object.flags = ObjectFlags::new();
        object.flags.set_gc_mark(vm.heap.gc.mark_flag);
        vm.heap.objects.lock().push(object);
        let object = ObjectSmartRefNN::new(object);
        Ok(object)
    }

    pub fn new_weak(vm: VMRef, object: ObjectRefNN) -> ObjectWeakRef {
        let weak = Boxed::new(
            ObjectWeakHandle {
                object: object.into()
            }
        );
        let ref_weak = (&weak).into();
        vm.heap.weak_handlers.lock().entry(object).or_insert(Vec::new()).push(weak);
        ObjectWeakRef(ref_weak)
    }

    pub fn drop_weak(vm: VMRef, weak: &ObjectWeakRef) {
        // SAFETY: Гарантия вызывающей стороны.
        let weak = unsafe { weak.0.as_ref() };
        // SAFETY: Гарантия структуры.
        let object = unsafe { ObjectRefNN::new_unchecked(weak.object.0) };
        let mut weak_handlers = vm.heap.weak_handlers.lock();
        let handlers = weak_handlers.get_mut(&object);
        // SAFETY: ObjectWeakRef всегда создаёт запись в weak_handlers.
        let handlers = unsafe { handlers.unwrap_unchecked() };
        let index = handlers.iter().position(|x| x.deref() == weak);
        // SAFETY: ObjectWeakRef всегда создаёт запись в weak_handlers.
        let index = unsafe { index.unwrap_unchecked() };
        handlers.remove(index);
        if handlers.is_empty() {
            object.flags.unmark_weak();
            weak_handlers.remove(&object);
        }
    }
}

impl VMHeapGC {
    pub fn gc(mut vm: VMRef, thread: Option<VMThreadRef>) -> Result<(), VMError> {
        // Проверка на повторный запуск сборки мусора
        if vm.flags.get_state() == VMState::GC { return Ok(()); }
        // Ожидание нужного состояния
        let state_save =
            loop {
                let state = vm.flags.get_state();
                if state.is_allow_gc() { break state; }
                std::thread::yield_now();
                std::hint::spin_loop();
            };
        // Установка состояния сборки мусора (останавливаем потоки)
        vm.flags.set_state(VMState::GC, false);
        // Обновляем состояние текущего потока
        if let Some(mut thread) = thread { thread.flags.state = VMThreadState::SuspendedGC }
        // Ожидание остановки потоков
        Self::gc_wait_thread_stop(vm);
        // Обновление состояния сборки мусора (разрешаем выполнение для маркировки)
        vm.flags.set_state(VMState::GC, true);
        // Маркировка
        let mark = !vm.heap.gc.mark_flag;
        vm.heap.gc.mark_flag = mark;
        // Поиск вершины достижимых объектов
        let mut for_marking = Vec::<ObjectRefNN>::new();
        vm.heap.gc.for_marking = &mut for_marking;
        // - Глобальный catcher
        if let Some(object) = vm.catcher.try_deref() {
            Self::gc_add_to_marking(vm, mark, &mut for_marking, object)?;
        }
        // // - Закреплённые в памяти
        // for (object, _) in vm.heap.pin_counters.lock().iter() {
        //     Self::gc_add_to_marking(vm, mark, &mut for_marking, object.clone())?;
        // }
        // - Счётчик ссылок
        for object in vm.heap.objects.lock().iter().filter(|x| x.flags.get_rc() != 0) {
            Self::gc_add_to_marking(vm, mark, &mut for_marking, object.clone())?;
        }
        // - Объекты модулей
        if state_save != VMState::Stopped {
            for module in vm.modules.modules.read().iter() {
                for object in module.objects.iter() {
                    Self::gc_add_to_marking(vm, mark, &mut for_marking, object.instance())?;
                }
            }
        } else {
            for module in vm.modules.modules.read().iter().filter(|module| module.path.starts_with("std")) {
                for object in module.objects.iter() {
                    Self::gc_add_to_marking(vm, mark, &mut for_marking, object.instance())?;
                }
            }
        }
        // - Объекты в кадрах живых потоков
        let _lock = vm.threads.lock.lock();
        for i in 0..vm.threads.threads.len() {
            let thread = VMThreadRef(vm.threads.threads[i].as_raw());
            if thread.flags.state.is_live() {
                for frame in &thread.frames {
                    for block in &frame.locals.blocks {
                        for local in block.values() {
                            if let Some(local) = local.try_deref() {
                                Self::gc_add_to_marking(vm, mark, &mut for_marking, local)?;
                            }
                        }
                    }
                }
                hard_fence();
                if let Some(cather) = thread.cather.try_deref() {
                    Self::gc_add_to_marking(vm, mark, &mut for_marking, cather)?;
                }
            }
        }
        drop(_lock);
        // Маркировка
        while let Some(object) = for_marking.pop() {
            for field in object.fields.values() {
                if let Some(field) = field.try_deref() {
                    Self::gc_add_to_marking(vm, mark, &mut for_marking, field)?;
                }
            }
        }
        vm.heap.gc.for_marking = null_mut();
        // Установка состояния сборки мусора (останавливаем потоки)
        vm.flags.set_state(VMState::GC, false);
        // Ожидание остановки потоков
        Self::gc_wait_thread_stop(vm);
        // Удаление немаркированных объектов и сбор объектов с деструкторами.
        let mut weak_handlers = vm.this().heap.weak_handlers.lock();
        let mut for_dealloc = Vec::new();
        vm.heap.objects.lock().retain(|object| {
            // SAFETY: Heap не хранит невалидные ссылки.
            let keep = object.flags.get_gc_mark() == mark;
            if !keep {
                if object.flags.check_uninit() {
                    for_dealloc.push(object.clone());
                } else {
                    Self::gc_dealloc(&mut weak_handlers, object.clone());
                }
            }
            keep
        });
        // Обновление состояния сборки мусора (разрешаем выполнение для вызова деструкторов)
        vm.flags.set_state(VMState::GC, true);
        // Вызов деструкторов и удаление объектов.
        while let Some(object) = for_dealloc.pop() {
            vm.clone().call_obj(&ObjectSmartRefNN::new(object), "__uninit__", &[])?;
            Self::gc_dealloc(&mut weak_handlers, object);
        }
        // Восстанавливаем работу
        vm.flags.set_state(state_save, true);
        Ok(())
    }

    pub fn gc_mark(mut thread: VMThreadRef, object: &ObjectSmartRefNN) -> Result<(), VMError> {
        if object.flags.set_gc_mark_if_changed(thread.vm.heap.gc.mark_flag) {
            // SAFETY: Гарантия вызывающей стороны.
            unsafe { (*thread.vm.heap.gc.for_marking).push(object.as_raw()); }
            if object.flags.check_marker() {
                thread.clone().call_obj(&object, "__mark__", &[])?;
            }
        }
        Ok(())
    }

    fn gc_wait_thread_stop(vm: VMRef) {
        while !VMThreadManager::check_threads_is_allow_gc(vm) {
            std::thread::yield_now();
            std::hint::spin_loop();
        }
    }

    fn gc_dealloc(weak_handlers: &mut HashMap<ObjectRefNN, Vec<Boxed<ObjectWeakHandle>>>, mut object: ObjectRefNN) {
        // Сбрасываем weak
        if let Some(handlers) = weak_handlers.get_mut(&object) {
            handlers.iter_mut().for_each(|handler| {
                handler.object = ObjectRef::null();
            });
        }
        // Высвобождаем память
        // SAFETY: Ответственность вызывающей стороны.
        unsafe {
            std::ptr::drop_in_place(&mut object.fields);
            std::alloc::dealloc(object.0.as_ptr() as *mut _, object.class.deref().layout)
        };
    }

    fn gc_add_to_marking(vm: VMRef, mark: bool, for_marking: &mut Vec<ObjectRefNN>, object: ObjectRefNN) -> Result<(), VMError> {
        if object.flags.set_gc_mark_if_changed(mark) {
            for_marking.push(object);
            if object.flags.check_marker() {
                vm.clone().call_obj(&ObjectSmartRefNN::new(object), "__mark__", &[])?;
            }
        }
        Ok(())
    }
}

impl ObjectFlags {
    const GC_MARK_MASK: u64     = 0b00000001 << 56;
    const WEAK_REF_MASK: u64    = 0b00000010 << 56;
    const LOCK_REF_MASK: u64    = 0b00000100 << 56;
    const _0: u64               = 0b00001000 << 56;
    const UNINIT_MARK_MASK: u64 = 0b00010000 << 56;
    const MARKER_MARK_MAST: u64 = 0b00100000 << 56;
    const PROXY_MARK_MASK: u64  = 0b01000000 << 56;
    const _2: u64               = 0b10000000 << 56;
    const RC_MASK: u64          = 0x00FF_FFFF_FFFF_FFFF;

    pub fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    // ----- Reference Counter -----
    pub fn inc_rc(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel)
    }

    pub fn dec_rc(&self) -> u64 {
        self.0.fetch_sub(1, Ordering::AcqRel)
    }

    pub fn get_rc(&self) -> u64 {
        self.0.load(Ordering::Acquire) & Self::RC_MASK
    }


    // ---------- GC mark ----------
    pub fn set_gc_mark_if_changed(&self, mark: bool) -> bool {
        let mask = Self::GC_MARK_MASK;
        self.0
            .try_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                let bit = old & mask;
                let target = if mark { mask } else { 0 };
                if bit == target {
                    None
                } else {
                    Some(if mark { old | mask } else { old & !mask })
                }
            })
            .is_ok()
    }

    pub fn set_gc_mark(&self, mark: bool) {
        if mark {
            self.0.fetch_or(Self::GC_MARK_MASK, Ordering::Release);
        } else {
            self.0.fetch_and(!Self::GC_MARK_MASK, Ordering::Release);
        }
    }

    pub fn get_gc_mark(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::GC_MARK_MASK != 0
    }

    // ---------- Weak ref ----------
    pub fn mark_weak(&self) {
        self.0.fetch_or(Self::WEAK_REF_MASK, Ordering::Release);
    }

    pub fn unmark_weak(&self) {
        self.0.fetch_and(!Self::WEAK_REF_MASK, Ordering::Release);
    }

    pub fn check_weak(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::WEAK_REF_MASK != 0
    }

    // ---------- Lock ref ----------
    pub fn mark_lock(&self) {
        self.0.fetch_or(Self::LOCK_REF_MASK, Ordering::Release);
    }

    pub fn unmark_lock(&self) {
        self.0.fetch_and(!Self::LOCK_REF_MASK, Ordering::Release);
    }

    pub fn check_lock(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::LOCK_REF_MASK != 0
    }

    // ---------- Uninit mark ----------
    pub fn mark_uninit(&self) {
        self.0.fetch_or(Self::UNINIT_MARK_MASK, Ordering::Release);
    }

    pub fn unmark_uninit(&self) {
        self.0.fetch_and(!Self::UNINIT_MARK_MASK, Ordering::Release);
    }

    pub fn check_uninit(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::UNINIT_MARK_MASK != 0
    }

    // ---------- Marker mark ----------
    pub fn mark_marker(&self) {
        self.0.fetch_or(Self::MARKER_MARK_MAST, Ordering::Release);
    }

    pub fn unmark_marker(&self) {
        self.0.fetch_and(!Self::MARKER_MARK_MAST, Ordering::Release);
    }

    pub fn check_marker(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::MARKER_MARK_MAST != 0
    }

    // ---------- Proxy mark ----------
    pub fn mark_proxy(&self) {
        self.0.fetch_or(Self::PROXY_MARK_MASK, Ordering::Release);
    }

    pub fn unmark_proxy(&self) {
        self.0.fetch_and(!Self::PROXY_MARK_MASK, Ordering::Release);
    }

    pub fn check_proxy(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::PROXY_MARK_MASK != 0
    }
}


impl ObjectRef {
    pub fn null() -> Self {
        Self(null_mut())
    }

    pub fn deref(&self) -> Result<ObjectRefNN, VMError> {
        ObjectRefNN::new(self.0)
    }

    pub fn try_deref(&self) -> Option<ObjectRefNN> {
        if self.0.is_null() { return None; }
        // SAFETY: Проверка is_null выше.
        Some(unsafe { ObjectRefNN::new_unchecked(self.0) })
    }
}

impl From<&mut Object> for ObjectRef {
    fn from(value: &mut Object) -> Self {
        Self(value)
    }
}

impl From<ObjectRefNN> for ObjectRef {
    fn from(value: ObjectRefNN) -> Self {
        Self(value.0.as_ptr())
    }
}

impl ObjectRefNN {
    pub fn new(value: *mut Object) -> Result<Self, VMError> {
        if value.is_null() {
            return Err(VMError::NullPointer)
        }
        // SAFETY: Проверка выше.
        Ok(unsafe { Self::new_unchecked(value) })
    }

    pub unsafe fn new_unchecked(value: *mut Object) -> Self {
        // SAFETY: Гарантии вызывающей стороны.
        Self(unsafe { NonNull::new_unchecked(value) })
    }
}

impl Deref for ObjectRefNN {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for ObjectRefNN {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Гарантия структуры.
        unsafe { self.0.as_mut() }
    }
}