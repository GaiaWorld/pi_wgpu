use super::{AdapterContext, GLState};

#[derive(Debug)]
pub(crate) struct Queue {
    pub(crate) state: GLState,
    pub(crate) adapter: AdapterContext,
}

impl Queue {
    #[inline]
    pub(crate) fn submit<I: IntoIterator<Item = super::CommandBuffer>>(&self, command_buffers: I) {
        self.state.clear_cache();
    }
}
