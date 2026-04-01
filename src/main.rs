#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use crate::{application::Application, komo::start_listen_for_workspaces};

mod application;
mod komo;
mod msgs;
mod utils;

fn begin_execution() -> anyhow::Result<()> {
    let mut app = Application::new()?;
    app.prepare()?;

    let hwnd = unsafe { app.get_primary_hwnd()?.raw_copy() };
    start_listen_for_workspaces(hwnd)?;

    app.run_loop()
}

fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    {
        unsafe { std::env::set_var("RUST_LOG", "trace") };
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
    env_logger::builder()
        .format_timestamp(None)
        .format_file(true)
        .format_line_number(true)
        .init();

    begin_execution().unwrap_or_else(|err| {
        println!("{:#?}", err.backtrace());
        log::error!("Application error: {}", err);
    });

    log::info!("Application exiting normally");

    Ok(())
}
