#[cfg(not(feature = "single_thread"))]
#[derive(Clone, Debug)]
pub(crate) struct ReentrantMutexWrap<T> {
    imp: pi_share::Share<parking_lot::ReentrantMutex<T>>,
}

#[cfg(feature = "single_thread")]
#[derive(Clone, Debug)]
pub(crate) struct ReentrantMutexWrap<T>(std::marker::PhantomData<T>);

#[cfg(not(feature = "single_thread"))]
#[derive(Debug)]
pub(crate) struct ReentrantMutexGuardWrap<'a, T> {
    imp: parking_lot::ReentrantMutexGuard<'a, T>,
}

#[cfg(feature = "single_thread")]
#[derive(Debug)]
pub(crate) struct ReentrantMutexGuardWrap<'a, T>(&'a std::marker::PhantomData<T>);

impl<T> ReentrantMutexWrap<T> {
    #[inline]
    pub(crate) fn new(val: T) -> Self {
        #[cfg(not(feature = "single_thread"))]
        {
            Self {
                imp: pi_share::Share::new(parking_lot::ReentrantMutex::new(val)),
            }
        }
        #[cfg(feature = "single_thread")]
        {
            Self(std::marker::PhantomData)
        }
    }

    #[inline]
    pub(crate) fn try_lock_for<'a>(
        &'a self,
        timeout: std::time::Duration,
    ) -> Option<ReentrantMutexGuardWrap<'_, T>> {
        #[cfg(not(feature = "single_thread"))]
        {
            self.imp
                .as_ref()
                .try_lock_for(timeout)
                .map(|v| ReentrantMutexGuardWrap { imp: v })
        }

        #[cfg(feature = "single_thread")]
        {
            Some(ReentrantMutexGuardWrap::<'_, T>(&std::marker::PhantomData))
        }
    }
}
