
pub mod db {
    pub mod amd {
        pub const VENDOR: u32 = 0x1002;
    }
    pub mod apple {
        pub const VENDOR: u32 = 0x106B;
    }
    pub mod arm {
        pub const VENDOR: u32 = 0x13B5;
    }
    pub mod broadcom {
        pub const VENDOR: u32 = 0x14E4;
    }
    pub mod imgtec {
        pub const VENDOR: u32 = 0x1010;
    }
    pub mod intel {
        pub const VENDOR: u32 = 0x8086;
        pub const DEVICE_KABY_LAKE_MASK: u32 = 0x5900;
        pub const DEVICE_SKY_LAKE_MASK: u32 = 0x1900;
    }
    pub mod mesa {
        // Mesa does not actually have a PCI vendor id.
        //
        // To match Vulkan, we use the VkVendorId for Mesa in the gles backend so that lavapipe (Vulkan) and
        // llvmpipe (OpenGL) have the same vendor id.
        pub const VENDOR: u32 = 0x10005;
    }
    pub mod nvidia {
        pub const VENDOR: u32 = 0x10DE;
    }
    pub mod qualcomm {
        pub const VENDOR: u32 = 0x5143;
    }
}
