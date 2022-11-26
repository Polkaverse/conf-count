use actix_web::{AsyncResponder, Error, HttpRequest, HttpResponse};
use actix_web::http::{header::HeaderValue, HeaderMap};
use futures::Future;
use futures::future::result;
use log::{error, info};
use mongodb::{bson, doc};
use serde_json::json;

use crate::constants::{CONFERENCE_ID, EMAIL, ID, JSON, NAME, RESPONSE, SET, USER_ID,
                       WRONG_CONFERENCE_ID_FORMAT, WRONG_EMAIL_FORMAT, WRONG_NAME_FORMAT,
                       WRONG_USER_ID_FORMAT};
use crate::utils::{check_email_format, check_id_format, check_name_format, fetch_user_details,
                   update_user_details, validate_user_for_conference};

/// Returns user details
///
/// # Argument
///
/// * `request` - An HTTP Request
///
/// # Return
///
/// Returns the user details
pub fn handle_user_details(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let header: &HeaderMap<HeaderValue> = request.headers();
    let user_key = doc! {EMAIL: header[EMAIL].to_str().unwrap()};
    result(Ok(HttpResponse::Ok()
        .content_type(JSON)
        .json(fetch_user_details(user_key))))
        .responder()
}

/// Returns User update response
///
/// # Argument
///
/// * `request` - An HTTP Request
///
/// # Return
///
/// Returns the User Updation status
pub fn handle_user_details_updation(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let header: &HeaderMap<HeaderValue> = request.headers();
    let response: &str;
    if check_id_format(header[USER_ID].to_str().unwrap()) {
        if check_name_format(header[NAME].to_str().unwrap()) {
            if check_email_format(header[EMAIL].to_str().unwrap()) {
                let user_id = doc! {ID: header[USER_ID].to_str().unwrap()};
                let updated_data = doc! {SET: {NAME: header[NAME].to_str().unwrap(), EMAIL: header[EMAIL].to_str().unwrap()}};
                match update_user_details(user_id, updated_data) {
                    1 => {
                        info!("User details updated");
                        response = "User details updated"
                    }
                    _ => {
                        error!("User ID not found");
                        response = "User ID not found"
                    }
                }
            } else {
                error!("{}", WRONG_EMAIL_FORMAT);
                response = WRONG_EMAIL_FORMAT;
            }
        } else {
            error!("{}", WRONG_NAME_FORMAT);
            response = WRONG_NAME_FORMAT;
        }
    } else {
        error!("{}", WRONG_USER_ID_FORMAT);
        response = WRONG_USER_ID_FORMAT;
    }
    result(Ok(HttpResponse::Ok()
        .json(json!({RESPONSE: response}))))
        .responder()
}

/// Returns register conference response
///
/// # Argument
///
/// * `request` - An HTTP Request
///
/// # Return
///
/// Returns the Conference registration status
pub fn handle_conference_registration(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let header: &HeaderMap<HeaderValue> = request.headers();
    let response: &str;
    let user_email: &str = header[EMAIL].to_str().unwrap();
    if check_email_format(user_email) {
        let user_id: &str = header[USER_ID].to_str().unwrap();
        if check_id_format(user_id) {
            let conference_id: &str = header[CONFERENCE_ID].to_str().unwrap();
            if check_id_format(conference_id) {
                response = validate_user_for_conference(user_email, user_id, conference_id);
            } else {
                error!("{}", WRONG_CONFERENCE_ID_FORMAT);
                response = WRONG_CONFERENCE_ID_FORMAT;
            }
        } else {
            error!("{}", WRONG_USER_ID_FORMAT);
            response = WRONG_USER_ID_FORMAT;
        }
    } else {
        error!("{}", WRONG_EMAIL_FORMAT);
        response = WRONG_EMAIL_FORMAT;
    }
    result(Ok(HttpResponse::Ok()
        .content_type(JSON)
        .json(json!({RESPONSE: response}))))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::{HttpResponse, test};
    use actix_web::http::StatusCode;

    use crate::constants::{ CONFERENCE_ID, EMAIL, NAME, TEST_CONFERENCE_ID, TEST_EMAIL_ID,
                            TEST_NAME, USER_ID};

    use crate::request_handlers::user_details_handler::{
        handle_conference_registration, handle_user_details, handle_user_details_updation};

    #[test]
    fn test_handle_user_details_success() {
        let response: HttpResponse = test::TestRequest::with_header(EMAIL, TEST_EMAIL_ID)
            .run(&handle_user_details)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_conference_registration_success() {
        let response: HttpResponse = test::TestRequest::default()
            .header(EMAIL, TEST_EMAIL_ID)
            .header(USER_ID, "1111111111")
            .header(CONFERENCE_ID, "5544332211")
            .run(&handle_conference_registration)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_conference_registration_wrong_user_id_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(EMAIL, TEST_EMAIL_ID)
            .header(USER_ID, "1111")
            .header(CONFERENCE_ID, "5544332211")
            .run(&handle_conference_registration)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_conference_registration_wrong_conference_id_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(EMAIL, TEST_EMAIL_ID)
            .header(USER_ID, "1111")
            .header(CONFERENCE_ID, "5544")
            .run(&handle_conference_registration)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_conference_registration_wrong_email_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(EMAIL, "test@knoldus")
            .header(USER_ID, "1111")
            .header(CONFERENCE_ID, "5544332211")
            .run(&handle_conference_registration)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_details_updation_success() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "1111111111")
            .header(EMAIL, TEST_EMAIL_ID)
            .header(NAME, TEST_NAME)
            .run(&handle_user_details_updation)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_details_updation_failure() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, TEST_CONFERENCE_ID)
            .header(EMAIL, TEST_EMAIL_ID)
            .header(NAME, TEST_NAME)
            .run(&handle_user_details_updation)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_details_update_wrong_email_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "1111111111")
            .header(EMAIL, "test@knoldus")
            .header(NAME, TEST_NAME)
            .run(&handle_user_details_updation)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_details_update_wrong_name_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "1111111111")
            .header(EMAIL, "test@knoldus.in")
            .header(NAME, "T")
            .run(&handle_user_details_updation)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_details_update_wrong_id_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "111")
            .header(EMAIL, "test@knoldus")
            .header(NAME, TEST_NAME)
            .run(&handle_user_details_updation)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}