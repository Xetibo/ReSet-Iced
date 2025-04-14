use std::any::Any;

use iced::{
    widget::{column, text},
    Element, Task,
};

#[no_mangle]
pub extern "C" fn enter() {
    println!("frontend startup called");
}

#[no_mangle]
pub extern "C" fn leave() {
    println!("frontend shutdown called");
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn model(
    ctx: &zbus::Connection,
    additional_data: &mut dyn std::any::Any,
) -> &'static mut dyn std::any::Any {
    println!("frontend data called");
    let model = Box::<Vec<i32>>::leak(Box::new(Vec::new()));
    model as &mut dyn Any
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(
    data: &&mut dyn std::any::Any,
    msg: &dyn std::any::Any,
) -> Option<&'static dyn std::any::Any> {
    println!("frontend data called");
    None
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn view(data: &dyn std::any::Any) -> Element<&'static mut dyn Any> {
    println!("frontend data called");
    column!(text("Henlo")).into()
}
