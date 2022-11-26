use std::env;

use log::error;
use mongodb::{Client, ThreadedClient};
use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;
use mongodb::Document;
use rusoto_rekognition::RekognitionClient;
use s3::{bucket::Bucket, credentials::Credentials};

use crate::constants::DB_SUCCESS_MESSAGE;

static DB_FAILURE_MESSAGE: &str = "Unable to insert data";
static HOST: &str = "Host";
static PORT: u16 = 27017;
static BUCKET_NAME: &str = "Clicked_Image_Bucket";
static REGION: &str = "Region";

/// This function inserts the data into database
///
/// # Arguments
///
/// * `collection` - This is the instance of the Database collection
///
/// * `data` - This data is in the bson format(Alias for `OrderedDocument)
///
/// # Return
///
/// This function returns the success or failure message of insert operation
pub fn insert_data(collection: Collection, data: Document) -> &'static str {
    match collection.insert_one(data, None) {
        Ok(_) => DB_SUCCESS_MESSAGE,
        Err(error) => {
            error!("Unable to insert data {:?} ", error);
            DB_FAILURE_MESSAGE
        }
    }
}

/// Creates a connection with the collection of database
///
/// # Arguments
///
/// * `client` - This is the instance of the Database client
///
/// # Return
///
/// Returns instance of the Database collection
pub fn connect_database_collection(client: Client, database: &str, collection: &str) -> Collection {
    client.db(database).collection(collection)
}

/// Establishes connection with MongoDB
///
/// # Return
///
/// Returns client of MongoDB
pub fn create_db_connection() -> Client {
    Client::connect(env::var(
        HOST).expect("Host not exported").as_str(), PORT).expect(
        "Database not ready")
}

/// The function create_rekognition_connection establishes connection with RekognitionClient
///
/// # Return
///
///  This function returns client of RekognitionClient
pub fn create_rekognition_connection() -> RekognitionClient {
    RekognitionClient::new(env::var(REGION)
        .expect("Region not exported").parse().unwrap())
}

/// The function create_bucket_connection establishes connection with S3 bucket and returns the bucket instance
///
/// # Return
///
///  This function returns instance of S3 Bucket
pub fn create_bucket_connection() -> Bucket {
    Bucket::new(
        env::var(BUCKET_NAME).expect("Bucket Name not exported").as_str(),
        env::var(REGION).expect("Region not exported").parse().unwrap(),
        Credentials::default(),
    )
}

#[cfg(test)]
mod test {
    use mongodb::{Client, ThreadedClient};
    use mongodb::coll::Collection;
    use mongodb::db::ThreadedDatabase;
    use rusoto_rekognition::{ListCollectionsRequest, Rekognition, RekognitionClient};
    use s3::bucket::Bucket;
    use s3::credentials::Credentials;
    use s3::region::Region;

    use crate::connection::{connect_database_collection, create_db_connection};
    use crate::connection::{create_bucket_connection, create_rekognition_connection};

    static TEST_HOST: &str = "localhost";
    const TEST_PORT: u16 = 27017;
    static TEST_DB_NAME: &str = "conf_db";
    static TEST_COLLECTION_NAME: &str = "conf_count";
    const TEST_BUCKET_NAME: &str = "labelsfacedetect";
    const REGION: Region = Region::ApSouth1;

    #[test]
    fn test_create_db_connection() {
        assert_eq!(0, create_db_connection().get_req_id())
    }

    #[test]
    fn test_connect_database_collection() {
        let client: Client = Client::connect(TEST_HOST, TEST_PORT).unwrap();
        let collection: Collection = client.db(TEST_DB_NAME).collection(TEST_COLLECTION_NAME);
        assert_eq!(collection.name(), connect_database_collection(client, TEST_DB_NAME,
                                                                  TEST_COLLECTION_NAME).name())
    }

    #[test]
    fn test_create_rekognition_connection_success() {
        let client: RekognitionClient = create_rekognition_connection();
        let request: ListCollectionsRequest = ListCollectionsRequest::default();
        assert!(client.list_collections(request).sync().is_ok())
    }

    #[test]
    fn test_create_bucket_connection_success() {
        let credentials: Credentials = Credentials::default();
        let bucket: Bucket = Bucket::new(TEST_BUCKET_NAME, REGION, credentials);
        assert_eq!(bucket, create_bucket_connection());
    }

    #[test]
    fn test_create_bucket_connection_failure() {
        let credentials: Credentials = Credentials::default();
        let bucket: Bucket = Bucket::new("invalid_bucket", REGION, credentials);
        assert_ne!(bucket, create_bucket_connection())
    }
}