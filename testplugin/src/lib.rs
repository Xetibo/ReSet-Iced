use std::any::Any;

use iced::{
    widget::{column, text},
    Element, Task,
};
use re_set_lib::utils::{any::ReSetAny, error::ReSetError};
use zbus::{proxy::SignalStream, Connection};

#[no_mangle]
pub extern "C" fn enter() -> Task<&'static mut dyn ReSetAny> {
    println!("frontend startup called");
    Task::none()
}

#[no_mangle]
pub extern "C" fn leave() -> Task<&'static mut dyn ReSetAny> {
    println!("frontend shutdown called");
    Task::none()
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn signals(conn: Connection) -> Option<SignalStream<'static>> {
    None
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn watch_signals(
    signals: &mut SignalStream<'static>,
    sender: &mut dyn ReSetAny,
) -> Result<(), ReSetError> {
    Ok(())
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn model(
    ctx: &zbus::Connection,
    additional_data: &mut dyn ReSetAny,
) -> &'static mut dyn ReSetAny {
    println!("frontend data called");
    let model = Box::<Vec<i32>>::leak(Box::new(Vec::new()));
    model as &mut dyn ReSetAny
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(
    data: &&mut dyn ReSetAny,
    msg: &dyn ReSetAny,
) -> Option<&'static dyn ReSetAny> {
    println!("frontend data called");
    None
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn view(
    data: &dyn ReSetAny,
) -> Result<Element<&'static mut dyn ReSetAny>, ReSetError> {
    println!("frontend data called");
    Ok(column!(text("Henlo")).into())
}
