use std::{fs::create_dir, io::ErrorKind, path::PathBuf, sync::atomic::AtomicBool};

use iced::{Element, Task};
use once_cell::sync::Lazy;
use re_set_lib::{
    create_config_directory,
    utils::{any::ReSetAny, error::ReSetError},
};
use zbus::proxy::SignalStream;

use crate::PluginFuncs;

pub static LIBS_LOADED: AtomicBool = AtomicBool::new(false);
pub static LIBS_LOADING: AtomicBool = AtomicBool::new(false);
static mut LIBS: Vec<libloading::Library> = Vec::new();
pub static mut PLUGIN_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(""));

pub static SETUP_PLUGIN_DIR: fn() -> Option<PathBuf> = || -> Option<PathBuf> {
    let config = create_config_directory("reset").expect("Could not create config directory");
    let plugin_dir = create_dir(config.join("plugins"));
    if let Err(error) = plugin_dir {
        if error.kind() != ErrorKind::AlreadyExists {
            None
        } else {
            Some(config.join("plugins"))
        }
    } else {
        Some(config.join("plugins"))
    }
};

pub static SETUP_LIBS: fn() = || {
    let read_dir: fn(PathBuf) = |dir: PathBuf| {
        let plugin_dir = dir.read_dir();
        if plugin_dir.is_err() {
            // do not print error to ignore the usr/lib if not needed
            return;
        }
        let plugin_dir = plugin_dir.unwrap();
        plugin_dir.for_each(|plugin| {
            if let Ok(file) = plugin {
                unsafe {
                    let path = file.path();
                    let lib = libloading::Library::new(&path);
                    if let Ok(lib) = lib {
                        LIBS.push(lib);
                    } else {
                        // TOOD handle
                    }
                }
            }
        });
    };
    SETUP_PLUGIN_DIR();
    read_dir(PathBuf::from("/home/dashie/.config/reset/plugins"));
};

pub fn load_plugins() -> Vec<PluginFuncs> {
    let mut plugins = Vec::new();
    unsafe {
        for lib in LIBS.iter() {
            let enter: Result<
                libloading::Symbol<unsafe extern "C" fn() -> Task<&'static mut dyn ReSetAny>>,
                libloading::Error,
            > = lib.get(b"enter");
            let leave: Result<
                libloading::Symbol<unsafe extern "C" fn() -> Task<&'static mut dyn ReSetAny>>,
                libloading::Error,
            > = lib.get(b"leave");
            let model: Result<
                libloading::Symbol<
                    unsafe extern "C" fn(
                        ctx: &zbus::Connection,
                        additional_data: &mut dyn ReSetAny,
                    ) -> &'static mut dyn ReSetAny,
                >,
                libloading::Error,
            > = lib.get(b"model");
            let update: Result<
                libloading::Symbol<
                    unsafe extern "C" fn(
                        data: &&mut dyn ReSetAny,
                        msg: &dyn ReSetAny,
                    ) -> Option<&'static dyn ReSetAny>,
                >,
                libloading::Error,
            > = lib.get(b"update");
            let view: Result<
                libloading::Symbol<
                    unsafe extern "C" fn(
                        data: &dyn ReSetAny,
                    )
                        -> Result<Element<&'static mut dyn ReSetAny>, ReSetError>,
                >,
                libloading::Error,
            > = lib.get(b"view");
            let signals: Result<
                libloading::Symbol<
                    unsafe extern "C" fn(conn: &zbus::Connection) -> Option<SignalStream<'static>>,
                >,
                libloading::Error,
            > = lib.get(b"signals");
            let watch_signals: Result<
                libloading::Symbol<
                    unsafe extern "C" fn(
                        sender: &mut dyn ReSetAny,
                        signals: &mut SignalStream<'static>,
                    ) -> Result<(), ReSetError>,
                >,
                libloading::Error,
            > = lib.get(b"watch_signals");

            match (enter, leave, model, update, view, signals, watch_signals) {
                (
                    Ok(enter),
                    Ok(leave),
                    Ok(model),
                    Ok(update),
                    Ok(view),
                    Ok(signals),
                    Ok(watch_signals),
                ) => {
                    plugins.push(PluginFuncs {
                        enter,
                        leave,
                        model,
                        update,
                        view,
                        signals,
                        watch_signals,
                    });
                }
                (enter, leave, model, update, view, signals, watch_signals) => {
                    dbg!(enter, leave, model, update, view, signals, watch_signals);
                    panic!("plugin could not be loaded")
                }
            }
        }
    }
    plugins
}
