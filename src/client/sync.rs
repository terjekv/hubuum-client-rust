use log::{error, trace};
use reqwest::blocking::Response;
use serde_json::Value;
use serde_urlencoded;
use std::marker::PhantomData;

use super::{Authenticated, ClientCore, IntoResourceFilter, Unauthenticated};
use crate::endpoints::Endpoint;
use crate::errors::ApiError;
use crate::resources::{ApiResource, Class, Group, Namespace, User};
use crate::types::{BaseUrl, Credentials, FilterOperator, Token};

#[derive(Debug, Clone)]
pub struct Client<S> {
    http_client: reqwest::blocking::Client,
    base_url: BaseUrl,
    state: S,
}

impl<S> ClientCore for Client<S> {
    fn build_url(&self, endpoint: &Endpoint) -> String {
        format!(
            "{}{}",
            self.base_url.with_trailing_slash(),
            endpoint.trim_start_matches('/')
        )
    }
}

trait ResponseHandler {
    fn check_success(&self, response: Response) -> Result<Response, ApiError>;
}

impl<T> ResponseHandler for Client<T> {
    fn check_success(&self, response: Response) -> Result<Response, ApiError> {
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text()?;
            let error_message = match serde_json::from_str::<Value>(&body) {
                Ok(json) => json["message"]
                    .as_str()
                    .unwrap_or("Error without message.")
                    .to_string(),
                Err(_) => body,
            };
            return Err(ApiError::HttpWithBody {
                status,
                message: error_message,
            });
        }
        Ok(response)
    }
}

impl Client<Unauthenticated> {
    pub fn new(base_url: BaseUrl) -> Self {
        Client {
            http_client: reqwest::blocking::Client::new(),
            base_url,
            state: Unauthenticated,
        }
    }
}

impl Client<Unauthenticated> {
    pub fn login(self, credentials: Credentials) -> Result<Client<Authenticated>, ApiError> {
        let token: Token = self
            .http_client
            .post(&self.build_url(&Endpoint::Login))
            .json(&credentials)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(Client {
            http_client: self.http_client,
            base_url: self.base_url,
            state: Authenticated { token: token.token },
        })
    }

    pub fn login_with_token(self, token: Token) -> Result<Client<Authenticated>, ApiError> {
        let status = self
            .http_client
            .get(self.build_url(&Endpoint::LoginWithToken))
            .header("Authorization", format!("Bearer {}", token.token))
            .send()?;

        if status.status().is_success() {
            Ok(Client {
                http_client: self.http_client,
                base_url: self.base_url,
                state: Authenticated { token: token.token },
            })
        } else {
            Err(ApiError::InvalidToken)
        }
    }
}

impl Client<Authenticated> {
    pub fn get_token(&self) -> &str {
        &self.state.token
    }

    pub fn get<R: ApiResource>(
        &self,
        resource: R,
        params: R::GetParams,
    ) -> Result<Vec<R::GetOutput>, ApiError> {
        // Change the return type to Vec
        let endpoint = resource.endpoint();
        let url = self.build_url(&endpoint);

        let query = serde_urlencoded::to_string(&params)?;
        let url = if !query.is_empty() {
            format!("{}?{}", url, query)
        } else {
            url
        };

        trace!("GET {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .send()?;

        let response = self.check_success(response)?;

        trace!("Response: {:?}", response);

        let response_text = response.text()?;
        let obj: Vec<R::GetOutput> = match serde_json::from_str(&response_text) {
            Ok(obj) => obj,
            Err(err) => {
                error!(
                    "Failed to deserialize response: {}\nResponse text: {}",
                    err, response_text
                );
                return Err(ApiError::DeserializationError(response_text));
            }
        };

        Ok(obj)
    }

    pub fn post<R: ApiResource>(
        &self,
        resource: R,
        params: R::PostParams,
    ) -> Result<R::PostOutput, ApiError> {
        let endpoint = resource.endpoint();
        let url = self.build_url(&endpoint);

        trace!("POST {} with {:?}", &url, params);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .json(&params)
            .send()?;

        let response = self.check_success(response)?;

        trace!("Response: {:?}", response);
        let response_text = response.text()?;
        let obj: R::PostOutput = match serde_json::from_str(&response_text) {
            Ok(obj) => obj,
            Err(err) => {
                error!(
                    "Failed to deserialize response: {}\nResponse text: {}",
                    err, response_text
                );
                return Err(ApiError::DeserializationError(response_text));
            }
        };

        Ok(obj)
    }

    pub fn patch<R: ApiResource>(
        &self,
        resource: R,
        id: i32,
        params: R::PatchParams,
    ) -> Result<R::PatchOutput, ApiError> {
        let endpoint = resource.endpoint();
        let url = format!("{}{}", self.build_url(&endpoint), id);

        trace!("PATCH {} with {:?}", &url, params);

        let response = self
            .http_client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .json(&params)
            .send()?;

        trace!("Response: {:?}", response);
        let response_text = response.text()?;
        let obj: R::PatchOutput = match serde_json::from_str(&response_text) {
            Ok(obj) => obj,
            Err(err) => {
                error!(
                    "Failed to deserialize response: {}\nResponse text: {}",
                    err, response_text
                );
                return Err(ApiError::DeserializationError(response_text));
            }
        };
        Ok(obj)
    }

    pub fn delete<R: ApiResource>(&self, resource: R, id: i32) -> Result<(), ApiError> {
        let endpoint = resource.endpoint();
        let url = format!("{}{}", self.build_url(&endpoint), id);

        trace!("DELETE {}", &url);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .send()?;

        let response = self.check_success(response)?;

        trace!("Response: {:?}", response);
        Ok(())
    }

    pub fn users(&self) -> Resource<User> {
        Resource::new(self.clone())
    }

    pub fn classes(&self) -> Resource<Class> {
        Resource::new(self.clone())
    }

    pub fn namespaces(&self) -> Resource<Namespace> {
        Resource::new(self.clone())
    }

    pub fn groups(&self) -> Resource<Group> {
        Resource::new(self.clone())
    }
}

pub struct FilterBuilder<T: ApiResource> {
    client: Client<Authenticated>,
    filters: Vec<(String, FilterOperator, String)>,
    _phantom: PhantomData<T>,
}

impl<T: ApiResource> FilterBuilder<T> {
    fn new(client: Client<Authenticated>) -> Self {
        FilterBuilder {
            client,
            filters: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn add_filter<V: ToString>(mut self, field: &str, op: FilterOperator, value: V) -> Self {
        self.filters
            .push((field.to_string(), op, value.to_string()));
        self
    }

    pub fn execute(self) -> Result<Vec<T::GetOutput>, ApiError> {
        let params = T::build_params(self.filters);
        let res = self.client.get::<T>(T::default(), params);
        println!("{:?}", res);
        res
    }
}

pub struct Resource<T: ApiResource> {
    client: Client<Authenticated>,
    _phantom: PhantomData<T>,
}

impl<T: ApiResource> Resource<T> {
    fn new(client: Client<Authenticated>) -> Self {
        Resource {
            client,
            _phantom: PhantomData,
        }
    }

    pub fn find(&self) -> FilterBuilder<T> {
        FilterBuilder::new(self.client.clone())
    }
    pub fn filter<F: IntoResourceFilter<T>>(
        &self,
        filter: F,
    ) -> Result<Vec<T::GetOutput>, ApiError> {
        let params = filter.into_resource_filter();
        self.client.get::<T>(T::default(), params)
    }

    pub fn create(&self, params: T::PostParams) -> Result<T::PostOutput, ApiError> {
        self.client.post::<T>(T::default(), params)
    }

    pub fn update(&self, id: i32, params: T::PatchParams) -> Result<T::PatchOutput, ApiError> {
        self.client.patch::<T>(T::default(), id, params)
    }

    pub fn delete(&self, id: i32) -> Result<(), ApiError> {
        self.client.delete::<T>(T::default(), id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use yare::parameterized;

    #[parameterized(
        login_foo = { "https://foo.bar.com", Endpoint::Login },
        get_user_foo = { "https://foo.bar.com", Endpoint::GetUser },
        get_class_foo = { "https://foo.bar.com", Endpoint::GetClass },
        login_bar = { "https://bar.baz.com", Endpoint::Login },
        get_user_bar = { "https://bar.baz.com", Endpoint::GetUser },
        get_class_bar = { "https://bar.baz.com", Endpoint::GetClass }

    )]

    fn test_build_url(server: &str, endpoint: Endpoint) {
        let base_url = BaseUrl::from_str(server).unwrap();
        let client = Client::new(base_url.clone());

        assert_eq!(
            client.build_url(&endpoint),
            format!(
                "{}{}",
                base_url.with_trailing_slash(),
                endpoint.trim_start_matches('/')
            )
        );
    }
}
