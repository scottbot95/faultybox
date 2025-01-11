pub(crate) mod app;
mod pages;
mod api_client;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<app::App>::new().render();
}
