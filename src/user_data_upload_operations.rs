use std::{ffi::OsStr, fs, path::Path};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread::spawn;

use log::error;
use log::info;
use mongodb::{bson, doc};
use mongodb::coll::Collection;
use s3::bucket::Bucket;

use crate::connection::{connect_database_collection, create_bucket_connection, create_db_connection,
                        insert_data};
use crate::constants::{ALREADY_REGISTERED, DB_NAME, EMAIL, ID, NAME, PASSWORD, S3_UPLOAD_SUCCESS,
                       USER, USER_TYPE, USERS_COLLECTION};
use crate::utils::{check_email_format, check_name_format};

const IMAGE_FORMATS: [&str; 2] = ["jpg", "png"];
static INVALID_IMAGE_FORMAT: &str = "Invalid Image Format";
static IO_ERROR: &str = "No such file or directory found";
static S3_UPLOAD_FAILURE: &str = "Unable to upload image on S3 bucket";
static INVALID_EXTENSION: &str = "Invalid extension";
static REGISTRATION_UNSUCCESSFUL: &str = "Registration unsuccessful as some fields are missing";
const SUCCESS_STATUS_CODE: u32 = 200;
const S3ERROR: u32 = 400;

/// Get the mime type for a clicked image
///
/// # Arguments
///
/// * `file` - This is the path of user's image
///
/// # Return
///
/// This function returns mime type of user's image
fn extract_image_extension(file: &str) -> Result<&str, &'static str> {
    let clicked_image_format: &str = Path::new(file)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or(INVALID_EXTENSION);
    for image_format in IMAGE_FORMATS.iter() {
        if *image_format == clicked_image_format {
            return Ok(clicked_image_format);
        }
    }
    Err(INVALID_IMAGE_FORMAT)
}


/// This function upload the user image on s3 bucket returning a string response message
///
/// # Arguments
///
/// * `upload_credentials` - This is the instance of UploadCredentials
///
/// * `user_id` - This is the unique id of the user
///
/// # Return
///
/// This function returns image upload response message from the s3 bucket.
pub fn upload_user_image(user_image: &'static str, user_id: String) -> &'static str {
    let bucket: Bucket = create_bucket_connection();
    match extract_image_extension(user_image.trim()) {
        Err(file_extension_error) => file_extension_error,
        Ok(content_type) => match fs::read(user_image.trim()) {
            Ok(clicked_image) => {
                match
                    spawn(move || match bucket.put(user_id.as_str(),
                                                   clicked_image.as_slice(), content_type) {
                        Ok(s3_result) => s3_result.1,
                        Err(s3_error) => {
                            error!("{:?}", s3_error);
                            S3ERROR
                        }
                    }).join().unwrap()
                    {
                        SUCCESS_STATUS_CODE => S3_UPLOAD_SUCCESS,
                        _ => S3_UPLOAD_FAILURE,
                    }
            }
            Err(_) => IO_ERROR,
        },
    }
}


/// Write user form data in mongodb
///
/// # Arguments
///
/// * `user_id` - This is the unique user identity number
///
/// * `user_data_path` - File path of user data
///
/// * `db_name` - Database Name
///
/// * `collection_name` - Collection Name
///
/// # Return
///
/// Returns success or failure message of write method
pub fn write_user_info(
    user_id: String,
    user_data_path: &'static str,
) -> &'static str {
    let mut registration_id: String = String::new();
    let mut name: String = String::new();
    let mut email: String = String::new();
    let mut password: String = String::new();
    registration_id.push_str(user_id.as_str());
    let file: File = File::open(user_data_path).unwrap();
    for (counter, line) in BufReader::new(file).lines().enumerate() {
        match counter + 1 {
            1 => name.push_str(line.unwrap().as_str()),
            2 => email.push_str(line.unwrap().as_str()),
            3 => password.push_str(line.unwrap().as_str()),
            _ => info!("No data found"),
        }
    }

    if (check_name_format(name.as_str()) && check_email_format(email.as_str()))
        && !password.is_empty() {
        let collection: Collection =
            connect_database_collection(
                create_db_connection(), DB_NAME, USERS_COLLECTION);
        let filter_doc = doc! {EMAIL: email.clone()};
        match collection.count(Some(filter_doc), None).expect("Unable to connect to Mongo DB") {
            0 => {
                let data = doc! {
                                    ID: registration_id.as_str(),
                                    NAME: name.as_str(),
                                    EMAIL: email.as_str(),
                                    PASSWORD: password.as_str(),
                                    USER_TYPE: USER,
                                };
                insert_data(collection, data)
            }
            _ => {
                error!("User with email {} already registered", email);
                ALREADY_REGISTERED
            }
        }
    } else {
        error!("{}", REGISTRATION_UNSUCCESSFUL);
        REGISTRATION_UNSUCCESSFUL
    }
}


#[cfg(test)]
pub mod tests {

    use crate::constants::{ALREADY_REGISTERED, S3_UPLOAD_SUCCESS};
    use crate::user_data_upload_operations::{extract_image_extension, INVALID_IMAGE_FORMAT, IO_ERROR,
                                             REGISTRATION_UNSUCCESSFUL, upload_user_image,
                                             write_user_info};

    pub static TEST_IMAGE_FILE: &str = "tests/resources/test.jpg";
    pub static TEST_UUID: &str = "1122334455";
    static TEST_TEXT_FILE: &str = "tests/resources/test.txt";

    #[test]
    fn test_extract_image_extension_success() {
        assert_eq!(
            extract_image_extension(TEST_IMAGE_FILE).unwrap_or_default(),
            "jpg"
        )
    }

    #[test]
    fn test_extract_image_extension_failure() {
        assert_eq!(
            extract_image_extension(TEST_TEXT_FILE).unwrap_err(),
            INVALID_IMAGE_FORMAT
        );
    }

    #[test]
    fn test_upload_user_image_success() {
        let upload_image: &str = TEST_IMAGE_FILE;
        assert_eq!(
            upload_user_image(upload_image, TEST_UUID.to_string()),
            S3_UPLOAD_SUCCESS
        );
    }

    #[test]
    fn test_upload_user_image_failure() {
        let upload_image: &str = "tests/wrong_path.jpg";
        assert_eq!(
            upload_user_image(upload_image, TEST_UUID.to_string()),
            IO_ERROR
        );
    }


    #[test]
    fn test_upload_user_image_file_format_failure() {
        let upload_image: &str = TEST_TEXT_FILE;
        assert_eq!(
            upload_user_image(upload_image, TEST_UUID.to_string()),
            INVALID_IMAGE_FORMAT
        );
    }

    #[test]
    fn test_put_data_mongo_db_already_register_failure() {
        assert_eq!(
            write_user_info(TEST_UUID.to_owned(), TEST_TEXT_FILE),
            ALREADY_REGISTERED)
    }

    #[test]
    fn test_put_data_mongo_db_invalid_data_failure() {
        assert_eq!(
            write_user_info(TEST_UUID.to_owned(), "tests/resources/test_file"),
            REGISTRATION_UNSUCCESSFUL)
    }
}
