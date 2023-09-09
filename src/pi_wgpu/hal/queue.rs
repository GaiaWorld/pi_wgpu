use super::{AdapterContext, GLState};

#[derive(Debug)]
pub(crate) struct Queue {
    pub(crate) state: GLState,
    pub(crate) adapter: AdapterContext,
}
