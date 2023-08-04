#[derive(Debug)]
pub struct Fence;

unsafe impl Send for Fence {}
unsafe impl Sync for Fence {}
