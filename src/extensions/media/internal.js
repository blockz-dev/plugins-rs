
export function ffmpeg_instance() {
  Deno.core.ops.ffmpeg_instance();
}

export function check() {
  return Deno.core.ops.check();
}

export function download(dest, temp) {
  Deno.core.ops.download(dest, temp);
}

export function download2(cb) {
  Deno.core.ops.download2(cb);
}

export function command(arg) {
  Deno.core.ops.command(arg);
}

export function command_with_args(args) {
  Deno.core.ops.command_with_args(args);
}
