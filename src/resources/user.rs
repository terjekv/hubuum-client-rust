use api_resource_derive::ApiResource;
#[allow(dead_code)]
#[derive(ApiResource)]
pub struct UserResource {
    #[api(read_only)]
    pub id: i32,
    pub username: String,
    #[api(post_only, read_only)]
    pub password: String,
    #[api(optional)]
    pub email: String,
    #[api(read_only)]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only)]
    pub updated_at: chrono::NaiveDateTime,
}
