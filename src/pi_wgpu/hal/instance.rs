use bitflags::bitflags;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;

use super::super::{hal::AdapterContext, wgt};

#[derive(Debug)]
pub(crate) struct Instance {
    context: AdapterContext,
}

impl Instance {
    pub(crate) fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
        let context = AdapterContext::default();

        Ok(Self { context })
    }

    // EGL 所谓的 枚举显卡，实际上是 取 系统默认设置的显卡！
    // 这里的迭代器，只返回一个值
    #[inline]
    pub(crate) fn enumerate_adapters(&self) -> Vec<super::super::ExposedAdapter<super::GL>> {
        super::Adapter::expose(self.context.clone())
            .into_iter()
            .collect()
    }

    pub(crate) fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        handle: &W,
    ) -> Result<super::Surface, super::InstanceError> {
        let context = self.context.clone();

        super::Surface::new(context, handle)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("Not supported")]
pub(crate) struct InstanceError;

bitflags!(
    /// Instance initialization flags.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub(crate) struct InstanceFlags: u32 {
        /// Generate debug information in shaders and objects.
        const DEBUG = 1 << 0;
        /// Enable validation, if possible.
        const VALIDATION = 1 << 1;
    }
);

#[derive(Clone, Debug)]
pub(crate) struct InstanceDescriptor<'a> {
    pub name: &'a str,
    pub flags: InstanceFlags,
    pub dx12_shader_compiler: wgt::Dx12Compiler,
}
