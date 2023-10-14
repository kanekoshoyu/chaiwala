use serde::{Deserialize, Serialize};
/// Core runtime status
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Deserialize, Serialize)]
pub enum RuntimeStatus {
    #[default]
    Idle,
    Running,
}
