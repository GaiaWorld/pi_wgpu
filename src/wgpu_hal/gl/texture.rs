#[derive(Debug)]
pub struct Texture;

unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

#[derive(Debug)]
pub struct TextureView;

unsafe impl Send for TextureView {}
unsafe impl Sync for TextureView {}
