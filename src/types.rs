#[derive(serde::Serialize)]
pub struct Output {
    pub data: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub identifier: String,
    pub version: String,
    pub entry: String,

    #[serde(skip)]
    url: Option<deno_core::url::Url>,
    #[serde(skip)]
    loaded: Option<bool>,
    #[serde(skip)]
    embed: Option<bool>,
}

impl Plugin {
    
    pub fn set_url(&mut self, url: deno_core::url::Url) {
        self.url = Some(url);
    }

    pub fn set_embed(&mut self, embed: bool) {
        self.embed = Some(embed);
    }

    pub fn set_loaded(&mut self, loaded: bool) {
        self.loaded = Some(loaded);
    }

    #[allow(unused)]
    pub fn url(&self) -> deno_core::url::Url {
        self.url.clone().unwrap()
    }

    #[allow(unused)]
    pub fn embed(&self) -> bool {
        self.embed.unwrap()
    }

    #[allow(unused)]
    pub fn loaded(&self) -> bool {
        self.loaded.unwrap()
    }

}