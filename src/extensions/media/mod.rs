use crate::types::Output;





use deno_core::v8;
use deno_core::{serde_v8, v8::Local};
use deno_core::{ extension, op2, OpState };

use ffbins_rs::{Binary, FFbins, InstallProgress, Options, Versions};




#[op2(fast)]
fn ffmpeg_instance(
  state: &mut OpState,
) -> std::io::Result<()> {
    state.put(FFbins::new(Options {
      binary: Binary::FFmpeg,
      version: Versions::V7_1,
      dest: "/usr/local/bin/".into(),
      temp: "/tmp/".into(),
    }));
    Ok(())
}


#[op2]
#[serde]
fn check(
  state: &mut OpState,
) -> std::io::Result<Output> {
    let mut ffmpeg: FFbins = state.take();
    let data = ffmpeg.check();
    Ok(Output { data: data.to_string() })
}


#[op2(reentrant)]
fn download<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  //state: &mut OpState,
  #[serde] cb: serde_v8::Value,
) -> () {
  let cb = to_v8_fn(scope, cb).unwrap();
  let ocb = cb.open(scope);

  let empty: Local<v8::String> = to_local(scope, "").into();
  let context = v8::Context::new(scope, Default::default());
  let scope = &mut v8::ContextScope::new(scope, context);

  let args = &[
    to_local(scope, "&data.name").into(),
  ];

  ocb.call(scope, empty.into(), args).unwrap();
}



#[allow(unused)]
fn ffbins_callback(cb: fn(InstallProgress)) -> () {
  let mut fbins = FFbins::new(Options {
    binary: Binary::FFmpeg,
    version: Versions::V7_1,
    dest: "/usr/local/bin/".into(),
    temp: "/tmp/".into(),
  });

  fbins.init().unwrap();

  fbins.install(&mut move |data| cb(data)).unwrap();

}








#[op2(reentrant)]
fn download2<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  _state: &mut OpState,
  #[serde] cb: serde_v8::Value,
) -> () {
  let cb = to_v8_fn(scope, cb).unwrap();
  let ocb = cb.open(scope);

  let empty: Local<v8::String> = to_local(scope, "").into();

  let args = &[
    to_local(scope, "'Hello' + ' World!'").into(),
  ];

  ocb.call(scope, empty.into(), args).unwrap();
}



fn to_local<'s>(scope: &mut v8::PinScope<'s, '_>, s: &str) -> Local<'s, v8::String> {
  v8::String::new(scope, s).unwrap()
}

fn to_v8_fn(
  scope: &mut v8::Isolate,
  value: serde_v8::Value,
) -> crate::Result<v8::Global<v8::Function>> {
  v8::Local::<v8::Function>::try_from(value.v8_value)
    .map(|cb| v8::Global::new(scope, cb))
    .map_err(|err| crate::Error::Unknown(err.to_string()))
}

#[inline]
#[allow(unused)]
fn to_v8_local_fn(
  value: serde_v8::Value,
) -> crate::Result<v8::Local<v8::Function>> {
  v8::Local::<v8::Function>::try_from(value.v8_value)
    .map_err(|err| crate::Error::Unknown(err.to_string()))
}



/*
#[op2]
#[serde]
fn command(
  state: &mut OpState,
  #[string] arg: String,
) -> std::io::Result<()> {
    let mut ffmpeg: FFbins = state.take();
    ffmpeg.command(&arg, move |data| {

      println!("{}", data);

    }).unwrap();
    Ok(())
}


#[op2]
#[serde]
fn command_with_args(
  state: &mut OpState,
  #[string] args: String,
) -> std::io::Result<()> {
    let mut ffmpeg: FFbins = state.take();
    ffmpeg.command_with_args(args.split(" ").map(|s| s.to_string()).collect(), move |data| {

      println!("{}", data);

    }).unwrap();
    Ok(())
}
*/



extension!(
  media_js,
  ops = [
    ffmpeg_instance,
    check,
    download,
    download2,
    //command,
    //command_with_args,
  ],
  esm_entry_point = "plugins:media", 
  esm = [
    dir "src/extensions/media",
    "plugins:media" = "internal.js"
  ],
  docs = "Rust Based HTML Scraper", "scraper html from rust"
);





pub fn init() -> deno_core::Extension {
    media_js::init()
}
