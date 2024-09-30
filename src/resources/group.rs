use api_resource_derive::ApiResource;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct GroupResource {
    #[api(read_only)]
    pub id: i32,
    #[api(list_rename = "Name")]
    pub groupname: String,
    #[api(list_rename = "Description")]
    pub description: String,
    #[api(read_only, list_rename = "Created")]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only, list_rename = "Updated")]
    pub updated_at: chrono::NaiveDateTime,
}
