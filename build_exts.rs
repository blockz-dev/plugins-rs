use deno_core::extension;

extension!(
    core_js, 
    esm_entry_point = "plugins:core", 
    esm = [
        dir "src/core",
        "plugins:core" = "internal.js"
    ]
);

extension!(
    capture_js, 
    esm_entry_point = "plugins:capture", 
    esm = [
        dir "src/extensions/capture",
        "plugins:capture" = "internal.js"
    ]
);

extension!(
    media_js, 
    esm_entry_point = "plugins:media", 
    esm = [
        dir "src/extensions/media",
        "plugins:media" = "internal.js"
    ]
);

extension!(
    pty_js, 
    esm_entry_point = "plugins:pty", 
    esm = [
        dir "src/extensions/pty",
        "plugins:pty" = "internal.js"
    ]
);

extension!(
    scrape_js, 
    esm_entry_point = "plugins:scrape", 
    esm = [
        dir "src/extensions/scrape",
        "plugins:scrape" = "internal.js"
    ]
);