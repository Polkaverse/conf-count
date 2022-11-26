use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::str;

use actix::FinishStream;
use actix_web::{Either, error, http, HttpMessage, HttpRequest, HttpResponse, multipart};
use actix_web::AsyncResponder;
use actix_web::error::MultipartError;
use actix_web::http::header::ContentDisposition;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use futures::{future, Future, Stream};
use futures::future::ok;
use log::{error, info};
use serde_json::json;
use uuid::Uuid;

use crate::constants::{ALREADY_REGISTERED, RESPONSE, S3_UPLOAD_SUCCESS, USER_INFO};
use crate::user_data_upload_operations::{upload_user_image, write_user_info};
use crate::utils::{check_password_format, path_exists};

const DB_SUCCESS_MESSAGE: &str = "Successfully Inserted";
static SUCCESS: &str = "Your data is successfully registered";
static NOT_SUCCESS: &str = "Registration unsuccessful as some fields are missing";
static USER_DATA: &str = "user_data";
static UPLOAD: &str = "upload.jpg";
static TEXT: &str = "text/html; charset=utf-8";
static INVALID_FILE: &str = "Unable to read file";
const USER_NAME: &str = "user_name";
const EMAIL_ID: &str = "email_id";
const UPLOAD_IMAGE: &str = "upload_image";
const PASSWORD: &str = "user_password";

/// Generate_registration_number generates a unique registration number of the user
///
/// # Return
///
/// This function returns the unique registration number of the user
pub fn generate_user_id() -> u32 {
    Uuid::new_v4().as_fields().0
}

/// Generates a unique registration number of the user
///
/// # Return
///
/// This function returns the unique registration number of the user
pub fn generate_conference_id(conf_data: String) -> u32 {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, conf_data.as_bytes()).as_fields().0
}

/// This function create a file
///
/// # Argument
///
/// * `file_path` - Path of the file
///
/// * `append_permission` - Permission to append data on file
///
/// # Return
///
/// This function returns a file
fn create_file(file_path: &str, append_permission: bool) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(append_permission)
        .create(true)
        .open(file_path)
        .expect(INVALID_FILE)
}

/// The handler function response for a user registration event
///
/// # Argument
///
/// * 'http_request' - An HTTP request
///
/// # Return
///
/// This function respond to a particular event trigger
pub fn handle_user_signup(
    http_request: &HttpRequest,
) -> Box<dyn Future<Item=HttpResponse, Error=MultipartError>> {
    let uuid: String = generate_user_id().to_string();
    http_request
        .multipart()
        .from_err()
        .and_then(move |multipart_item| match multipart_item {
            multipart::MultipartItem::Field(field) => {
                let content: ContentDisposition = field.content_disposition().unwrap();
                let key: &str = content.get_name().unwrap();
                match key {
                    USER_NAME | EMAIL_ID => {
                        let user_data_file_path: &str = USER_DATA;
                        let mut file: File = create_file(user_data_file_path, true);

                        Either::A(Either::A(
                            field
                                .from_err()
                                .map(move |chunk| {
                                    writeln!(
                                        file,
                                        "{}",
                                        str::from_utf8(&chunk).expect(
                                            "Unable to parse bytes")
                                    )
                                })
                                .finish(),
                        ))
                    }
                    PASSWORD => {
                        let user_data_file_path: &str = USER_DATA;
                        let mut file: File = create_file(user_data_file_path, true);

                        Either::A(Either::B(
                            field
                                .map(move |chunk| {
                                    let password: &str =
                                        str::from_utf8(&chunk).expect(
                                            "Unable to parse bytes");
                                    if check_password_format(password) {
                                        let mut hasher: Sha256 = Sha256::new();
                                        hasher.input_str(password);
                                        writeln!(file, "{}", hasher.result_str().as_str())
                                    } else {
                                        writeln!(file)
                                    }
                                })
                                .finish()
                        ))
                    }

                    UPLOAD_IMAGE => {
                        let user_image: &str = UPLOAD;
                        let mut file: File = create_file(user_image, false);
                        Either::B(Either::A(field
                            .fold(0i64, move |acc, bytes| {
                                let file_size: Result<i64, MultipartError> = file
                                    .write(bytes.as_ref())
                                    .map(|_| acc + bytes.len() as i64)
                                    .map_err(|error| {
                                        error!("Writing image in file failed with : {:?}", error);
                                        error::MultipartError::Payload(error::PayloadError::Io(
                                            error,
                                        ))
                                    });
                                future::result(file_size)
                            }).map(|_| ())))
                    }
                    _ => Either::B(Either::B(ok(()))),
                }
            }
            multipart::MultipartItem::Nested(_) => Either::B(Either::B(ok(()))),
        })
        .finish()
        .map(move |_| {
            HttpResponse::Ok().json({
                if path_exists("upload.jpg") && fs::metadata("upload.jpg").unwrap().len() > 0 {
                    let db_success: &str =
                        write_user_info(uuid.clone(), USER_INFO);
                    fs::remove_file(USER_INFO).expect("User info file not found");
                    match db_success {
                        DB_SUCCESS_MESSAGE => {
                            let s3_success: &str = upload_user_image(
                                UPLOAD,
                                uuid.clone(),
                            );

                            fs::remove_file(UPLOAD).expect(
                                "Unable to delete user information");

                            match s3_success {
                                S3_UPLOAD_SUCCESS => {
                                    info!("{}", "New user registered");
                                    json!({RESPONSE: SUCCESS})
                                }
                                _ => {
                                    json!({RESPONSE: "Please upload the image"})
                                }
                            }
                        }
                        ALREADY_REGISTERED => {
                            fs::remove_file(UPLOAD).expect(
                                "Unable to delete uer information");
                            json!({RESPONSE: ALREADY_REGISTERED})
                        }
                        _ => {
                            fs::remove_file(UPLOAD).expect(
                                "Unable to delete use information");
                            json!({RESPONSE: NOT_SUCCESS})
                        }
                    }
                } else {
                    fs::remove_file(USER_INFO).expect("User info file not found");
                    error!("User is trying to register without image");
                    json!({RESPONSE: "Please upload the image"})
                }
            })
        })
        .responder()
}

/// The handler function response for a particular route
///
/// # Argument
///
/// * 'http_request' - An HTTP request
///
/// # Return
///
/// This function response to a particular event trigger
pub fn load_registration_form(_http_request: &HttpRequest) -> Result<HttpResponse, error::Error> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type(TEXT)
        .body(include_str!("../../static/signup.html")))
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;

    use actix_web::{http, HttpResponse, test};

    use crate::request_handlers::signup_handler::{create_file, generate_conference_id,
                                                  generate_user_id, load_registration_form};

    static TRUE: &str = "True";
    static FALSE: &str = "False";

    #[test]
    fn test_form_response_success() {
        let response: HttpResponse = test::TestRequest::with_header("content-type",
                                                                    "text/html")
            .run(&load_registration_form)
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[test]
    #[should_panic]
    fn test_create_file_failure() {
        create_file("", true);
    }

    #[test]
    fn test_create_file_success() {
        let _ = create_file("tests/resources/test_file", true);
        assert!(File::open("tests/resources/test_file").is_ok());
    }

    fn test_range() -> &'static str {
        match generate_user_id() {
            1..=4294967295 => TRUE,
            _ => FALSE,
        }
    }

    #[test]
    fn test_generate_registration_number_success() {
        assert_eq!(TRUE, test_range());
    }

    #[test]
    fn test_generate_conference_id_success()
    {
        assert!(generate_conference_id(
            "1234567".to_string()) > 0)
    }
}