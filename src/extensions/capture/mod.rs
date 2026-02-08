use crate::deno_core::{ extension, op2, OpState };



#[op2(fast)]
fn connect(
  state: &mut OpState,
  #[string] _url: String,
) -> std::io::Result<()> {
    state.put(Websocket::new().unwrap());
    Ok(())
}

struct Websocket {}

impl Websocket {
  pub fn new() -> crate::Result<Self> {
    Ok(Websocket {})
  }
}

extension!(
  capture_js,
  ops = [
    connect,
  ],
  esm_entry_point = "plugins:capture", 
  esm = [
    dir "src/extensions/capture",
    "plugins:capture" = "internal.js"
  ],
  docs = "Rust Based HTML Scraper", "scraper html from rust"
);

pub fn init() -> deno_core::Extension {
    capture_js::init()
}
