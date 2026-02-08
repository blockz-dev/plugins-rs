mod archive;

mod core;
mod extensions;

mod errors;
mod types;
mod loader;

pub use errors::{Error, Result};

pub use ansi_term;

use std::collections::HashMap;
use std::rc::Rc;
use std::path::PathBuf;

use std::sync::Arc;
use std::sync::Mutex;

use include_dir::Dir;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use deno_core::{
    JsRuntime,
    RuntimeOptions,
    PollEventLoopOptions
};

use deno_core::{ serde_v8, v8 };






#[derive(Clone)]
pub enum PluginType {
    Module,
    Archive,
}

impl PluginType {
    
    pub fn is_archive(&self) -> bool {
        match self {
            PluginType::Archive => true,
            _ => false,
        }
    }

    pub fn typ(&self) -> &str {
        match self {
            PluginType::Archive => ".tar.xz",
            _ => "",
        }
    }

}



#[derive(Clone)]
pub struct Options {
    /// #### Static Plugins dir for Runtime loading
    pub plugins: Option<PathBuf>,
    /// #### Plugin Type
    pub plugin_type: PluginType,
    /// #### Preload Module dir (will be loaded before all plugins)
    pub preload: Option<Dir<'static>>,
    /// #### Embeded plugins dir
    pub embeded: Option<Dir<'static>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            plugins: None,
            plugin_type: PluginType::Module,
            preload: None,
            embeded: None,
        }
    }
}









pub struct PluginSystem{
    options: Options,
    pub runtime: JsRuntime,
    plugins: Arc<Mutex<HashMap<String, types::Plugin>>>,
}

impl PluginSystem {

    pub fn new(options: Options, extensions: Option<Vec<deno_core::Extension>>) -> Self {

        #[cfg(target_os = "windows")]
        core::enable_ansi();

        let mut default_extensions = vec![
            crate::core::init(),
            #[cfg(feature = "capture")]
            crate::extensions::capture::init(),
            #[cfg(feature = "media")]
            crate::extensions::media::init(),
            #[cfg(feature = "pty")]
            crate::extensions::pty::init(),
            #[cfg(feature = "scrape")]
            crate::extensions::scrape::init(),
        ];

        if let Some(extensions) = extensions {
            default_extensions.extend(extensions);
        }

        Self {
            options: options.clone(),
            plugins: Arc::new(Mutex::new(HashMap::new())),
            runtime: JsRuntime::new(RuntimeOptions {
                module_loader: Some(Rc::new(loader::PluginLoader::new(options.embeded, options.preload))),
                is_main: true,
                inspector: true,
                startup_snapshot: None,
                extensions: default_extensions,
                ..Default::default()
            }),
        }
    }

    fn basename(&self) -> String {
        String::from("plugin.json")
    }

    //
    //
    //

    fn url_archive(&self, pdir: &str, a: &str) -> crate::Result<(deno_core::url::Url, types::Plugin)> {
        let base = PathBuf::from(pdir);
        let file = match archive::Archives::find_file(base.join(a), self.basename().as_str()) {
            Ok(content) => {
                let plugin: types::Plugin = serde_json::from_str(&content).unwrap();
                Ok(plugin)
            },
            Err(_) => Err(Error::Unknown("ARCHIVE: plugin.json not found.".to_string())),
        }?;
        let name = file.entry.clone();
        let uri = match archive::Archives::exists(base.join(a), &name) {
            Ok(_) => {
                let pname = PathBuf::from(format!("archive://{}", base.join(a).to_str().unwrap()));
                let mut base = deno_core::url::Url::parse(pname.to_str().unwrap()).unwrap();
                base.set_username(&name).unwrap();
                Ok(base)
            },
            Err(e) => Err(Error::Unknown(format!("ARCHIVE: {}", e.to_string()))),
        }?;
        Ok((uri, file))
    }

    fn url_static(&self, pdir: &str) -> crate::Result<(deno_core::url::Url, types::Plugin)> {
        let base = PathBuf::from(pdir);
        let file = match std::fs::read_to_string(&base.join(self.basename())) {
            Ok(content) => {
                let plugin: types::Plugin = serde_json::from_str(&content).unwrap();
                Ok(plugin)
            },
            Err(_) => Err(Error::Unknown("STATIC: plugin.json not found.".to_string())),
        }?;
        let name = file.entry.clone();
        let uri = match std::fs::metadata(&base.join(&name)) {
            Ok(_) => {
                let name = PathBuf::from(format!("static://{}", base.join(name).to_str().unwrap()));
                let base = deno_core::url::Url::parse(name.to_str().unwrap()).unwrap();
                Ok(base)
            },
            Err(e) => Err(Error::Unknown(e.to_string())),
        }?;
        Ok((uri, file))
    }


    fn url_embed(&self, side: bool, pdir: Option<&str>) -> crate::Result<(deno_core::url::Url, types::Plugin)> {
        let preload = if side {
            self.options.embeded.clone().unwrap()
        } else {
            self.options.preload.clone().unwrap()
        };
        let basename = PathBuf::from(pdir.unwrap_or("")).join(self.basename());
        let (filename, p) = match preload.contains(&basename) {
            true => {
                let content = preload.get_file(basename).unwrap().contents();
                let plugin: types::Plugin = serde_json::from_slice(content)?;
                let entry = PathBuf::from(pdir.unwrap_or("")).join(&plugin.entry);
                Ok((match preload.contains(&entry) {
                    true => {
                        let content = preload.get_file(entry).unwrap().path();
                        Ok(content)
                    },
                    false => Err(Error::Unknown("entry file not found.".to_string()))
                }?, plugin))
            },
            false => Err(Error::Unknown(format!("{} not found.", basename.to_str().unwrap()))),
        }?;
        let mut base = deno_core::url::Url::parse(format!("embeded://{}", filename.to_str().unwrap()).as_str()).unwrap();
        base.set_username(if side { "embeded" } else { "preload" }).unwrap();
        Ok((base, p))
    }

    //
    //
    //

    async fn preload(&mut self) -> Result<()> {
        if self.options.preload.is_some() {
            let (url, mut file) = self.url_embed(false, None)?;
            let mod_id = self.runtime.load_main_es_module(&url).await?;
            let result = self.runtime.mod_evaluate(mod_id);
            result.await?;
            file.set_loaded(true);
        }
        Ok(())
    }

    async fn load_embed(&mut self) -> Result<()> {
        if self.options.embeded.is_some() {
            let mut pguard = self.plugins.lock().unwrap();
            for entry in self.options.embeded.clone().unwrap().entries() {
                let (url, mut file) = self.url_embed(true, Some(entry.path().to_str().unwrap()))?;
                let mod_id = self.runtime.load_side_es_module(&url).await?;
                let result = self.runtime.mod_evaluate(mod_id);
                result.await?;
                file.set_loaded(true);
                file.set_url(url);
                file.set_embed(true);
                pguard.insert(file.identifier.clone(), file);
            }
            drop(pguard);
        }
        Ok(())
    }

    async fn load_archive(&mut self) -> Result<()> {
        if let Some(plugins_dir) = &self.options.plugins {
            let mut pguard = self.plugins.lock().unwrap();
            if !std::fs::metadata(&plugins_dir).is_ok() {
                return Ok(());
            }
            let read = std::fs::read_dir(plugins_dir)?;
            for entry in read {
                let dir = entry?;
                if !dir.metadata().unwrap().is_file() || !dir.file_name().into_string().unwrap().ends_with(PluginType::Archive.typ()) {
                    continue;
                }
                let filename = dir.file_name().into_string().unwrap();
                let fullpath = dir.path().to_str().unwrap().replace(&filename, "");
                let (url, mut file) = self.url_archive(&fullpath, &filename)?;
                let mod_id = self.runtime.load_side_es_module(&url).await?;
                let result = self.runtime.mod_evaluate(mod_id);
                result.await?;
                file.set_loaded(true);
                file.set_url(url);
                file.set_embed(false);
                pguard.insert(file.identifier.clone(), file);
            }
            drop(pguard);
        }
        Ok(())
    }

    async fn load_static(&mut self) -> Result<()> {
        if let Some(plugins_dir) = &self.options.plugins {
            let mut pguard = self.plugins.lock().unwrap();
            if !std::fs::metadata(&plugins_dir).is_ok() {
                return Ok(());
            }
            let read = std::fs::read_dir(plugins_dir)?;
            for entry in read {
                let dir = entry?;
                if dir.metadata().unwrap().is_file() || dir.file_name().into_string().unwrap().ends_with(PluginType::Archive.typ()) {
                    continue;
                }
                let (url, mut file) = self.url_static(dir.path().to_str().unwrap())?;
                let mod_id = self.runtime.load_side_es_module(&url).await?;
                let result = self.runtime.mod_evaluate(mod_id);
                result.await?;
                file.set_loaded(true);
                file.set_url(url);
                file.set_embed(false);
                pguard.insert(file.identifier.clone(), file);
            }
            drop(pguard);
        }
        Ok(())
    }

    //
    //
    //

    async fn init(&mut self) -> Result<()> {

        self.runtime.execute_script("__RUNTIME_API__", include_str!("api/index.js"))?;

        self.preload().await?;
        self.load_embed().await?;

        match self.options.plugin_type {
            PluginType::Module => self.load_static().await?,
            PluginType::Archive => self.load_archive().await?,
        }

        self.runtime.run_event_loop(PollEventLoopOptions::default()).await?;
        
        Ok(())
    }

    //
    //
    //

    pub fn run(mut self) -> crate::Result<PluginSystem> {
        let art = AsyncRuntimeBuilder::new_current_thread().enable_all().build().unwrap();
        art.block_on(self.init())?;
        Ok(self)
    }

    //
    //
    //
    // PUBLIC FUNCTIONS
    //
    //
    //

    pub fn execute(&mut self, namespace: &'static str, plugin: &'static str, key: &'static str) -> crate::Result<serde_json::Value> {
        let code = format!(r#"(window.loadPlugin("{}").{})"#, plugin, key);
        let result = self.runtime.execute_script(namespace, code)?;
        deno_core::scope!(scope, self.runtime);
        let local = v8::Local::new(scope, result);
        Ok(serde_v8::from_v8::<serde_json::Value>(scope, local)?)
    }

    //

    pub fn eval(&mut self, namespace: &'static str, message: &'static str) -> crate::Result<serde_json::Value> {
        let result = self.runtime.execute_script(namespace, message)?;
        deno_core::scope!(scope, self.runtime);
        let local = v8::Local::new(scope, result);
        Ok(serde_v8::from_v8::<serde_json::Value>(scope, local)?)
    }
    
    //

    pub fn send(&mut self, namespace: &'static str, message: &'static str) -> crate::Result<String> {
        let result = self.runtime.execute_script(namespace, message)?;
        deno_core::scope!(scope, self.runtime);
        let local = v8::Local::new(scope, result);
        Ok(serde_v8::from_v8::<serde_json::Value>(scope, local)?.to_string())
    }
    
    //

}