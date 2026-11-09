//! UI components for scoreboard, match control, and device management.

pub mod center_view;
pub mod inspector;
pub mod master_modal;
pub mod sidebar;
pub mod wifi_modal;

pub use center_view::{HmiCenterView, MatchControlPanel};
pub use inspector::{DeviceInspector, HmiInspector};
pub use master_modal::MasterServerModal;
pub use sidebar::{HmiSidebar, NavigationSidebar};
pub use wifi_modal::WifiConfigModal;
