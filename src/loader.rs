use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic;
use std::path::PathBuf;
use std::rc::Rc;

use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceMapOption;
use deno_core::ModuleLoadOptions;
use deno_core::ModuleLoadReferrer;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceCode;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;
use deno_core::error::ModuleLoaderError;
use deno_core::resolve_import;
use deno_error::JsErrorBox;

use include_dir::Dir;




pub type SourceMapStore = Rc<RefCell<HashMap<String, Vec<u8>>>>;

pub struct PluginLoader {
    source_maps: SourceMapStore,
    embed_dir: Option<Dir<'static>>,
    preload_dir: Option<Dir<'static>>,
}



impl PluginLoader {
    pub fn new(embed_dir: Option<Dir<'static>>, preload_dir: Option<Dir<'static>>) -> Self {
        Self {
            embed_dir,
            preload_dir,
            source_maps: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}



impl ModuleLoader for PluginLoader {

    fn resolve(&self, specifier: &str, referrer: &str, _kind: ResolutionKind) -> Result<ModuleSpecifier, ModuleLoaderError> {
        resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)
    }

    fn load(&self, module_specifier: &ModuleSpecifier, _maybe_referrer: Option<&ModuleLoadReferrer>, _options: ModuleLoadOptions) -> ModuleLoadResponse {

        let source_maps = self.source_maps.clone();

        fn load(
            source_maps: SourceMapStore,
            module_specifier: &ModuleSpecifier,
            embeded: Option<Dir<'static>>,
            preload: Option<Dir<'static>>,
        ) -> Result<ModuleSource, ModuleLoaderError> {

            let (content, media_type) = match module_specifier.scheme() {
                "static" => {
                    let path = PathBuf::from(module_specifier.path().replace("/\\", "\\"));
                    let media_type = MediaType::from_path(path.as_path());
                    (std::fs::read_to_string(path.as_path()).map_err(JsErrorBox::from_err)?, media_type)
                },
                "archive" => {
                    let path = PathBuf::from(module_specifier.path().replace("/\\", "\\"));
                    let content = crate::archive::Archives::find_file(
                        path.clone(), 
                        module_specifier.username()
                    ).map_err(|e| JsErrorBox::generic(e.to_string()))?;
                    let media_type = MediaType::from_path(path.join(module_specifier.username()).as_path());
                    (content, media_type)
                },
                "embeded" => {
                    match module_specifier.username() {
                        "preload" => {
                            let entry = PathBuf::from(module_specifier.as_str().replace("embeded://preload@", ""));
                            let media_type = MediaType::from_path(&entry);
                            (preload.unwrap().get_file(entry).unwrap().contents_utf8().unwrap().to_string(), media_type)
                        }
                        "embeded" => {
                            let entry = PathBuf::from(module_specifier.as_str().replace("embeded://embeded@", ""));
                            let media_type = MediaType::from_path(&entry);
                            (embeded.unwrap().get_file(entry).unwrap().contents_utf8().unwrap().to_string(), media_type)
                        }
                        &_ => panic!("Unknown scheme {}", module_specifier.scheme()),
                    }
                },
                &_ => panic!("Unknown scheme {}", module_specifier.scheme()),
            };

            let (module_type, should_transpile) = match media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (ModuleType::JavaScript, false),
                MediaType::Jsx => (ModuleType::JavaScript, true),
                MediaType::TypeScript => (ModuleType::JavaScript, true),
                MediaType::Tsx => (ModuleType::JavaScript, true),
                MediaType::Json => (ModuleType::Json, false),
                _ => {
                    return Err(JsErrorBox::generic(format!("Unknown extension")));
                }
            };

            let code = transpile(content, source_maps, module_specifier, media_type, should_transpile).unwrap();

            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                module_specifier,
                None,
            ))

        }

        ModuleLoadResponse::Sync(load(source_maps, module_specifier, self.embed_dir.clone(), self.preload_dir.clone()))

    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self
            .source_maps
            .borrow()
            .get(specifier)
            .map(|v| v.clone().into())
    }

}


fn transpile(code: String, source_maps: SourceMapStore, module_specifier: &ModuleSpecifier, media_type: MediaType, should_transpile: bool) -> crate::Result<String> {
  if should_transpile {
    let parsed = deno_ast::parse_module(ParseParams {
          specifier: module_specifier.clone(),
          text: code.into(),
          media_type,
          capture_tokens: false,
          scope_analysis: false,
          maybe_syntax: None,
    }).map_err(JsErrorBox::from_err)?;

   let res = parsed
      .transpile(
        &deno_ast::TranspileOptions {
          imports_not_used_as_values: deno_ast::ImportsNotUsedAsValues::Remove,
          //use_decorators_proposal: true,
          ..Default::default()
        },
        &deno_ast::TranspileModuleOptions { module_kind: None },
        &deno_ast::EmitOptions {
          source_map: SourceMapOption::Separate,
          inline_sources: true,
          ..Default::default()
        },
    ).map_err(JsErrorBox::from_err)?;

    let res = res.into_source();
    let source_map = res.source_map.unwrap().into_bytes();

    source_maps.borrow_mut().insert(module_specifier.to_string(), source_map);

    Ok(res.text)
  } else {
    Ok(code)
  }
}

