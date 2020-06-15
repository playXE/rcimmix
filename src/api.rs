use std::sync::atomic::{AtomicUsize,AtomicU32,Ordering};

pub unsafe trait UnsafeTrace {
    fn visit_children(&self,trace_fn: &mut dyn FnMut(*const dyn HeapTrait));
}

pub trait Trace {
    fn visit(&self,_trace_fn: &mut dyn FnMut(*const dyn HeapTrait)) {}
}

unsafe impl<T: Trace> UnsafeTrace for T {
    fn visit_children(&self,trace_fn: &mut dyn FnMut(*const dyn HeapTrait)) {
        self.visit(trace_fn);
    }
}

pub trait HeapTrait {
    fn slot(&self) -> *mut u8;
    fn get_fwd(&self) -> *mut u8;
    fn set_fwd(&self,addr: *mut u8);
    fn copy_to(&self,addr: *mut u8);
    fn addr(&self) -> *mut u8;
    fn gc_object(&self) -> *const GCObject<dyn Trace>;
    fn mark(&self) {}
    fn unmark(&self) {}
    fn is_marked(&self) -> bool {false}
}
macro_rules! no_gc {
    ($($item: ty)*) => {
        $(
            impl Trace for $item {}
        )*
    }
}

no_gc!(
    i8 i16 i32 i64 i128
    u8 u16 u32 u64 u128
    char String bool f32 f64 
    isize usize 
    std::fs::File
    &'static str
);

/// GC object
pub struct GCObject<T: Trace + ?Sized> {
    /// Forwarding pointer, also used to store next information:
    /// - does object span lines? (1 bit)
    /// - mark (1 bit)
    /// - is object forwarded? (1 bit)
    /// - new (1 bit)
    pub(crate) fwdptr: AtomicUsize,
    /// Reference count - shows how many objects point to this object
    pub(crate) rc: AtomicU32,
    pub(crate) value: T
}
use std::ptr::NonNull;

pub struct Handle<T: Trace + ?Sized> {
    pub(crate) object: NonNull<GCObject<T>> 
}
