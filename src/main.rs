use live_table::App;



fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));

    yew::start_app::<App>();
}
