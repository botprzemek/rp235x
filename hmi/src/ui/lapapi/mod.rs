//! UI components for scoreboard, match control, and device management.

pub mod center_view;
pub mod inspector;
pub mod master_modal;
pub mod sidebar;
pub mod wifi_modal;

pub use center_view::{LapapiCenterView, MatchControlPanel};
pub use inspector::{DeviceInspector, LapapiInspector};
pub use master_modal::MasterServerModal;
pub use sidebar::{LapapiSidebar, NavigationSidebar};
pub use wifi_modal::WifiConfigModal;
