use actix_web::{AsyncResponder, Error, HttpRequest, HttpResponse};
use actix_web::http::header::HeaderValue;
use actix_web::http::HeaderMap;
use futures::Future;
use futures::future::result;
use log::error;
use log::info;
use mongodb::coll::Collection;
use serde_json::{json, Value};

use crate::connection::{connect_database_collection, create_db_connection};
use crate::constants::{COLLECTION_EMPTY, CONFERENCE_DETAILS, CONFERENCE_ID, DB_NAME, PROCESS_COMPLETE, RESPONSE, WRONG_CONFERENCE_ID_FORMAT};
use crate::response_service::fetch_response;
use crate::user_data::trigger_camera;
use crate::utils::check_conference_id_format;

/// Returns camera response
///
/// # Argument
///
/// * `request` - An HTTP Request
///
/// # Return
///
/// Returns the Camera status
pub fn handle_camera(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response_body: Value;
    let header: &HeaderMap<HeaderValue> = request.headers();
    let conference_id: &str = header[CONFERENCE_ID].to_str().unwrap();
    if check_conference_id_format(conference_id) {
        let conference_collection: Collection = connect_database_collection(
            create_db_connection(), DB_NAME, CONFERENCE_DETAILS);
        info!("Camera triggered Successfully for conference_id {}", conference_id);
        let response: &str;
        match trigger_camera() {
            Ok(_) => {
                response = match fetch_response(conference_collection, conference_id) {
                    PROCESS_COMPLETE => PROCESS_COMPLETE,
                    _ => COLLECTION_EMPTY
                };
                info!("{}", response);
            }
            Err(camera_error) => {
                response = "No Camera Detected";
                error!("{}", camera_error);
            }
        }
        response_body = json!({
                      RESPONSE: response
                      });
    } else {
        error!("{}", WRONG_CONFERENCE_ID_FORMAT);
        response_body = json!({RESPONSE: WRONG_CONFERENCE_ID_FORMAT});
    }
    result(Ok(HttpResponse::Ok()
        .json(response_body)))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::{HttpResponse, test};
    use actix_web::http::StatusCode;

    use crate::constants::{CONFERENCE_ID, TEST_CONFERENCE_ID};
    use crate::request_handlers::camera_handler::handle_camera;

    #[test]
    fn test_handle_camera_trigger_failure()
    {
        let response: HttpResponse = test::TestRequest::with_header(CONFERENCE_ID,
                                                                    TEST_CONFERENCE_ID)
            .run(&handle_camera)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_camera_wrong_confrant_format()
    {
        let response: HttpResponse = test::TestRequest::with_header(CONFERENCE_ID, "1122")
            .run(&handle_camera)
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}