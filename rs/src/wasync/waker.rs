extern crate alloc;
use {
    core::{
        marker::PhantomData,
        mem::ManuallyDrop,
        ops::Deref,
        task::{
            RawWaker,
            RawWakerVTable,
            Waker as CoreWaker,
        },
    },
    alloc::sync::Arc
};

pub trait Waker: Send + Sync {
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    fn wake_by_ref(arc_self: &Arc<Self>);
}

pub fn waker_vtable<W: Waker>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_arc_raw::<W>,
        wake_arc_raw::<W>,
        wake_by_ref_arc_raw::<W>,
        drop_arc_raw::<W>,
    )
}

#[allow(dead_code)]
pub fn waker<W>(wake: Arc<W>) -> CoreWaker
    where
        W: Waker,
{
    let ptr = Arc::into_raw(wake) as *const ();

    unsafe {
        CoreWaker::from_raw(
            RawWaker::new(
                ptr,
                waker_vtable::<W>(),
            )
        )
    }
}

unsafe fn increase_refcount<T: Waker>(data: *const ()) {
    let arc = ManuallyDrop::new(
        Arc::<T>::from_raw(
            data as *const T
        )
    );
    let _arc_clone: ManuallyDrop<_> = arc.clone();
}

unsafe fn clone_arc_raw<T: Waker>(data: *const ()) -> RawWaker {
    increase_refcount::<T>(data);
    RawWaker::new(data, waker_vtable::<T>())
}

unsafe fn wake_arc_raw<T: Waker>(data: *const ()) {
    let arc: Arc<T> = Arc::from_raw(data as *const T);
    Waker::wake(arc);
}

unsafe fn wake_by_ref_arc_raw<T: Waker>(data: *const ()) {
    // Retain Arc, but do not touch refcount by wrapping in ManuallyDrop.
    let arc = ManuallyDrop::new(
        Arc::<T>::from_raw(
            data as *const T
        )
    );
    Waker::wake_by_ref(&arc);
}

unsafe fn drop_arc_raw<T: Waker>(data: *const ()) {
    drop(Arc::<T>::from_raw(data as *const T))
}

#[derive(Debug)]
pub struct WakerRef<'a> {
    waker: ManuallyDrop<CoreWaker>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> WakerRef<'a> {
    pub fn new(waker: &'a CoreWaker) -> Self {
        let waker = ManuallyDrop::new(unsafe {
            core::ptr::read(waker)
        });
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }

    pub fn new_unowned(waker: ManuallyDrop<CoreWaker>) -> Self {
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }
}

impl Deref for WakerRef<'_> {
    type Target = CoreWaker;

    fn deref(&self) -> &CoreWaker {
        &self.waker
    }
}

#[inline]
pub fn waker_ref<W>(wake: &Arc<W>) -> WakerRef<'_>
    where
        W: Waker,
{
    let ptr = (&**wake as *const W) as *const ();
    let waker = ManuallyDrop::new(unsafe {
        CoreWaker::from_raw(RawWaker::new(
            ptr,
            waker_vtable::<W>(),
        ))
    });

    WakerRef::new_unowned(waker)
}