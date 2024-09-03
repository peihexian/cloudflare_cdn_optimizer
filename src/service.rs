use crate::ip_utils::parse_cidr_list;
use crate::ping::ping_ips;
use crate::ip_storage::save_top_ips;
use crate::cloudflare_api::update_dns_record;
use crate::config;
use std::time::Duration;
use std::ffi::OsString;
use anyhow::Result;
use tokio::time::sleep;


#[cfg(target_os = "windows")]
use windows_service::{
    service_control_handler::{self, ServiceControlHandlerResult},
    service::{
        ServiceType, ServiceAccess, ServiceErrorControl, ServiceStartType,
        ServiceInfo, ServiceExitCode, ServiceControlAccept, ServiceState,
        ServiceControl, ServiceStatus,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

pub const SERVICE_NAME: &str = "CloudflareCDNOptimizer";
const SERVICE_DISPLAY_NAME: &str = "Cloudflare CDN Optimizer";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const OUTPUT_FILE: &str = "fastest_cdn_ips.txt";


#[cfg(target_os = "windows")]
pub fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                // Implement proper shutdown logic here
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        loop {
            if let Err(e) = run_optimization().await {
                log::error!("Error during optimization: {:?}", e);
            }
            let seconds= config::GLOBAL_CONFIG.optimization.run_interval_seconds;
            sleep(Duration::from_secs(seconds)).await; 
        }
    });

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}


#[cfg(target_os = "windows")]
pub fn install_service() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    //get current application exe filename
    let exe_path = ::std::env::current_exe().unwrap();

    let service_binary_path = ::std::env::current_exe()
        .unwrap()
        .with_file_name(exe_path.file_name().unwrap());

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, 
        account_password: None,
    };

    let _service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    println!("Service installed successfully");
    Ok(())

}


#[cfg(target_os = "windows")]
pub fn uninstall_service() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    service.delete()?;
    println!("Service uninstalled successfully");
    Ok(())
}

pub async fn run_optimization() -> Result<()> {
    let ips = parse_cidr_list(&config::GLOBAL_CONFIG.cdn.cidr_list);

    log::debug!("Total IP addresses: {}", ips.len());

    let ping_results = ping_ips(ips, config::GLOBAL_CONFIG.optimization.ping_threads).await;

    let mut sorted_results = ping_results;
    sorted_results.sort_by_key(|&(_, duration)| duration);
    save_top_ips(&sorted_results, OUTPUT_FILE, config::GLOBAL_CONFIG.optimization.top_ips_to_save)?;

    if config::GLOBAL_CONFIG.cloudflare.update_dns {
        if let Some((fastest_ip, _)) = sorted_results.first() {
            update_dns_record(
                &config::GLOBAL_CONFIG.cloudflare.api_token,
                &config::GLOBAL_CONFIG.cloudflare.zone_id,
                &config::GLOBAL_CONFIG.cloudflare.record_id,
                &config::GLOBAL_CONFIG.cloudflare.domain,
                &fastest_ip.to_string(),
            )
            .await?;
        }
    }

    Ok(())
}

