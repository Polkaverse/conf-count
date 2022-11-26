use actix_web::{AsyncResponder, Error, HttpRequest, HttpResponse};
use actix_web::http::{header::HeaderValue, HeaderMap};
use futures::Future;
use futures::future::result;
use log::error;
use mongodb::{bson, doc};
use mongodb::coll::Collection;
use serde_json::{json, Value};

use crate::connection::{connect_database_collection, create_db_connection};
use crate::constants::{DB_NAME, EMAIL, NON_REGISTERED_USER_MESSAGE, RESPONSE, USERS_COLLECTION,
                       WRONG_EMAIL_FORMAT};
use crate::utils::{check_email_format, fetch_conference_for_user, fetch_conferences};

/// Fetches conference details
///
/// # Argument
///
/// * `_request` - An HTTP Request
///
/// # Return
///
/// Returns the conference details
pub fn fetch_conference_details(
    _request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    result(Ok(HttpResponse::Ok()
        .json(json!({RESPONSE: fetch_conferences()}))))
        .responder()
}

/// Fetches conference registered by a particular user
///
/// # Argument
///
/// * `request` - An HTTP Request
///
/// # Return
///
/// Returns the list of conferences registered by a user
pub fn fetch_registered_conferences(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let header: &HeaderMap<HeaderValue> = request.headers();
    let response: Value;
    let user_email: &str = header[EMAIL].to_str().unwrap();
    if check_email_format(user_email) {
        let users_collection: Collection = connect_database_collection
            (create_db_connection(), DB_NAME, USERS_COLLECTION);
        let user_doc = doc! {EMAIL: user_email};
        response = match users_collection.find_one(Some(user_doc), None).unwrap() {
            Some(_) => {
                json!({RESPONSE: &fetch_conference_for_user(user_email)})
            }
            None => {
                error!("A non registered user tried to access the conference list");
                json!({RESPONSE: NON_REGISTERED_USER_MESSAGE})
            }
        }
    } else {
        error!("{}", WRONG_EMAIL_FORMAT);
        response = json!({RESPONSE: WRONG_EMAIL_FORMAT});
    }
    result(Ok(HttpResponse::Ok()
        .json(response)))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::{HttpResponse, test};
    use actix_web::http::StatusCode;

    use crate::constants::{EMAIL, TEST_EMAIL_ID};
    use crate::request_handlers::conference_handler::{
        fetch_conference_details,
        fetch_registered_conferences};

    #[test]
    fn test_fetch_registered_conferences_success() {
        let response: HttpResponse = test::TestRequest::with_header(
            EMAIL, TEST_EMAIL_ID)
            .run(&fetch_registered_conferences)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_fetch_conference_details_success() {
        let response: HttpResponse = test::TestRequest::default()
            .run(&fetch_conference_details)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    #[should_panic]
    fn test_fetch_registered_conferences_failure() {
        test::TestRequest::default()
            .run(&fetch_registered_conferences)
            .unwrap();
    }
}
