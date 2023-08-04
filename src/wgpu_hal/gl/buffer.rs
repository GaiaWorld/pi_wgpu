#[derive(Debug)]
pub struct Buffer;

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}
