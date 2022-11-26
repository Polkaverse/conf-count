use actix_web::{AsyncResponder, Error, Form, http, HttpRequest, HttpResponse};
use actix_web::http::{header::HeaderValue, HeaderMap};
use chrono::{Datelike, NaiveDate};
use futures::Future;
use futures::future::result;
use log::{error, info};
use mongodb::{bson, doc};
use mongodb::Document;
use serde_json::{json, Value};

use crate::constants::{CONFERENCE_NAME, EMAIL, EMPTY_ADDRESS, ID, NAME, RESPONSE, SET, STATUS,
                       TEXT, USER_ID, WRONG_CONFERENCE_ID_FORMAT, WRONG_DATE, WRONG_DATE_FORMAT,
                       WRONG_EMAIL_FORMAT, WRONG_NAME_FORMAT, WRONG_USER_ID_FORMAT};
use crate::request_handlers::signup_handler::generate_conference_id;
use crate::utils::{add_conference, check_date_format, check_email_format, check_id_format,
                   check_name_format, check_non_empty, delete_conference, delete_user,
                   filter_conference, filter_user_conference, update_conference,
                   update_user_details};

static CONFERENCE_DATE: &str = "conference_date";
static CONFERENCE_ID: &str = "_id";
static CONFERENCE_ADDRESS1: &str = "conference_address1";
static CONFERENCE_ADDRESS2: &str = "conference_address2";
static CONFERENCE_ADDRESS3: &str = "conference_address3";
static GREATER: &str = "$gte";
static LESSER: &str = "$lte";
static DATE_FORMAT: &str = "%Y-%m-%d";

/// This function response for a particular route request
///
/// # Argument
///
/// * `_http_request` - An HTTP request
///
/// # Return
///
/// This function response for register user button
pub fn load_admin_dashboard(
    _http_request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type(TEXT)
        .body(include_str!("../../static/dashboard.html"))))
        .responder()
}

#[derive(Deserialize)]
pub struct UserId {
    user_id: String,
}

/// Respond to a user input submit request
///
/// # Argument
///
/// * `user_id` - User unique ID
///
/// # Return
///
/// Respond to a delete user button
pub fn handle_user_deletion(
    user_id: Form<UserId>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: &str;
    if check_id_format(user_id.user_id.as_str()) {
        response = delete_user(user_id.user_id.to_string())
    } else {
        error!("{}", WRONG_USER_ID_FORMAT);
        response = WRONG_USER_ID_FORMAT
    }
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(json!({RESPONSE: response}))))
        .responder()
}

/// This handler function response for a user input submit request
///
/// # Argument
///
/// * `user_data` - User unique ID, User Email and User name
///
/// # Return
///
/// This function response to a update user button
pub fn handle_user_updation(
    request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let header: &HeaderMap<HeaderValue> = request.headers();
    let response: &str;
    if check_id_format(header[USER_ID].to_str().unwrap()) {
        if check_name_format(header[NAME].to_str().unwrap()) {
            if check_email_format(header[EMAIL].to_str().unwrap()) {
                let user_id = doc! {ID: header[USER_ID].to_str().unwrap()};
                let updated_data = doc! {SET: {NAME: header[NAME].to_str().unwrap(),
                                            EMAIL: header[EMAIL].to_str().unwrap()}};
                match update_user_details(user_id, updated_data) {
                    1 => {
                        info!("User with id {} has been updated", header[USER_ID].to_str().unwrap());
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
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(json!({RESPONSE: response}))))
        .responder()
}

#[derive(Deserialize)]
pub struct Conference {
    conference_id: String,
}

/// This handler function response to a delete conference request
///
/// # Argument
///
/// * `conference` - New Conference name
///
/// # Return
///
/// This function response to a delete user button
pub fn handle_conference_deletion(
    conference: Form<Conference>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: &str = if check_id_format(conference.conference_id.as_str()) {
        let filter: Document = doc! {CONFERENCE_ID : conference.conference_id.as_str()};
        delete_conference(filter)
    } else {
        error!("{}", WRONG_CONFERENCE_ID_FORMAT);
        WRONG_CONFERENCE_ID_FORMAT
    };
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(json!({RESPONSE: response}))))
        .responder()
}

#[derive(Deserialize)]
pub struct UpdateConferenceDetails {
    conference_id: String,
    conference_date: String,
    conference_address1: String,
    conference_address2: String,
    conference_address3: String,
}

/// This handler response to a update conference request
///
/// # Argument
///
/// * `update_conference_details` - Conference Id, Conference date, Conference address
///
/// # Return
///
/// This function response to update conStatusference button
pub fn handle_conference_updation(
    update_conference_details: Form<UpdateConferenceDetails>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: &str;
    if check_id_format(update_conference_details.conference_id.as_str()) {
        if check_date_format(update_conference_details.conference_date.as_str()) {
            if check_non_empty(&[update_conference_details.conference_address1.as_str(),
                update_conference_details.conference_address2.as_str(),
                update_conference_details.conference_address3.as_str()]) {
                let filter_data: Document = doc! {
                        CONFERENCE_ID : update_conference_details.conference_id.as_str(),
                };

                let updated_conference_data: Document = doc! {
                SET : {
                CONFERENCE_DATE : update_conference_details.conference_date.as_str(),
                CONFERENCE_ADDRESS1 : update_conference_details.conference_address1.as_str(),
                CONFERENCE_ADDRESS2 : update_conference_details.conference_address2.as_str(),
                CONFERENCE_ADDRESS3 : update_conference_details.conference_address3.as_str(),
                }
            };
                response = update_conference(filter_data, updated_conference_data);
            } else {
                error!("{}", EMPTY_ADDRESS);
                response = EMPTY_ADDRESS;
            }
        } else {
            error!("{}", WRONG_DATE_FORMAT);
            response = WRONG_DATE_FORMAT;
        }
    } else {
        error!("{}", WRONG_CONFERENCE_ID_FORMAT);
        response = WRONG_CONFERENCE_ID_FORMAT;
    }
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(json!({RESPONSE: response}))))
        .responder()
}

#[derive(Deserialize)]
pub struct NewConference {
    add_conference: String,
    add_conference_date: String,
    add_conference_address1: String,
    add_conference_address2: String,
    add_conference_address3: String,
}

pub enum ConferenceStatus {
    Completed,
    NotCompleted,
}

impl ConferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ConferenceStatus::Completed => "completed",
            ConferenceStatus::NotCompleted => "not_completed",
        }
    }
}

/// This handler response to a add conference request
///
/// # Argument
///
/// * `new_conference` - Conference name, Conference date, Conference address
///
/// # Return
///
/// This function response to a add new conference button
pub fn handle_conference_addition(
    new_conference: Form<NewConference>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: &str;
    if check_non_empty(&[new_conference.add_conference.as_str()]) {
        if check_date_format(new_conference.add_conference_date.as_str()) {
            if check_non_empty(&[new_conference.add_conference_address1.as_str(),
                new_conference.add_conference_address2.as_str(),
                new_conference.add_conference_address3.as_str()]) {
                let mut conf_data: String = new_conference.add_conference.clone();
                conf_data.push_str(&new_conference.add_conference_date);
                let conf_id: String = generate_conference_id(conf_data).to_string();
                let validation_data: Document = doc! {
                        CONFERENCE_ID : conf_id.as_str(),
                };

                let conference_data: Document = doc! {
                    CONFERENCE_ID : conf_id.as_str(),
                    CONFERENCE_NAME : new_conference.add_conference.as_str(),
                    CONFERENCE_DATE : new_conference.add_conference_date.as_str(),
                    CONFERENCE_ADDRESS1 : new_conference.add_conference_address1.as_str(),
                    CONFERENCE_ADDRESS2 : new_conference.add_conference_address2.as_str(),
                    CONFERENCE_ADDRESS3 : new_conference.add_conference_address3.as_str(),
                    STATUS: ConferenceStatus::NotCompleted.as_str(),
            };
                response = add_conference(conference_data, validation_data);
            } else {
                error!("{}", EMPTY_ADDRESS);
                response = EMPTY_ADDRESS;
            }
        } else {
            error!("{}", WRONG_DATE_FORMAT);
            response = WRONG_DATE_FORMAT;
        }
    } else {
        error!("Conference name is empty");
        response = "Conference name is empty";
    }
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(json!({RESPONSE: response}))))
        .responder()
}

#[derive(Deserialize)]
pub struct FilterConference {
    first_conference_date: String,
    last_conference_date: String,
}

/// Respond to a filter conference request
///
/// # Argument
///
/// * `filter_conferences` - first and last conference date to filter conferences
///
/// # Return
///
/// Responds to a filter conference button
pub fn handle_conference_filtration(
    filter_conferences: Form<FilterConference>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: Value;
    let day1: &str = filter_conferences.first_conference_date.as_str();
    let day2: &str = filter_conferences.last_conference_date.as_str();
    if check_date_format(day1) && check_date_format(day2) {
        let first_date: NaiveDate = NaiveDate::parse_from_str(day1, DATE_FORMAT).unwrap();
        let last_date: NaiveDate = NaiveDate::parse_from_str(day2, DATE_FORMAT).unwrap();
        let no_of_days_day1: i32 = first_date.num_days_from_ce();
        let no_of_days_day2: i32 = last_date.num_days_from_ce();
        let diff_of_dates: i32 = no_of_days_day2 - no_of_days_day1;
        match diff_of_dates {
            0..=365_000 => {
                let filter = doc! {CONFERENCE_DATE: {GREATER: day1,LESSER: day2,},};
                info!("Admin just filtered out some conferences");
                response = json!({RESPONSE: filter_conference(filter)});
            }
            _ => {
                error!("{}", WRONG_DATE);
                response = json!({RESPONSE: WRONG_DATE.to_string()});
            }
        }
    } else {
        error!("{}", WRONG_DATE_FORMAT);
        response = json!({RESPONSE: WRONG_DATE_FORMAT});
    }
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(response)))
        .responder()
}

#[derive(Deserialize)]
pub struct FilterUser {
    conference_id: String,
}

/// Respond to a view users request for a specific conference
///
/// # Argument
///
/// * `filter_conferences` - conference name
///
/// # Return
///
/// Responds to a view users button for a specific conference
pub fn handle_user_filtration(
    filter_conference: Form<FilterUser>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: String = if check_id_format(filter_conference.conference_id.as_str()) {
        filter_user_conference(filter_conference.conference_id.as_str())
    } else {
        info!("Admin entered wrong Conference Id format");
        WRONG_CONFERENCE_ID_FORMAT.to_string()
    };
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .json(json!({RESPONSE: response}))))
        .responder()
}

/// Respond to timeout of the form
///
/// # Argument
///
/// * `http_request` - An HTTP request
///
/// # Return
///
/// Responds to a session timeout
pub fn handle_admin_timeout(
    _http_request: &HttpRequest
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    result(Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type(TEXT)
        .body(include_str!("../../static/session_timeout.html"))))
        .responder()
}


#[cfg(test)]
pub mod tests {
    use actix_web::{Form, http, HttpResponse, test};
    use actix_web::http::StatusCode;
    use futures::future::Future;

    use crate::constants::{EMAIL, NAME, USER_ID};
    use crate::request_handlers::admin_handler::{
        Conference,
        FilterConference,
        handle_admin_timeout,
        handle_conference_addition,
        handle_conference_deletion,
        handle_conference_filtration,
        handle_conference_updation,
        handle_user_deletion,
        handle_user_updation,
        load_admin_dashboard,
        NewConference,
        UpdateConferenceDetails,
        UserId};

    #[test]
    fn test_load_admin_dashboard_success() {
        let response: HttpResponse = test::TestRequest::with_header("content-type", "text/html")
            .run(&load_admin_dashboard)
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[test]
    fn test_handle_admin_timeout_success() {
        let response: HttpResponse = test::TestRequest::with_header("content-type", "text/html")
            .run(&handle_admin_timeout)
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[test]
    fn test_handle_user_deletion_success() {
        let user_id = UserId { user_id: "2121212121".to_string() };
        assert!(handle_user_deletion(Form(user_id)).wait().is_ok())
    }

    #[test]
    fn test_handle_user_deletion_failure() {
        let user_id = UserId { user_id: "212".to_string() };
        assert!(handle_user_deletion(Form(user_id)).wait().is_ok())
    }

    #[test]
    fn test_handle_user_update_success() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "1111111111")
            .header(NAME, "Test")
            .header(EMAIL, "test@knoldus.in")
            .run(&handle_user_updation)
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_update_wrong_name_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "111")
            .header(NAME, "a")
            .header(EMAIL, "test@knoldus.in")
            .run(&handle_user_updation)
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_update_wrong_id_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "111")
            .header(NAME, "Test")
            .header(EMAIL, "test@knoldus.in")
            .run(&handle_user_updation)
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_user_update_wrong_email_format() {
        let response: HttpResponse = test::TestRequest::default()
            .header(USER_ID, "1111111111")
            .header(NAME, "Test")
            .header(EMAIL, "test@knoldus")
            .run(&handle_user_updation)
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_handle_conference_deletion_success() {
        let conf_name = Conference {
            conference_id: "123456789".to_string(),
        };
        assert!(handle_conference_deletion(Form(conf_name)).wait().is_ok())
    }

    #[test]
    fn test_handle_conference_deletion_failure() {
        let conf_name = Conference {
            conference_id: "Test_conf_name".to_string(),
        };
        assert!(handle_conference_deletion(Form(conf_name)).wait().is_ok())
    }

    #[test]
    fn test_handle_conference_update_invalid_id() {
        let conf_data = UpdateConferenceDetails {
            conference_id: "Test conference".to_string(),
            conference_date: "2018-06-24".to_string(),
            conference_address1: "Test add1".to_string(),
            conference_address2: "Test_add2".to_string(),
            conference_address3: "Test add3".to_string(),
        };
        assert!(handle_conference_updation(Form(conf_data)).wait().is_ok())
    }

    #[test]
    fn test_handle_conference_update_invalid_date() {
        let conf_data = UpdateConferenceDetails {
            conference_id: "1111111111".to_string(),
            conference_date: "Test date".to_string(),
            conference_address1: "Test add1".to_string(),
            conference_address2: "Test_add2".to_string(),
            conference_address3: "Test add3".to_string(),
        };
        assert!(handle_conference_updation(Form(conf_data)).wait().is_ok())
    }

    #[test]
    fn test_handle_conference_update_invalid_address() {
        let conf_data = UpdateConferenceDetails {
            conference_id: "1111111111".to_string(),
            conference_date: "2018-06-24".to_string(),
            conference_address1: "".to_string(),
            conference_address2: "".to_string(),
            conference_address3: "".to_string(),
        };
        assert!(handle_conference_updation(Form(conf_data)).wait().is_ok())
    }

    #[test]
    fn test_handle_conference_update_success() {
        let conf_data = UpdateConferenceDetails {
            conference_id: "5544332211".to_string(),
            conference_date: "2018-06-24".to_string(),
            conference_address1: "test_address_1".to_string(),
            conference_address2: "test_address_2".to_string(),
            conference_address3: "test_address_3".to_string(),
        };
        assert!(handle_conference_updation(Form(conf_data)).wait().is_ok())
    }

    #[test]
    fn test_handle_conference_addition_success() {
        let new_conf = NewConference {
            add_conference: "test_conference".to_string(),
            add_conference_date: "2018-06-24".to_string(),
            add_conference_address1: "test_address_1".to_string(),
            add_conference_address2: "test_address_2".to_string(),
            add_conference_address3: "test_address_3".to_string(),
        };
        assert!(handle_conference_addition(Form(new_conf)).wait().is_ok())
    }

    #[test]
    pub fn test_handle_conference_filtration_success() {
        let filter_conf = FilterConference {
            first_conference_date: "09-02-2019".to_string(),
            last_conference_date: "05-07-2019".to_string(),
        };
        assert!(handle_conference_filtration(Form(filter_conf)).wait().is_ok())
    }


    #[test]
    pub fn test_handle_conference_addition_invalid_name_format() {
        let new_conf = NewConference {
            add_conference: "T".to_string(),
            add_conference_date: "Test date".to_string(),
            add_conference_address1: "Test add1".to_string(),
            add_conference_address2: "Test add2".to_string(),
            add_conference_address3: "Test add3".to_string(),
        };
        assert!(handle_conference_addition(Form(new_conf)).wait().is_ok())
    }
}
