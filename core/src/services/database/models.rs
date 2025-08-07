use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProject {
    pub id: String,
    pub group_id: String,
    pub name: String,
    pub relative_path: String,
    pub date: String,
    pub project_type: String,
    pub language: String,
}