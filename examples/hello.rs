use pi_wgpu::{Instance, RequestAdapterOptions};

/// This example shows how to describe the adapter in use.
async fn run() {
    let adapter = {
        let instance = Instance::default();

        instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap()
    };

    log::info!("Adapter: {:?}", adapter.get_info())
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .init();

        pollster::block_on(run());
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        console_log::init().expect("could not initialize logger");

        wasm_bindgen_futures::spawn_local(run());
    }
}
