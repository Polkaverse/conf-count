use log::info;
use mongodb::coll::Collection;

use crate::connection::create_rekognition_connection;
use crate::constants::{COLLECTION_EMPTY, PROCESS_COMPLETE};
use crate::db_operations::{fetch_user_ids, update_db};

/// Returns the response based on the bucket values
///
/// # Arguments
///
/// * `conference_collection` - Instance of the Database collection
///
/// # Return
///
/// Returns response message for the db update process
pub fn fetch_response(conference_collection: Collection, conference_id: &str) -> &'static str
{
    let res: Vec<String> = fetch_user_ids(&conference_collection, conference_id);
    match res.len() {
        0 => COLLECTION_EMPTY,
        _ => {
            for user_id in res {
                let response: &str =
                    update_db(&create_rekognition_connection(),
                              &conference_collection,
                              user_id.clone());
                info!("{} - {}", response, user_id.clone());
            }
            PROCESS_COMPLETE
        }
    }
}

#[cfg(test)]
mod test {
    use mongodb::coll::Collection;

    use crate::connection::{connect_database_collection, create_db_connection};
    use crate::constants::{COLLECTION_EMPTY, PROCESS_COMPLETE};
    use crate::response_service::fetch_response;

    static TEST_DB_NAME: &str = "Conf_Count";
    static TEST_EMPTY_CONFERENCE_COLLECTION: &str = "conf";
    static TEST_CONFERENCE_COLLECTION: &str = "conference_details";

    #[test]
    fn test_fetch_response_empty()
    {
        let conference_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_EMPTY_CONFERENCE_COLLECTION);
        assert_eq!(COLLECTION_EMPTY, fetch_response(conference_collection, "test_conference"))
    }

    #[test]
    fn test_fetch_response_success()
    {
        let conference_collection: Collection = connect_database_collection
            (create_db_connection(), TEST_DB_NAME, TEST_CONFERENCE_COLLECTION);
        assert_eq!(PROCESS_COMPLETE, fetch_response(conference_collection, "5544332211"))
    }
}