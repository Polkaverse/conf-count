use actix_web::{AsyncResponder, Error, http, HttpRequest, HttpResponse};
use futures::Future;
use futures::future::result;

use crate::constants::TEXT;

/// Responds to timeout of the form
///
/// # Argument
///
/// * '_http_request' - An HTTP request
///
/// # Return
///
/// Displays the timeout page
pub fn handle_logout(
    _http_request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type(TEXT)
        .body(include_str!("../../static/logout.html"))))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::{http, HttpResponse, test};

    use crate::request_handlers::logout_handler::handle_logout;

    #[test]
    fn test_handle_logout_success() {
        let response: HttpResponse = test::TestRequest::default()
            .run(&handle_logout)
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
    }
}
