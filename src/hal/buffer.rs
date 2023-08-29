use super::BindTarget;

#[derive(Debug)]
pub(crate) struct Buffer {
    pub(crate) raw: glow::Buffer,
    pub(crate) target: BindTarget,

    pub(crate) stride: i32,
    pub(crate) size: i32,

    pub(crate) divisor_step: u32,
}
