use std::env;
use std::ffi::OsString;

#[cfg(target_os = "windows")]
use windows_service::service_dispatcher;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;

mod service;
mod ping;
mod ip_utils;
mod cloudflare_api;
mod ip_storage;
mod config;
mod logging;

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init_logger()?;
    config::init_global_config();
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "install" => service::install_service()?,
            "uninstall" => service::uninstall_service()?,
            _ => println!("Unknown command. Use 'install' or 'uninstall'."),
        }
    } else if env::var("RUNNING_AS_SERVICE").is_ok() {
        let _=service_dispatcher::start(service::SERVICE_NAME, service_main);      
    } else {
        tokio::runtime::Runtime::new()?.block_on(run_optimization_loop())?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Runtime::new()?.block_on(run_optimization_loop())?;
    Ok(())
}

async fn run_optimization_loop() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Starting optimization cycle...");
        let starttime=std::time::Instant::now();
        if let Err(e) = service::run_optimization().await {
            log::error!("Error during optimization: {:?}", e);
            eprintln!("Error during optimization: {:?}", e);
        }
        let elapsed=starttime.elapsed();
        println!("Optimization cycle completed in {} seconds. Waiting for next cycle...", elapsed.as_secs());
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}


extern "system" fn service_main(argc: u32, argv: *mut *mut u16) {
    let arguments = unsafe {
        std::slice::from_raw_parts(argv, argc as usize)
            .iter()
            .map(|arg| {
                OsString::from_wide(std::slice::from_raw_parts(
                    *arg,
                    libc::wcslen(*arg),
                ))
            })
            .collect::<Vec<OsString>>()
    };

    if let Err(e) = service::run_service(arguments) {
        eprintln!("Service error: {:?}", e);
    }
}