use super::{AdapterContext, GLState};

#[derive(Debug)]
pub(crate) struct Queue {
    pub(crate) state: GLState,
    pub(crate) adapter: AdapterContext,
}

impl Queue {
    #[inline]
    pub(crate) fn submit<I: IntoIterator<Item = super::CommandBuffer>>(&self, _command_buffers: I) {
        let lock = self.adapter.lock(None);

        let gl = lock.get_glow();

        self.state.clear_cache(&gl);
    }
}
