use std::any::TypeId;
use std::fmt::Debug;

use iced::advanced::graphics::futures::MaybeSend;

// Taken from https://doc.rust-lang.org/stable/src/core/any.rs.html
// adjusted to fit ReSet
pub trait ReSetAny: 'static + MaybeSend + Debug + Send + Sync {
    fn type_id(&self) -> TypeId;
}

impl<T: 'static + ?Sized + Debug + MaybeSend + Send + Sync> ReSetAny for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn ReSetAny {
    #[inline]
    pub fn is<T: ReSetAny>(&self) -> bool {
        // Get `TypeId` of the type this function is instantiated with.
        let t = TypeId::of::<T>();
        // Get `TypeId` of the type in the trait object (`self`).
        let concrete = self.type_id();
        // Compare both `TypeId`s on equality.
        t == concrete
    }

    #[inline]
    pub fn downcast_ref<T: ReSetAny>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: ReSetAny>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: ReSetAny>(&mut self) -> &mut T {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &mut *(self as *mut dyn ReSetAny as *mut T) }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: ReSetAny>(&self) -> &T {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &*(self as *const dyn ReSetAny as *const T) }
    }
}
