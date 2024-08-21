use api_resource_derive::ApiResource;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct GroupResource {
    #[api(read_only)]
    pub id: i32,
    pub groupname: String,
    pub description: String,
    #[api(read_only)]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only)]
    pub updated_at: chrono::NaiveDateTime,
}
