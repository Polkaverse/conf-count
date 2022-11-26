use actix_web::{AsyncResponder, Error, Form, http, HttpRequest, HttpResponse};
use futures::Future;
use futures::future::result;
use serde_json::json;

use crate::constants::{RESPONSE, TEXT};
use crate::utils::validate_user_details;

/// Respond to a particular route
///
/// # Argument
///
/// * `_http_request` - An HTTP request
///
/// # Return
///
/// Responds to a particular event trigger
pub fn load_login_form(
    _http_request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type(TEXT)
        .body(include_str!("../../static/login.html"))))
        .responder()
}

#[derive(Deserialize)]
pub struct UserData {
    email: String,
    password: String,
}

/// Returns user type
///
/// # Argument
///
/// * `user_data` - User e-mail and password
///
/// # Return
///
/// Returns the User type
pub fn handle_login(
    user_data: Form<UserData>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response = json!({RESPONSE: validate_user_details(
        user_data.email.as_str(), user_data.password.as_str())
    });
    result(Ok(HttpResponse::Ok()
        .json(response)))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::{Form, http, HttpResponse, test};
    use futures::future::Future;

    use crate::constants::{PASSWORD, TEST_EMAIL_ID};
    use crate::request_handlers::login_handler::{handle_login, load_login_form, UserData};

    #[test]
    fn test_load_login_form_success() {
        let response: HttpResponse = test::TestRequest::default()
            .run(&load_login_form)
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[test]
    fn test_handle_login_success()
    {
        let user_data = UserData
            {
                email: TEST_EMAIL_ID.to_string(),
                password: PASSWORD.to_string(),
            };
        assert!(handle_login(Form(user_data)).wait().is_ok());
    }
}