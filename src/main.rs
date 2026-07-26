#![deny(unsafe_op_in_unsafe_fn)]

mod app;
mod automation;
mod settings;

fn main() {
    app::run();
}
