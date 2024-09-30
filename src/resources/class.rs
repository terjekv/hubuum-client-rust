use api_resource_derive::ApiResource;

use super::Namespace;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct ClassResource {
    #[api(read_only)]
    pub id: i32,
    #[api(list_rename = "Name")]
    pub name: String,
    #[api(list_rename = "Description")]
    pub description: String,
    #[api(as_id, list_rename = "Namespace")]
    pub namespace: Namespace,
    #[api(optional, list_rename = "Schema")]
    pub json_schema: serde_json::Value,
    #[api(optional, list_rename = "Validate")]
    pub validate_schema: bool,
    #[api(read_only, list_rename = "Created")]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only, list_rename = "Updated")]
    pub updated_at: chrono::NaiveDateTime,
}

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct ClassRelationResource {
    #[api(read_only)]
    pub id: i32,
    #[api(list_rename = "FromClass")]
    pub from_class_id: i32,
    #[api(list_rename = "ToClass")]
    pub to_class_id: i32,
    #[api(read_only, list_rename = "Created")]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only, list_rename = "Updated")]
    pub updated_at: chrono::NaiveDateTime,
}
