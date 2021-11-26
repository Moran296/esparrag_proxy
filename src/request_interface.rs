use crate::ServiceManager;
use std::convert::Infallible;
use std::sync::Arc;
use warp::{http, Filter};

/// get active services
async fn get_services() -> Result<impl warp::Reply, Infallible> {
    let services = ServiceManager::get_services_meta().await;
    Ok(warp::reply::json(&services))
}

///post an action for a service
async fn post_action(
    service_name: String,
    action_name: String,
    action: serde_json::Value,
    service_manager: Arc<ServiceManager>,
) -> Result<impl warp::Reply, Infallible> {
    println!("got request service/{}/{}", service_name, action_name);

    let service = service_manager.make_service(&service_name).await;
    if service.is_none() {
        return Ok(warp::reply::with_status(
            format!("service does not exist!"),
            http::StatusCode::BAD_REQUEST,
        ));
    }

    let mut service = service.unwrap();
    let res = service.perform(&action_name, action).await;
    if res.is_err() {
        return Ok(warp::reply::with_status(
            format!("service action failed!. {}", res.unwrap_err().to_string()),
            http::StatusCode::BAD_REQUEST,
        ));
    }

    let val = res.unwrap();
    return Ok(warp::reply::with_status(
        format!("{:?}", val),
        http::StatusCode::OK,
    ));
}

#[derive(Clone)]
pub struct RequestInterface {
    service_manager: Arc<ServiceManager>,
}

impl RequestInterface {
    pub fn new(service_manager: Arc<ServiceManager>) -> Self {
        RequestInterface { service_manager }
    }

    pub async fn run(self) {
        let with_service_manager = warp::any().map(move || self.service_manager.clone());

        let services_get = warp::get()
            .and(warp::path("services"))
            .and(warp::path::end())
            .and_then(get_services);

        let action_post = warp::post()
            .and(warp::path("service"))
            .and(warp::path::param::<String>())
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and(with_service_manager.clone())
            .and_then(post_action);

        let routes = services_get.or(action_post);
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    }
}
