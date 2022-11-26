use actix_web::{AsyncResponder, Error, HttpRequest, HttpResponse};
use actix_web::http::header::HeaderValue;
use actix_web::http::HeaderMap;
use futures::Future;
use futures::future::result;
use log::error;
use serde_json::{json, Value};

use crate::constants::{EMAIL, RESPONSE, USER_STATUS, WRONG_EMAIL_FORMAT};
use crate::utils::{check_email_format, validate_user_email};

/// Returns user status
///
/// # Argument
///
/// * `request` - An HTTP Request
///
/// # Return
///
/// Returns the User Status
pub fn handle_forgot_password(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let header: &HeaderMap<HeaderValue> = request.headers();
    let response_body: Value;
    let email: &str = header[EMAIL].to_str().unwrap();
    if check_email_format(email) {
        response_body = json!({USER_STATUS: validate_user_email(email)});
    } else {
        error!("{}", WRONG_EMAIL_FORMAT);
        response_body = json!({RESPONSE: WRONG_EMAIL_FORMAT});
    }
    result(Ok(HttpResponse::Ok()
        .json(response_body)))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::{HttpResponse, test};
    use actix_web::http::StatusCode;

    use crate::constants::{EMAIL, TEST_EMAIL_ID};
    use crate::request_handlers::forgot_password_handler::handle_forgot_password;

    #[test]
    fn test_handle_forgot_password_success() {
        let response: HttpResponse = test::TestRequest::with_header(EMAIL, TEST_EMAIL_ID)
            .run(&handle_forgot_password)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    #[should_panic]
    fn test_handle_forgot_password_failure() {
        test::TestRequest::default()
            .run(&handle_forgot_password)
            .unwrap();
    }
}