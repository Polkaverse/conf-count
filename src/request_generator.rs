use std::{env, fs};

use rusoto_rekognition::{CompareFacesRequest, Image, S3Object};

use crate::constants::SIMILARITY_THRESHOLD;

static BUCKET_NAME: &str = "Clicked_Image_Bucket";

/// Creates a request for comparing two images in Amazon Rekognition API
///
/// # Arguments
///
/// * `user_id` - Id of the user
///
/// * `target_image_path` - Path of the target image
///
/// # Return
///
/// Returns the instance of the CompareFacesRequest
pub fn create_compare_faces_request(
    user_id: &str,
    target_image_path: &str,
) -> CompareFacesRequest {
    let source_image = Image {
        bytes: None,
        s3_object: Some(
            S3Object {
                bucket: Some(env::var(BUCKET_NAME)
                    .expect("Bucket Name not exported").to_string()),
                name: Some(user_id.to_string()),
                version: None,
            }
        ),
    };

    let target_image = Image {
        bytes: Some(fs::read(target_image_path)
            .expect("Target Image Path not exported")),
        s3_object: None,
    };

    CompareFacesRequest {
        similarity_threshold: Some(SIMILARITY_THRESHOLD),
        source_image,
        target_image,
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use rusoto_rekognition::{CompareFacesRequest, Image, S3Object};

    use crate::request_generator::create_compare_faces_request;

    use super::SIMILARITY_THRESHOLD;

    static BUCKET_NAME: &str = "labelsfacedetect";
    static KEY: &str = "1122334455";
    static TARGET_IMAGE_PATH: &str = "tests/resources/test.jpg";
    static DIFFERENT_SOURCE_IMAGE_PATH: &str = "tests/resources/test1.jpg";

    #[test]
    fn test_create_compare_faces_request_success() {
        let source_image = Image {
            bytes: None,
            s3_object: Some(
                S3Object {
                    bucket: Some(BUCKET_NAME.to_string()),
                    name: Some(KEY.to_string()),
                    version: None,
                }
            ),
        };
        let target_image = Image {
            bytes: Some(fs::read(TARGET_IMAGE_PATH).unwrap()),
            s3_object: None,
        };
        let compare_faces_request: CompareFacesRequest = CompareFacesRequest {
            similarity_threshold: Some(SIMILARITY_THRESHOLD),
            source_image,
            target_image,
        };
        assert_eq!(compare_faces_request,
                   create_compare_faces_request(
                       KEY, TARGET_IMAGE_PATH))
    }

    #[test]
    fn test_create_compare_faces_request_failure() {
        let source_image = Image {
            bytes: None,
            s3_object: Some(
                S3Object {
                    bucket: Some(BUCKET_NAME.to_string()),
                    name: Some(KEY.to_string()),
                    version: None,
                }
            ),
        };

        let target_image = Image {
            bytes: Some(fs::read(DIFFERENT_SOURCE_IMAGE_PATH).unwrap()),
            s3_object: None,
        };
        let compare_faces_request: CompareFacesRequest = CompareFacesRequest {
            similarity_threshold: Some(SIMILARITY_THRESHOLD),
            source_image,
            target_image,
        };

        assert_ne!(compare_faces_request,
                   create_compare_faces_request(
                       KEY, TARGET_IMAGE_PATH))
    }
}
