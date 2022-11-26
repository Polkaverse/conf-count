use std::path::Path;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use log::{error, info};
use mongodb::{bson, doc};
use mongodb::coll::Collection;
use mongodb::cursor::Cursor;
use mongodb::Document;
use regex;
use regex::RegexSet;

use crate::connection::{connect_database_collection, create_db_connection};
use crate::constants::{ABSENT, ADMIN, ALREADY_REGISTERED, CONFERENCE_COLLECTION, CONFERENCE_DETAILS,
                       CONFERENCE_ID, CONFERENCE_NAME, DB_NAME, DELETION_FAIL,
                       DELETION_SUCCESS, EMAIL, ID, NAME, NO_CONFERENCE, NON_REGISTERED_USER_MESSAGE,
                       NOT_COMPLETED, PASSWORD, REGISTERED_USER_MESSAGE, SET, STATUS, UPDATION_FAIL,
                       UPDATION_SUCCESS, USER, USER_ID, USER_TYPE, USERS_COLLECTION,
                       WRONG_EMAIL_FORMAT, WRONG_PASSWORD_FORMAT};

static WRONG_PASSWORD: &str = "Wrong Password";
static WRONG_USER_TYPE: &str = "User Type is ambiguous, Please contact Admin";
static NO_USER_TYPE: &str = "User_type field not present";
static RESET_SUCCESSFUL: &str = "Password has been reset successfully";
static RESET_UNSUCCESSFUL: &str = "Password and confirm password field does not match";
static EMPTY_DATA: &str = "No such value exists";
static ADDITION_FAIL: &str = "Error occurred in adding conference";
static ADDITION_SUCCESS: &str = "Successfully added conference";

/// Validate weather function's arguments are empty or not
///
/// # Argument
///
/// * `args` - Collection of arguments
///
/// # Return
///
/// Return boolean value
pub fn check_non_empty(args: &[&str]) -> bool {
    let mut result: bool = true;
    for arg in args {
        if arg.is_empty() {
            result = false;
            break;
        }
    }
    result
}

/// Validate conference id with minimum 5 to maximum 15 positive integers
///
/// # Argument
///
/// * `conference_id` - Unique ID of the conference
///
/// # Return
///
/// Return boolean value
pub fn check_conference_id_format(conference_id: &str) -> bool {
    RegexSet::new(&[r"^([0-9]{5,15})$"]).unwrap().is_match(conference_id)
}

/// Validate date with YYYY-MM-DD format
///
/// # Argument
///
/// * `date` - Date of the conference
///
/// # Return
///
/// Returns boolean value
pub fn check_date_format(date: &str) -> bool {
    RegexSet::new(&[r"^(20[0-9]{2})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"]).unwrap().is_match(date)
}

/// Validate email with abc@xyz.pqr format
///
/// # Argument
///
/// * `email` - Email Id of the user
///
/// # Return
///
/// Returns boolean value
pub fn check_email_format(email: &str) -> bool {
    RegexSet::new(&[r"[A-Za-z]+@[A-Za-z]+\.([A-Za-z])"]).unwrap().is_match(email)
}

/// Validate user password with minimum 5 to maximum 15 characters
///
/// # Argument
///
/// * 'password` - Password for user to Login
///
/// # Return
///
/// Returns boolean value
pub fn check_password_format(password: &str) -> bool {
    RegexSet::new(&[r"\b\w{5,20}\b"]).unwrap().is_match(password)
}

/// Validate conference id and user id with minimum 5 to maximum 15 positive integers
///
/// # Arguments
///
/// * `id` - Id of an entity
///
/// # Return
///
/// Returns boolean value
pub fn check_id_format(id: &str) -> bool {
    RegexSet::new(&[r"\b[0-9]{5,15}\b"]).unwrap().is_match(id)
}

/// Validate user name format
///
/// # Argument
///
/// * `name` - Name of the user
///
/// # Return
///
/// Returns boolean value
pub fn check_name_format(name: &str) -> bool {
    RegexSet::new(&[r"\b[a-zA-Z]{2,35}\b"]).unwrap().is_match(name)
}

/// Validate user email-id, password and returns the user type or error,
/// based on the input in the Login form
///
/// # Arguments
///
/// * `email_id` - Email-id entered by the user
///
/// * `password` - Password entered by the user
///
/// # Return
///
/// Returns type of user
pub fn validate_user_details(
    email_id: &str,
    password: &str,
) -> &'static str {
    if check_email_format(email_id) {
        if check_password_format(password) {
            let email = doc! {EMAIL: email_id};
            let users_collection: Collection = connect_database_collection
                (create_db_connection(), DB_NAME, USERS_COLLECTION);
            match users_collection.find_one(Some(email), None).unwrap() {
                Some(email_doc) => {
                    validate_password(email_doc, password)
                }
                None => {
                    error!("A non registered user tried to login with email id as {}", email_id);
                    NON_REGISTERED_USER_MESSAGE
                }
            }
        } else {
            error!("{}", WRONG_PASSWORD_FORMAT);
            WRONG_PASSWORD_FORMAT
        }
    } else {
        error!("{}", WRONG_EMAIL_FORMAT);
        WRONG_EMAIL_FORMAT
    }
}

/// Validate user email-id
///
/// # Arguments
///
/// * `users_collection` - Instance of the Database collection
///
/// * `email_id` - Email-id entered by the user
///
/// # Return
///
/// Returns user message
pub fn validate_user_email(
    email_id: &str
) -> &'static str {
    let users_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, USERS_COLLECTION);
    let email = doc! {EMAIL: email_id};
    match users_collection.find_one(Some(email), None).unwrap() {
        Some(_) => {
            info!("Registered user with email {} accessed forgot password module", email_id);
            REGISTERED_USER_MESSAGE
        }
        None => {
            error!("Non registered user with email {} tried to access forget password module", email_id);
            NON_REGISTERED_USER_MESSAGE
        }
    }
}

/// Validates the password against the entry in the users collection
///
/// # Arguments
///
/// * `email_doc` - Document containing matching E-mail
///
/// * `password` - Password entered by the user
///
/// # Return
///
/// Returns type of user
pub fn validate_password(
    email_doc: bson::ordered::OrderedDocument,
    password: &str,
) -> &'static str {
    let mut encrypted_password: Sha256 = Sha256::new();
    encrypted_password.input_str(password);
    if encrypted_password.result_str().eq(email_doc.get_str(PASSWORD).unwrap())
    {
        check_user_type(email_doc)
    } else {
        error!("{:?} has entered a wrong password", email_doc.get_str(NAME).unwrap());
        WRONG_PASSWORD
    }
}

/// Returns the type of user
///
/// # Arguments
///
/// * `email_doc` - Document containing matching E-mail
///
/// # Return
///
/// Returns type of user
pub fn check_user_type(
    email_doc: bson::ordered::OrderedDocument
) -> &'static str {
    match email_doc.get_str(USER_TYPE).expect(NO_USER_TYPE) {
        USER => {
            info!("User {:?} logged in with email id {:?}", email_doc.get_str(NAME).unwrap(),
                  email_doc.get_str(EMAIL).unwrap());
            USER
        }
        ADMIN => {
            info!("Admin {:?} logged in with email id {:?}", email_doc.get_str(NAME).unwrap(),
                  email_doc.get_str(EMAIL).unwrap());
            ADMIN
        }
        _ => WRONG_USER_TYPE
    }
}

/// Returns the reset password status
///
/// # Arguments
///
/// * `password` - Desired password
///
/// * `confirm_password` - Desired password
///
/// # Return
///
/// Returns password reset response
pub fn validate_reset_password(
    email: &str,
    password: &str,
    confirm_password: &str,
) -> &'static str {
    let email_doc = doc! {EMAIL: email};
    let users_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, USERS_COLLECTION);
    match users_collection.find_one(Some(email_doc), None).unwrap() {
        Some(doc) => {
            if password == confirm_password {
                let mut encrypted_password: Sha256 = Sha256::new();
                encrypted_password.input_str(password);
                let new_password = doc! {PASSWORD: encrypted_password.result_str()};
                let data = doc! {SET => new_password};
                match users_collection.update_one(doc, data, None) {
                    Ok(_) => info!("Password has been reset for email id {}", email),
                    Err(_) => error!("Password has not been reset for email id {} due to MongoDB Error", email)
                }
                RESET_SUCCESSFUL
            } else {
                error!("Password and confirm password field does not match for {}", email);
                RESET_UNSUCCESSFUL
            }
        }
        None => {
            error!("An unregistered user with Email Id {} has tried to reset the password", email);
            NON_REGISTERED_USER_MESSAGE
        }
    }
}

/// Fetches the list of registered conferences for a user
///
/// # Argument
///
/// * `user_id` - User Id
///
/// * `user_email` - User email Id
///
/// * `conference_id` - Conference name to register
///
/// # Return
///
/// Returns Conference registration status
pub fn validate_user_for_conference(
    user_email: &str,
    user_id: &str,
    conference_id: &str,
) -> &'static str {
    let conference_collection: Collection = connect_database_collection(
        create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    let conference_filter = doc! {ID: conference_id};

    match conference_collection.find_one(Some(conference_filter), None).unwrap() {
        Some(_) => {
            let users_collection: Collection = connect_database_collection(
                create_db_connection(), DB_NAME, USERS_COLLECTION);
            let user_filter = doc! {ID: user_id, EMAIL: user_email};

            match users_collection.find_one(Some(user_filter), None).unwrap() {
                Some(_) => {
                    let db_collection: Collection = connect_database_collection(
                        create_db_connection(), DB_NAME, CONFERENCE_DETAILS);

                    let user_detail = doc! {
                        USER_ID: user_id,
                        EMAIL: user_email,
                        STATUS: ABSENT,
                        CONFERENCE_ID: conference_id};
                    let check_user = doc! {USER_ID: user_id, CONFERENCE_ID: conference_id};
                    match db_collection.find_one(Some(check_user), None).unwrap()
                        {
                            Some(_) => {
                                info!("{:?} tried to register for already registered conference {}",
                                      user_email, conference_id);
                                "Conference Already Registered"
                            }
                            None => {
                                match db_collection.insert_one(user_detail, None) {
                                    Ok(_) => {
                                        info!("{:?} registered for {:?} conference", user_email, conference_id);
                                        "Conference Registered"
                                    }
                                    Err(_) => {
                                        info!("{:?} tried to register for {:?} but an unexpected error happened",
                                              user_email, conference_id);
                                        "Failed to Register Conference"
                                    }
                                }
                            }
                        }
                }
                None => {
                    info!("{}", NON_REGISTERED_USER_MESSAGE);
                    NON_REGISTERED_USER_MESSAGE
                }
            }
        }
        None => {
            info!("{}", NO_CONFERENCE);
            NO_CONFERENCE
        }
    }
}

/// Fetches the list of registered conferences for a user
///
/// # Argument
///
/// * `user_email` - User email Id
///
/// # Return
///
/// Returns list of registered conferences for a user
pub fn fetch_conference_for_user(
    user_email: &str
) -> Vec<String> {
    let conference_key = doc! {STATUS: NOT_COMPLETED};
    let db_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    let conference_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, CONFERENCE_DETAILS);
    info!("List of registered conferences fetched for user details module");
    let conference_data: Cursor =
        db_collection.find(Some(conference_key), None).unwrap();
    let mut user_conference_list: Vec<String> = Vec::new();
    for conference_detail in conference_data {
        let conference_document = conference_detail.unwrap();
        let conference_name: String = conference_document.get_str(ID).expect(
            "There is no such field like _id").to_string();
        let user_conference = doc! {EMAIL: user_email,CONFERENCE_ID: conference_name};
        if let Ok(user_conference) =
        conference_collection.find_one(Some(user_conference.clone()), None) {
            if let Some(user_conference) = user_conference {
                let conference = user_conference.get_str(CONFERENCE_ID).expect(
                    "No such field like conf_id exists").to_string();
                user_conference_list.push(conference)
            }
        }
    }
    user_conference_list
}

/// Fetches the list of upcoming conferences
///
/// # Return
///
/// Returns list of upcoming conferences
pub fn fetch_conferences() -> Vec<bson::Document> {
    let conference_key = doc! {STATUS: NOT_COMPLETED};
    let conference_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    info!("List of upcoming conferences fetched for user details module");
    let conference_data: Cursor = conference_collection.find(Some(conference_key),
                                                             None).unwrap();

    let mut conf_data: Vec<bson::Document> = Vec::new();

    for conference_detail in conference_data {
        let conference_document = conference_detail.unwrap();
        conf_data.push(conference_document)
    }
    conf_data
}

/// Deletes the user data from the database
///
/// # Argument
///
/// * `user_id` - Unique User id
///
/// # Return
///
/// Returns confirmation message for the deleted data
pub fn delete_user(
    user_id: String
) -> &'static str {
    let users_collection: Collection =
        connect_database_collection(
            create_db_connection(), DB_NAME, USERS_COLLECTION);
    let data = doc! {ID: user_id.as_str(), USER_TYPE: USER};
    match users_collection.find_one_and_delete(data, None) {
        Ok(document) => match document {
            Some(_) => {
                info!("User with Id {:?} has been deleted by admin", user_id);
                DELETION_SUCCESS
            }
            None => {
                error!("Admin tried to delete a non existing user with user Id {:?}", user_id);
                EMPTY_DATA
            }
        },
        Err(error) => {
            error!("Unable to delete error - {:?} ", error);
            DELETION_FAIL
        }
    }
}


/// Adds a new conference in the database
///
/// # Argument
///
/// * `conference_data` - Record to be added in mongodb collection
///
/// * `validation_data` - Validation
/// # Return
///
/// Returns the confirmation message for the addition of the record
pub fn add_conference(
    conference_data: Document,
    validation_data: Document,
) -> &'static str {
    let collection: Collection =
        connect_database_collection(
            create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    let count: i64 = collection.count(Some(validation_data), None).unwrap();
    match count {
        0 => {
            match collection.insert_one(conference_data, None) {
                Ok(_) => {
                    info!("New conference added");
                    ADDITION_SUCCESS
                }
                Err(err) => {
                    error!("Mongodb error : {} ", err);
                    ADDITION_FAIL
                }
            }
        }
        _ => {
            error!("Admin is trying to create a duplicate conference");
            ALREADY_REGISTERED
        }
    }
}

/// Updates the conference data in the database
///
/// # Argument
///
/// * `filter_data` - Filter record in mongodb collection
///
/// * `updated_conf_data` - Record to be updated in mongodb collection
///
/// # Return
///
/// Returns confirmation message for the updation of the record
pub fn update_conference(
    filter_data: Document,
    updated_conf_data: Document,
) -> &'static str {
    let collection: Collection =
        connect_database_collection(
            create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    match collection.find_one_and_update(filter_data, updated_conf_data, None) {
        Ok(document) => match document {
            Some(_) => {
                info!("A conference was updated");
                UPDATION_SUCCESS
            }
            None => {
                error!("Admin tried to update a non existing conference");
                EMPTY_DATA
            }
        },
        Err(error) => {
            error!("Unable to update error {:?} ", error);
            UPDATION_FAIL
        }
    }
}

/// Deletes the conference data from the database
///
/// # Argument
///
/// * `conf_name` - Record to be deleted from mongodb collection
///
/// # Return
///
/// Returns confirmation message for the deleted data
pub fn delete_conference(
    conf_id: Document
) -> &'static str {
    let collection: Collection =
        connect_database_collection(
            create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    match collection.find_one_and_delete(conf_id, None) {
        Ok(document) => match document {
            Some(_) => {
                info!("Conference was deleted successfully");
                DELETION_SUCCESS
            }
            None => {
                error!("Admin tried to delete a non existing conference");
                EMPTY_DATA
            }
        },
        Err(error) => {
            error!("Unable to update error {:?} ", error);
            DELETION_FAIL
        }
    }
}

/// Filters conferences within a date range
///
/// # Argument
///
/// * `conf_filter_date` - Document containing filter data
///
/// # Return
///
/// Returns the conferences within the date range
pub fn filter_conference(
    conf_filter_date: Document,
) -> String {
    let collection: Collection =
        connect_database_collection(
            create_db_connection(), DB_NAME, CONFERENCE_COLLECTION);
    let cursor: Cursor = collection.find(Some(conf_filter_date), None).unwrap();
    let docs: Vec<_> = cursor
        .map(|doc| doc
            .expect("Error occurred in iterating over documents")
            .get(CONFERENCE_NAME)
            .expect("No such value exists")
            .clone())
        .collect();


    serde_json::to_string(&docs).expect("Unable to serialize into JSON")
}

/// Filters users for a specific conference
///
/// # Argument
///
/// * `collection_name` - Conference Collection
///
/// * `collection_id` - Conference Id
///
/// # Return
///
/// Returns the list of users for a specific conference
pub fn filter_user_conference(
    collection_id: &str
) -> String {
    let conference_collection: Collection =
        connect_database_collection(
            create_db_connection(), DB_NAME, collection_id);
    let cursor: Cursor = conference_collection.find(None, None).unwrap();
    info!("Admin just filtered out list of users for conference {}", collection_id);
    let docs: Vec<_> = cursor
        .map(|doc| doc
            .expect("Error occurred in iterating over documents")
            .get(EMAIL)
            .unwrap()
            .clone())
        .collect();
    serde_json::to_string(&docs)
        .expect("Unable to serialize into JSON")
}

/// Fetches user details for a specific user
///
/// # Argument
///
/// * `user_key` - Document for filtration
///
/// # Return
///
/// Returns all the details about a user
pub fn fetch_user_details(
    user_key: Document
) -> String {
    let users_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, USERS_COLLECTION);
    let user_data: Option<bson::Document> =
        users_collection.find_one(Some(user_key), None).unwrap();
    serde_json::to_string(&user_data).unwrap()
}

/// Updates user details for a specific user
///
/// # Argument
///
/// * `filter_data` - Document for filtration
///
/// * `updated_user_data` - Document with updated data
///
/// # Return
///
/// Returns update status
pub fn update_user_details(
    filter_data: Document,
    updated_user_data: Document,
) -> i32 {
    let users_collection: Collection = connect_database_collection
        (create_db_connection(), DB_NAME, USERS_COLLECTION);
    users_collection.update_one(
        filter_data, updated_user_data, None).unwrap().modified_count
}

/// Checks weather a file path exists on not
///
/// # Argument
///
/// * `path` - Source Path
///
/// # Return
///
/// Returns boolean value
pub fn path_exists(path: &'static str) -> bool {
    Path::new(path).exists()
}

#[cfg(test)]
mod test {
    use mongodb::{bson, doc};
    use mongodb::coll::Collection;
    use mongodb::Document;

    use crate::connection::{connect_database_collection, create_db_connection};
    use crate::constants::{ADMIN, ADMIN_EMAIL, ALREADY_REGISTERED, EMAIL, ID, NAME, NO_CONFERENCE,
                           NON_REGISTERED_USER_MESSAGE, PASSWORD, REGISTERED_USER_MESSAGE, SET,
                           STATUS, TEST_DB_NAME, TEST_EMAIL_ID, TEST_USERS_COLLECTION, UPDATION_SUCCESS,
                           USER, WRONG_EMAIL_FORMAT, WRONG_PASSWORD_FORMAT};
    use crate::request_handlers::admin_handler::ConferenceStatus;
    use crate::utils::{add_conference, check_conference_id_format, check_date_format,
                       check_email_format, check_id_format, check_name_format, check_non_empty,
                       check_password_format, check_user_type, delete_conference, delete_user,
                       EMPTY_DATA, fetch_user_details, filter_conference, filter_user_conference,
                       path_exists, RESET_SUCCESSFUL, RESET_UNSUCCESSFUL, update_conference,
                       update_user_details, validate_password, validate_reset_password,
                       validate_user_details, validate_user_email, validate_user_for_conference, };

    static WRONG_PASSWORD: &str = "Wrong Password";
    static WRONG_USER_TYPE: &str = "User Type is ambiguous, Please contact Admin";

    #[test]
    fn test_check_non_empty_success()
    {
        assert!(check_non_empty(&["Test1", "Test2", "Test3"]))
    }

    #[test]
    fn test_check_non_empty_failure()
    {
        assert!(!check_non_empty(&[""]))
    }

    #[test]
    fn test_check_conference_id_format_success()
    {
        assert!(check_conference_id_format("12345"))
    }

    #[test]
    fn test_check_conference_id_format_failure()
    {
        assert!(!check_conference_id_format("test"))
    }

    #[test]
    fn test_check_date_format_success()
    {
        assert!(check_date_format("2012-03-06"))
    }

    #[test]
    fn test_check_date_format_failure()
    {
        assert!(!check_date_format("06-03-2012"))
    }

    #[test]
    fn test_check_email_format_success()
    {
        assert!(check_email_format("testing@knoldus.in"))
    }

    #[test]
    fn test_check_email_format_failure()
    {
        assert!(!check_email_format("test@testing"))
    }

    #[test]
    fn test_check_password_format_success()
    {
        assert!(check_password_format("test_password"))
    }

    #[test]
    fn test_check_password_format_failure()
    {
        assert!(!check_password_format("test"))
    }

    #[test]
    fn test_check_id_format_success()
    {
        assert!(check_id_format("1234567"))
    }

    #[test]
    fn test_check_id_format_failure()
    {
        assert!(!check_id_format("123"))
    }

    #[test]
    fn test_check_name_format_success()
    {
        assert!(check_name_format("Test Name"))
    }

    #[test]
    fn test_check_name_format_failure()
    {
        assert!(!check_name_format("A"))
    }

    #[test]
    fn test_check_user_type_success_admin()
    {
        let users_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_USERS_COLLECTION);
        let email = doc! {EMAIL: ADMIN_EMAIL};
        let email_doc = users_collection.find_one(Some(email), None).unwrap().unwrap();
        assert_eq!(check_user_type(email_doc), ADMIN);
    }

    #[test]
    fn test_check_user_type_success_user()
    {
        let users_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_USERS_COLLECTION);
        let email = doc! {EMAIL: TEST_EMAIL_ID};
        let email_doc = users_collection.find_one(Some(email), None).unwrap().unwrap();
        assert_eq!(check_user_type(email_doc), USER);
    }

    #[test]
    fn test_check_user_type_failure()
    {
        let users_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_USERS_COLLECTION);
        let email = doc! {EMAIL: "test.testing@knoldus.in"};
        let email_doc = users_collection.find_one(Some(email), None).unwrap().unwrap();
        assert_eq!(check_user_type(email_doc), WRONG_USER_TYPE);
    }

    #[test]
    fn test_validate_password_success()
    {
        let users_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_USERS_COLLECTION);
        let email = doc! {EMAIL: ADMIN_EMAIL};
        let email_doc = users_collection.find_one(Some(email), None).unwrap().unwrap();
        assert_eq!(validate_password(email_doc, ADMIN), ADMIN);
    }

    #[test]
    fn test_validate_password_failure()
    {
        let users_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_USERS_COLLECTION);
        let email = doc! {EMAIL: ADMIN_EMAIL};
        let email_doc = users_collection.find_one(Some(email), None).unwrap().unwrap();
        assert_eq!(validate_password(email_doc, PASSWORD), WRONG_PASSWORD);
    }

    #[test]
    fn test_validate_user_details_success()
    {
        assert_eq!(validate_user_details(ADMIN_EMAIL, ADMIN), ADMIN);
    }

    #[test]
    fn test_validate_user_details_failure()
    {
        assert_eq!(validate_user_details("wrong@email.com", "pword"),
                   NON_REGISTERED_USER_MESSAGE);
    }

    #[test]
    fn test_validate_user_details_email_failure()
    {
        assert_eq!(validate_user_details(
            "wrong@email", "pword"), WRONG_EMAIL_FORMAT);
    }

    #[test]
    fn test_validate_user_details_password_failure()
    {
        assert_eq!(validate_user_details("wrong@email.com", "pwo"),
                   WRONG_PASSWORD_FORMAT);
    }

    #[test]
    fn test_validate_reset_password_success()
    {
        assert_eq!(RESET_SUCCESSFUL, validate_reset_password("test@knoldus.in",
                                                             "test123",
                                                             "test123"))
    }

    #[test]
    fn test_validate_reset_password_match_failure()
    {
        assert_eq!(RESET_UNSUCCESSFUL, validate_reset_password("test@knoldus.in",
                                                               "test123",
                                                               "test12345"))
    }

    #[test]
    fn test_validate_reset_password_failure()
    {
        assert_eq!(NON_REGISTERED_USER_MESSAGE, validate_reset_password("test1234@knoldus.in",
                                                                        "test123",
                                                                        "test123"))
    }

    #[test]
    fn test_validate_user_email_success()
    {
        assert_eq!(REGISTERED_USER_MESSAGE,
                   validate_user_email("test@knoldus.in"))
    }

    #[test]
    fn test_validate_user_email_failure()
    {
        assert_eq!(NON_REGISTERED_USER_MESSAGE,
                   validate_user_email("test1234@knoldus.in"))
    }

    #[test]
    fn test_validate_user_for_conference_success()
    {
        assert_eq!("Conference Already Registered", validate_user_for_conference(
            "test@knoldus.in", "1111111111", "5544332211"))
    }

    #[test]
    fn test_validate_user_for_conference_user_failure()
    {
        assert_eq!(NON_REGISTERED_USER_MESSAGE, validate_user_for_conference(
            "test@knoldus.in", "11111111", "5544332211"))
    }


    #[test]
    fn test_validate_user_for_conference_failure()
    {
        assert_eq!(NO_CONFERENCE, validate_user_for_conference(
            "test@knoldus.in", "1111111111", "55432211"))
    }

    #[test]
    fn test_delete_user_failure()
    {
        assert_eq!(delete_user("abcdefg".to_string()), EMPTY_DATA)
    }

    #[test]
    fn test_add_conference_failure()
    {
        let validation_data: Document = doc! {
                        "_id" :"5544332211",
                };

        let conference_data: Document = doc! {
                    "_id" : "5544332211",
                    "conf_name" : "test_conference",
                    "conference_data" : "2019-02-02",
                    "conference_address1" : "test_address_1",
                    "conference_address2" : "test_address_2",
                    "conference_address3" : "test_address_3",
                    STATUS: ConferenceStatus::NotCompleted.as_str(),
            };
        assert_eq!(ALREADY_REGISTERED, add_conference(conference_data, validation_data))
    }

    #[test]
    fn test_update_conference_success()
    {
        let filter_data: Document = doc! {
                        "_id" : "5544332211",
                };

        let updated_conference_data: Document = doc! {
                SET : {
                "conference_date" : "2018-06-24",
                "conference_address1" : "test_address_1",
                "conference_address2" : "test_address_2",
                "conference_address3" : "test_address_3",
                }
            };
        assert_eq!(UPDATION_SUCCESS, update_conference(filter_data, updated_conference_data))
    }

    #[test]
    fn test_update_conference_failure()
    {
        let filter_data: Document = doc! {
                        "_id" : "55443311",
                };

        let updated_conference_data: Document = doc! {
                SET : {
                "conference_date" : "2018-06-24",
                "conference_address1" : "test_address_1",
                "conference_address2" : "test_address_2",
                "conference_address3" : "test_address_3",
                }
            };
        assert_eq!(EMPTY_DATA, update_conference(filter_data, updated_conference_data))
    }

    #[test]
    fn test_delete_conference_failure()
    {
        let filter_data: Document = doc! {
                        "_id" : "55443311", };
        assert_eq!(EMPTY_DATA, delete_conference(filter_data))
    }

    #[test]
    fn test_filter_conference_success()
    {
        let filter = doc! {"conference_date": {"$gte": "2017-06-24","$lte": "2017-06-26",},};
        assert_eq!(filter_conference(filter), "[]".to_string())
    }

    #[test]
    fn test_filter_user_conference_success()
    {
        assert_eq!(filter_user_conference("5544332211"), "[]")
    }

    #[test]
    fn test_fetch_user_details_success()
    {
        let filter_data = doc! {"email" : "test@tester.in"};
        assert_eq!(fetch_user_details(filter_data), "null")
    }

    #[test]
    fn test_update_user_details_success()
    {
        let user_id = doc! {ID: "2839448279"};
        let updated_data = doc! {SET: {NAME: "Test",EMAIL: "test@knoldus.in"}};

        assert_eq!(update_user_details(user_id, updated_data), 0)
    }

    #[test]
    fn test_path_exists_success()
    {
        assert!(path_exists(
            "/home/ajsuper47/Hawk-Conf-Count/tests/resources/test.txt"))
    }

    #[test]
    fn test_path_exists_failure()
    {
        assert!(!path_exists(
            "/home/ajsuper47/Hawk-Conf-Count/tests/resources/teerst.txt"))
    }
}
