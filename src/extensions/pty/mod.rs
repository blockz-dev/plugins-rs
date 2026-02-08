use deno_core::{ extension, op2, OpState };


#[op2(fast)]
fn connect1(
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
  pty_js,
  ops = [
    connect1,
  ],
  esm_entry_point = "plugins:pty", 
  esm = [
    dir "src/extensions/pty",
    "plugins:pty" = "internal.js"
  ],
  docs = "Rust Based HTML Scraper", "scraper html from rust"
);

pub fn init() -> deno_core::Extension {
    pty_js::init()
}
