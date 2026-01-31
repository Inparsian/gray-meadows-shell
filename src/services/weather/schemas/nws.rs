// ! This is not comprehensive; only the fields we care about are included.
// ! If more fields need to be added, they'll be added here.
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NwsAlertStatus {
    Actual,
    Exercise,
    System,
    Test,
    Draft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NwsAlertMessageType {
    Alert,
    Update,
    Cancel,
    Ack,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NwsAlertSeverity {
    Extreme,
    Severe,
    Moderate,
    Minor,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NwsAlertCertainty {
    Observed,
    Likely,
    Possible,
    Unlikely,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NwsAlertUrgency {
    Immediate,
    Expected,
    Future,
    Past,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwsFeatureProperties {
    pub id: String,
    pub sent: String,
    pub effective: String,
    pub onset: Option<String>,
    pub expires: String,
    pub ends: Option<String>,
    pub status: NwsAlertStatus,
    pub message_type: NwsAlertMessageType,
    pub severity: NwsAlertSeverity,
    pub certainty: NwsAlertCertainty,
    pub urgency: NwsAlertUrgency,
    pub event: String,
    pub sender: String,
    pub sender_name: String,
    pub headline: Option<String>,
    pub description: String,
    pub instruction: Option<String>,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NwsFeature {
    pub properties: NwsFeatureProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NwsAlertsResponse {
    pub features: Vec<NwsFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NwsAlertsError {
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub detail: String,
}