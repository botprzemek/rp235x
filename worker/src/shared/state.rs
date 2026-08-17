// use crate::shared::channels::SYSTEM_STATE;

// #[repr(u8)]
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub enum SystemState {
//     PowerOn = 0,
//     NetworkBoot = 1,
//     Running = 2,
//     Fault = 255,
// }

// pub fn set_state(state: SystemState) {
//     let _ = SYSTEM_STATE.try_send(state);
// }

// pub async fn get_state() -> SystemState {
//     SYSTEM_STATE.receive().await
// }
