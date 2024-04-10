use bitflags::bitflags;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, HasWindowHandle, HasDisplayHandle};
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

    pub(crate) fn create_surface<W: HasWindowHandle + HasDisplayHandle>(
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
    pub struct InstanceFlags: u32 {
        /// Generate debug information in shaders and objects.
        const DEBUG = 1 << 0;
        /// Enable validation, if possible.
        const VALIDATION = 1 << 1;
        /// Don't pass labels to wgpu-hal.
        const DISCARD_HAL_LABELS = 1 << 2;
        /// Whether wgpu should expose adapters that run on top of non-compliant adapters.
        ///
        /// Turning this on might mean that some of the functionality provided by the wgpu
        /// adapter/device is not working or is broken. It could be that all the functionality
        /// wgpu currently exposes works but we can't tell for sure since we have no additional
        /// transparency into what is working and what is not on the underlying adapter.
        ///
        /// This mainly applies to a Vulkan driver's compliance version. If the major compliance version
        /// is `0`, then the driver is ignored. This flag allows that driver to be enabled for testing.
        const ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER = 1 << 3;
    }
);

impl Default for InstanceFlags {
    fn default() -> Self {
        Self::from_build_config()
    }
}

impl InstanceFlags {
    /// Enable debugging and validation flags.
    pub fn debugging() -> Self {
        InstanceFlags::DEBUG | InstanceFlags::VALIDATION
    }

    /// Infer good defaults from the build type
    ///
    /// Returns the default flags and add debugging flags if the build configuration has `debug_assertions`.
    pub fn from_build_config() -> Self {
        if cfg!(debug_assertions) {
            return InstanceFlags::debugging();
        }

        InstanceFlags::empty()
    }

    /// Returns this set of flags, affected by environment variables.
    ///
    /// The presence of an environment variable implies that the corresponding flag should be set
    /// unless the value is "0" in which case the flag is unset. If the environment variable is
    /// not present, then the flag is unaffected.
    ///
    /// For example `let flags = InstanceFlags::debugging().with_env();` with `WGPU_VALIDATION=0`
    /// does not contain `InstanceFlags::VALIDATION`.
    ///
    /// The environment variables are named after the flags prefixed with "WGPU_". For example:
    /// - WGPU_DEBUG
    /// - WGPU_VALIDATION
    pub fn with_env(mut self) -> Self {
        fn env(key: &str) -> Option<bool> {
            std::env::var(key).ok().map(|s| match s.as_str() {
                "0" => false,
                _ => true,
            })
        }

        if let Some(bit) = env("WGPU_VALIDATION") {
            self.set(Self::VALIDATION, bit);
        }
        if let Some(bit) = env("WGPU_DEBUG") {
            self.set(Self::DEBUG, bit);
        }
        if let Some(bit) = env("WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER") {
            self.set(Self::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER, bit);
        }

        self
    }
}

#[derive(Clone, Debug)]
pub(crate) struct InstanceDescriptor<'a> {
    pub name: &'a str,
    pub flags: InstanceFlags,
    pub dx12_shader_compiler: wgt::Dx12Compiler,
}
