use api_resource_derive::ApiResource;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct ClassResource {
    #[api(read_only)]
    pub id: i32,
    pub name: String,
    pub description: String,
    pub namespace_id: i32,
    #[api(optional)]
    pub json_schema: serde_json::Value,
    #[api(optional)]
    pub validate_schema: bool,
    #[api(read_only)]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only)]
    pub updated_at: chrono::NaiveDateTime,
}
