#[derive(Debug)]
pub struct Sampler;

unsafe impl Send for Sampler {}
unsafe impl Sync for Sampler {}
