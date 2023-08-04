#[derive(Debug)]
pub struct PipelineLayout;

unsafe impl Send for PipelineLayout {}
unsafe impl Sync for PipelineLayout {}
