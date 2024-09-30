use api_resource_derive::ApiResource;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct ObjectResource {
    #[api(read_only)]
    pub id: i32,
    #[api(list_rename = "Name")]
    pub name: String,
    #[api(list_rename = "Namespace")]
    pub namespace_id: i32,
    #[api(list_rename = "Class")]
    pub hubuum_class_id: i32,
    #[api(list_rename = "Description")]
    pub description: String,
    #[api(optional, list_rename = "Data")]
    pub data: serde_json::Value,
    #[api(read_only, list_rename = "Created")]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only, list_rename = "Updated")]
    pub updated_at: chrono::NaiveDateTime,
}
