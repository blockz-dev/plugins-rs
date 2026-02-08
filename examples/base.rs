







fn main() {

    let local = std::env::current_dir().unwrap()
        .join("examples")
        .join("javascript")
        .join("plugins");

    let opt = plugins_rs::Options {
        plugins: Some(local),
        plugin_type: plugins_rs::PluginType::Archive,
        //preload: Some(include_dir::include_dir!("examples/javascript/main")),
        //embeded: Some(include_dir::include_dir!("examples/javascript/embed")),
        ..Default::default()
    };

    let mut prt = match plugins_rs::PluginSystem::new(opt, None).run() {
        Ok(prt) => prt,
        Err(err) => panic!("{}", err),
    };

    loop {

        println!("{:?}", prt.send("namespace", r#"console.log("test")"#).unwrap());

        std::thread::sleep(std::time::Duration::from_millis(1000));

    }

}