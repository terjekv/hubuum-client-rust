use std::borrow::Cow;

use crate::endpoints::Endpoint;
use crate::QueryFilter;

pub mod r#async;
pub mod sync;

pub use self::r#async::Client as AsyncClient;
pub use self::sync::Client as SyncClient;

use crate::resources::ApiResource;

pub type UrlParams = Vec<(Cow<'static, str>, Cow<'static, str>)>;

trait ClientCore {
    fn build_url(&self, endpoint: &Endpoint, url_params: UrlParams) -> String;
}

pub trait IntoResourceFilter<T: ApiResource> {
    fn into_resource_filter(self) -> Vec<QueryFilter>;
}

#[derive(Debug, Clone)]
pub struct Unauthenticated;

#[derive(Debug, Clone)]
pub struct Authenticated {
    token: String,
}
