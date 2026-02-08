use deno_core::OpState;
use yawc::FrameView;



#[crate::op2(async)]
async fn connect(
  state: &mut OpState,
  #[string] url: String,
) -> std::io::Result<()> {
    state.put(Websocket::new(&url).await.unwrap());
    Ok(())
}

struct Websocket {
  client : yawc::WebSocket
}

impl Websocket {
  pub async fn new(url: &str) -> crate::Result<Self> {
    Ok(Websocket {
      client : yawc::WebSocket::connect(url::Url::parse(url)?).await?
    })
  }

  pub async fn send(&mut self, message: String) -> crate::Result<()> {
    self.client.send(FrameView::text(message)).await?;
    Ok(())
  }
}

crate::extension!(
  media_js,
  ops = [
    connect,
],
  esm_entry_point = "ext:media/internal.js",
  esm = [ dir "src/media", "internal.js" ],
  docs = "Rust Based HTML Scraper", "scraper html from rust"
);

pub fn init() -> deno_core::Extension {
    media_js::init()
}
