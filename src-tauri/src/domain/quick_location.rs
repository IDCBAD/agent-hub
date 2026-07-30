use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickLocation {
    pub id: String,
    pub name: String,
    pub path: String,
    pub show_in_tray: bool,
    pub sort_order: i64,
    pub last_opened_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuickLocationRequest {
    pub name: String,
    pub path: String,
    pub show_in_tray: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuickLocationRequest {
    pub id: String,
    pub name: String,
    pub show_in_tray: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderQuickLocationsRequest {
    pub ids: Vec<String>,
}
