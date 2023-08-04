use crate::wgpu_hal as hal;

type HalInstance<A> = <A as hal::Api>::Instance;

#[derive(Default)]
pub struct Instance {
    #[allow(dead_code)]
    pub name: String,

    #[cfg(feature = "empty")]
    pub empty: Option<HalInstance<hal::Empty>>,

    #[cfg(feature = "gl")]
    pub gl: Option<HalInstance<hal::GL>>,
}
