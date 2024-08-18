use crate::endpoints::Endpoint;
use crate::resources::ApiResource;
use crate::types::FilterOperator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub namespace_id: i32,
    pub json_schema: serde_json::Value,
    pub validate_schema: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ClassGet {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub namespace_id: Option<i32>,
    pub json_schema: Option<serde_json::Value>,
    pub validate_schema: Option<bool>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ClassPost {
    pub name: String,
    pub description: String,
    pub namespace_id: i32,
    pub json_schema: Option<serde_json::Value>,
    pub validate_schema: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ClassPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub namespace_id: Option<i32>,
    pub json_schema: Option<serde_json::Value>,
    pub validate_schema: Option<bool>,
}

impl ApiResource for Class {
    type GetParams = ClassGet;
    type GetOutput = Class;
    type PostParams = ClassPost;
    type PostOutput = Class;
    type PatchParams = ClassPatch;
    type PatchOutput = Class;
    type DeleteParams = ();
    type DeleteOutput = ();

    fn endpoint(&self) -> Endpoint {
        Endpoint::GetClass
    }

    fn build_params(filters: Vec<(String, FilterOperator, String)>) -> Self::GetParams {
        let mut params = ClassGet::default();

        for (field, op, value) in filters {
            match (field.as_str(), op) {
                ("name", FilterOperator::Eq) => params.name = Some(value),
                ("description", FilterOperator::Eq) => params.description = Some(value),
                _ => {} // Add more cases as needed
            }
        }

        params
    }
}
