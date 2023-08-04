#[derive(Debug)]
pub struct QuerySet;

unsafe impl Send for QuerySet {}
unsafe impl Sync for QuerySet {}
