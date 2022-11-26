use std::env;

use mongodb::{bson, doc};
use mongodb::Bson;
use mongodb::coll::Collection;
use mongodb::cursor::Cursor;
use mongodb::Document;
use rusoto_rekognition::RekognitionClient;

use crate::constants::{CONFERENCE_ID, DIFFERENT, EMAIL, FAILURE, PRESENT, SET, SIMILAR, SUCCESS,
                       USER_ID};
use crate::email_service::send_email;
use crate::image_comparison::compare_images;
use crate::request_generator::create_compare_faces_request;

static TARGET_IMAGE_PATH: &str = "Clicked_Image_Path";
static DEFAULT_MAIL_STATUS: &str = "Email not sent";
static INDEFINITE_RESULT: &str = "Indefinite Result";
static SENDER: &str = "ayush@knoldus.com";
static STATUS: &str = "status";
static ABSENT_STATUS: &str = "absent";
static PRESENT_STATUS: &str = "present";

/// Update the records for the absent participants
///
/// # Arguments
///
/// * `conference_collection` - Instance of the Database collection
///
/// * `data` - Data in the bson format(Alias for OrderedDocument)
///
/// * `user_id` - Id of the user
///
/// # Return
///
/// Returns the success or failure message of update operation
fn update_record(
    conference_collection: &Collection,
    data: Document,
    user_id: &str,
) -> &'static str {
    let record = doc! {USER_ID: user_id};
    match conference_collection.update_one(record, data, None) {
        Ok(doc) => {
            match doc.modified_count {
                0 => PRESENT,
                _ => SUCCESS
            }
        }
        Err(_) => FAILURE
    }
}

/// Update the records for the present participants
///
/// # Arguments
///
/// * `conference_collection` - Instance of the Database collection
///
/// * `user_id` - Id of the user
///
/// # Return
///
/// Returns the success or failure message of update db service
fn update_present_status(
    conference_collection: &Collection,
    user_id: &str,
) -> &'static str {
    let status_present = doc! {STATUS: PRESENT_STATUS};
    let data = doc! {SET => status_present};
    update_record(&conference_collection, data, &user_id)
}

/// Sends the E-mail to the absent participants
///
/// # Arguments
///
/// * `conference_collection` - Instance of the Database collection
///
/// * `user_id` - Id of the user
///
/// # Return
///
/// Returns the success or failure message of E-mail service
fn update_absent_status(
    conference_collection: &Collection,
    user_id: &str,
) -> &'static str {
    let status_absent = doc! {STATUS: ABSENT_STATUS, USER_ID: user_id};
    let mut mail_status: &str = DEFAULT_MAIL_STATUS;
    let absentee_emails: Cursor = conference_collection.find(
        Some(status_absent), None)
        .expect("Database connection disrupted");
    for email_list in absentee_emails {
        if let Ok(email_list) = email_list {
            if let Some(&Bson::String(ref email)) =
            email_list.get(EMAIL) {
                mail_status = send_email(SENDER, email);
            }
        }
    }
    mail_status
}

/// Update the records for the absent and present participants
///
/// # Arguments
///
/// * `rekognition_client` - Instance of the client of RekognitionClient
///
/// * `conference_collection` - Instance of the Database collection
///
/// * `user_id` - Id of the user
///
/// # Return
///
/// Returns the success or failure message of match operation
pub fn update_db(
    rekognition_client: &RekognitionClient,
    conference_collection: &Collection,
    user_id: String,
) -> &'static str {
    match compare_images(&rekognition_client,
                         create_compare_faces_request(
                             &user_id, env::var(TARGET_IMAGE_PATH)
                                 .expect("Target Image path not specified").as_str())) {
        Ok(response) => {
            match response
                {
                    SIMILAR => {
                        update_present_status(conference_collection, user_id.as_str())
                    }
                    DIFFERENT => {
                        update_absent_status(conference_collection, user_id.as_str())
                    }
                    _ => INDEFINITE_RESULT
                }
        }
        Err(aws_error) => aws_error
    }
}

/// Fetch user Ids of the registered user for a particular conference
///
/// # Arguments
///
/// * `conference_collection` - Instance of the Database collection
///
/// # Return
///
/// Returns the Ids of the user registered for a conference
pub fn fetch_user_ids(conference_collection: &Collection, conference_id: &str) -> Vec<String> {
    let mut user_ids: Vec<String> = Vec::new();
    let filter = doc!(CONFERENCE_ID: conference_id);
    if let Ok(user_data) = conference_collection.find(Some(filter), None) {
        let mut user_document: Vec<bson::Document> = Vec::new();
        for conf_user in user_data {
            user_document.push(conf_user.expect("Unable to push data into vector"))
        }
        for user_id in user_document {
            user_ids.push(user_id.get_str("user_id")
                .expect("No such key exist")
                .to_string());
        }
    }
    user_ids
}


#[cfg(test)]
mod test {
    use mongodb::{bson, doc};
    use mongodb::Client;
    use mongodb::coll::Collection;
    use mongodb::db::ThreadedDatabase;
    use mongodb::ThreadedClient;
    use rusoto_rekognition::RekognitionClient;

    use crate::connection::create_rekognition_connection;
    use crate::constants::{FAILURE, PRESENT};
    use crate::db_operations::{DEFAULT_MAIL_STATUS, PRESENT_STATUS, STATUS, update_absent_status, update_db, update_present_status, update_record};

    static AWS_SERVER_ERROR: &str = "Image key not found in s3 bucket";
    static TEST_HOST: &str = "localhost";
    const TEST_PORT: u16 = 27017;
    const INVALID_TEST_PORT: u16 = 20000;
    static TEST_DB_NAME: &str = "Conf_Count";
    static TEST_COLLECTION_NAME: &str = "emp_test";
    static MATCHED_IMAGE_KEY: &str = "1122334455";
    static UNMATCHED_IMAGE_KEY: &str = "knol-2000";
    static INVALID_IMAGE_KEY: &str = "IMAGE_KEY";
    static EMAIL_SENT: &str = "Email Sent";
    static EMAIL_NOT_SENT: &str = "Email not sent";
    static ABSENT_REGISTRATION_ID: &str = "2008";
    static PRESENT_REGISTRATION_ID: &str = "2006";

    #[test]
    fn test_update_record_success() {
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        let data = doc! {"$set" => {STATUS => PRESENT_STATUS }};
        assert_eq!(PRESENT, update_record(&employees_collection, data,
                                          MATCHED_IMAGE_KEY));
    }

    #[test]
    fn test_update_record_failure() {
        let client: Client = Client::connect(TEST_HOST, INVALID_TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        let data = doc! {"$set" => {STATUS => PRESENT_STATUS}};
        assert_eq!(FAILURE, update_record(&employees_collection, data,
                                          MATCHED_IMAGE_KEY));
    }

    #[test]
    fn test_update_db_error()
    {
        let rekognition_client: RekognitionClient = create_rekognition_connection();
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(AWS_SERVER_ERROR, update_db(
            &rekognition_client, &employees_collection,
            INVALID_IMAGE_KEY.to_string()));
    }

    #[test]
    fn test_update_db_success()
    {
        let rekognition_client: RekognitionClient = create_rekognition_connection();
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(DEFAULT_MAIL_STATUS, update_db(
            &rekognition_client, &employees_collection,
            MATCHED_IMAGE_KEY.to_string()));
    }

    #[test]
    fn test_update_db_failure()
    {
        let rekognition_client: RekognitionClient = create_rekognition_connection();
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(AWS_SERVER_ERROR, update_db(
            &rekognition_client, &employees_collection,
            UNMATCHED_IMAGE_KEY.to_string()));
    }

    #[test]
    fn test_update_present_status_success() {
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(PRESENT, update_present_status(
            &employees_collection, ABSENT_REGISTRATION_ID))
    }

    #[test]
    fn test_update_present_status_failure() {
        let client: Client = Client::connect(TEST_HOST, INVALID_TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(FAILURE, update_present_status(
            &employees_collection, ABSENT_REGISTRATION_ID))
    }

    #[test]
    fn test_update_absent_status_success() {
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(EMAIL_SENT, update_absent_status(
            &employees_collection, PRESENT_REGISTRATION_ID))
    }

    #[test]
    fn test_update_absent_status_failure() {
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let employees_collection: Collection =
            client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(EMAIL_NOT_SENT, update_absent_status(
            &employees_collection, UNMATCHED_IMAGE_KEY))
    }
}
