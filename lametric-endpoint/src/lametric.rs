use serde::Serialize;

pub const LM_ICON_TISSUE: usize = 46254;
pub const LM_ICON_SPERM: usize = 46255;
pub const LM_ICON_CLOCK: usize = 82;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaMetricFrame {
    pub text: String,
    pub icon: Option<usize>,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaMetricResponse {
    pub frames: Vec<LaMetricFrame>,
}
