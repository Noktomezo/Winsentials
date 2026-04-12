pub mod disable_cloud_sync;
pub mod disable_dotnet_telemetry;
pub mod disable_input_data_collection;
pub mod disable_inventory_collector;
pub mod disable_location_data_collection;
pub mod disable_powershell_telemetry;
pub mod disable_targeted_advertising;
pub mod disable_telemetry_scheduled_tasks;
pub mod disable_windows_error_reporting;
pub mod disable_windows_telemetry;

use crate::tweaks::Tweak;

use self::disable_cloud_sync::DisableCloudSyncTweak;
use self::disable_dotnet_telemetry::DisableDotnetTelemetryTweak;
use self::disable_input_data_collection::DisableInputDataCollectionTweak;
use self::disable_inventory_collector::DisableInventoryCollectorTweak;
use self::disable_location_data_collection::DisableLocationDataCollectionTweak;
use self::disable_powershell_telemetry::DisablePowershellTelemetryTweak;
use self::disable_targeted_advertising::DisableTargetedAdvertisingTweak;
use self::disable_telemetry_scheduled_tasks::DisableTelemetryScheduledTasksTweak;
use self::disable_windows_error_reporting::DisableWindowsErrorReportingTweak;
use self::disable_windows_telemetry::DisableWindowsTelemetryTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(DisableWindowsTelemetryTweak::new()),
        Box::new(DisableTelemetryScheduledTasksTweak::new()),
        Box::new(DisableCloudSyncTweak::new()),
        Box::new(DisableInputDataCollectionTweak::new()),
        Box::new(DisableInventoryCollectorTweak::new()),
        Box::new(DisableLocationDataCollectionTweak::new()),
        Box::new(DisableTargetedAdvertisingTweak::new()),
        Box::new(DisableDotnetTelemetryTweak::new()),
        Box::new(DisablePowershellTelemetryTweak::new()),
        Box::new(DisableWindowsErrorReportingTweak::new()),
    ]
}
