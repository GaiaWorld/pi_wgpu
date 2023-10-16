// 2023-[0-9]+-[0-9]+T[0-9]+:[0-9]+:[0-9]+\.[0-9a-zA-Z]+ TRACE [_a-zA-Z:]+
// BufferUsages(COPY_DST | VERTEX)    =>> BufferUsages::COPY_DST | BufferUsages::VERTEX
// source: Glsl =>> source: ShaderSource::Glsl
// stage: Vertex =>> stage: ShaderStage::Vertex
// stage: Fragment ==> stage: ShaderStage::Fragment
// visibility: ShaderStages(VERTEX | FRAGMENT) ==> visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT
// visibility: ShaderStages(VERTEX) ==> visibility: ShaderStages::VERTEX
// visibility: ShaderStages(FRAGMENT) ==> visibility: ShaderStages::FRAGMENT
// [BindGroupLayoutEntry ==> &[BindGroupLayoutEntry
// [BindGroupLayout {         ==> &[BindGroupLayout {
// BindGroupLayout {         ==> &BindGroupLayout
// TextureUsages(COPY_DST | TEXTURE_BINDING)
// ty: Buffer { ty: Uniform ==> ty: BindingType::Buffer { ty: BufferBindingType::Uniform
// ty: Texture { ==> ty: BindingType::Texture {
// defines: {} ==> defines: HashMap::default()
// ty: Texture { sample_type: Float  ==>  ty: BindingType::Texture { sample_type: TextureSampleType::Float
// view_dimension: TextureDimension::D2      ==>       view_dimension: TextureViewDimension::D2
// ty: Sampler(Filtering)    ====>    ty: BindingType::Sampler(SamplerBindingType::Filtering)
// ClampToEdge      =====>      AddressMode::ClampToEdge
//  mag_filter: Linear     =====>  mag_filter: FilterMode::Linear
//  min_filter: Linear     =====>  min_filter: FilterMode::Linear
//  mag_filter: Nearest     =====>  mag_filter: FilterMode::Nearest
//  min_filter: Nearest     =====>  min_filter: FilterMode::Nearest
// mipmap_filter: FilterMode::Linear    =====>  mipmap_filter: FilterMode::Linear
// mipmap_filter: Nearest   ====> mipmap_filter: FilterMode::Nearest
// min_binding_size: Some\(([0-9]+)\)   ====> min_binding_size: Some(NonZeroU64::new($1).unwrap())
// entries: []  ====> entries: &[]
// contents: [ ====> contents: &[
// usage: BufferUsages(VERTEX)       ======>   usage: BufferUsages::VERTEX
// BufferUsages(INDEX) =========>>> BufferUsages::INDEX
// dimension: D2 ======>   dimension: TextuareDimension::D2
// format: Rgba8Unorm ======> format: TextureFormat::Rgba8Unorm
// view_formats: [   ======> view_formats: &[
// format: Depth32Float =====> format: TextureFormat::Depth32Float
// aspect: All     ======> aspect: TextureAspect::All
// resource: Sampler =====> resource: BindingResource::Sampler
// resource: TextureView =======> resource: BindingResource::TextureView
// step_mode: Vertex  ===== > step_mode: VertexStepMode::Vertex
// format: Float32x2 ======> format: VertexFormat::Float32x2
// front_face: Ccw ========> front_face: FrontFace::Ccw
// topology: TriangleList =======> topology: PrimitiveTopology::TriangleList
// polygon_mode: Fill  ======> polygon_mode: PolygonMode::Fill
// compare: Always ======> compare: CompareFunction::Always
// fail_op: Keep   =====>   fail_op: StencilOperation::Keep
// pass_op: Keep ======> pass_op: StencilOperation::Keep
// format: Bgra8Unorm =======> format: TextureFormat::Bgra8Unorm
// operation: Add  ======> operation: BlendOperation::Add
// ColorWrites(RED | GREEN | BLUE | ALPHA)   =====> ColorWrites::RED | ColorWrites::GREEN | ColorWrites::BLUE | ColorWrites::ALPHA
// : One   ======> :BlendFactor::One
// : Zero   =====> : BlendFactor::Zero
// BufferUsages(COPY_DST | UNIFORM)   ====> BufferUsages::COPY_DST  | BufferUsages::UNIFORM
// resource: Buffer   ======> resource: BindingResource::Buffer
// size: Some\(([0-9]+)\)   =======> size: Some(NonZeroU64::new($1).unwrap())

fn main(){}