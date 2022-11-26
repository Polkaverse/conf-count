use futures::future::Future;
use futures::sync::oneshot::spawn;
use log::error;
use rusoto_rekognition::{CompareFacesRequest, Rekognition, RekognitionClient};
use tokio::runtime::Runtime;

use crate::constants::{DIFFERENT, SIMILAR};

static AWS_SERVER_ERROR: &str = "Image key not found in s3 bucket";

/// Compare two images and return their status(Similar or not)
///
/// # Arguments
///
/// * `rekognition_client` - This is the client of Amazon Rekognition API
///
/// * `compare_faces_request` - This is the instance of the CompareFacesRequest
///
/// # Return
///
/// Returns the success or failure of image comparison
pub fn compare_images(
    rekognition_client: &RekognitionClient,
    compare_faces_request: CompareFacesRequest,
) -> Result<&'static str, &'static str> {
    spawn(rekognition_client.compare_faces(compare_faces_request)
              .map(|response| {
                  if !response.face_matches.expect("Empty payload").is_empty() {
                      SIMILAR
                  } else {
                      DIFFERENT
                  }
              })
              .map_err(|error| {
                  error!("{}", error);
                  AWS_SERVER_ERROR
              })
          , &Runtime::new()
            .expect("Tokio is not working")
            .executor())
        .wait()
}

#[cfg(test)]
mod test {
    use std::fs;

    use rusoto_core::Region;
    use rusoto_rekognition::{CompareFacesRequest, Image, RekognitionClient, S3Object};

    use crate::constants::{DIFFERENT, SIMILAR};
    use crate::image_comparison::compare_images;

    static BUCKET_NAME: &str = "labelsfacedetect";
    static KEY: &str = "1122334455";
    const SIMILARITY_THRESHOLD: f32 = 75.0;
    static TEST_IMAGE_FILE: &str = "tests/resources/test.jpg";
    static DIFFERENT_TEST_IMAGE_FILE: &str = "tests/resources/test1.jpg";
    static SIMILAR_TEST_IMAGE_FILE: &str = "tests/resources/test2.jpg";
    static TEST_TEXT_FILE: &str = "tests/resources/test.txt";

    fn get_utilities(
        source_image: Image,
        target_image: Image,
    ) -> (RekognitionClient, CompareFacesRequest) {
        (RekognitionClient::new(Region::ApSouth1),
         CompareFacesRequest {
             similarity_threshold: Some(SIMILARITY_THRESHOLD),
             source_image,
             target_image,
         }
        )
    }

    #[test]
    fn test_compare_images_success() {
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
            bytes: Some(fs::read(SIMILAR_TEST_IMAGE_FILE).unwrap()),
            s3_object: None,
        };

        let utilities: (RekognitionClient, CompareFacesRequest) =
            get_utilities(source_image, target_image);
        assert_eq!(SIMILAR, compare_images(
            &utilities.0, utilities.1).unwrap())
    }

    #[test]
    fn test_compare_images_failure() {
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
            bytes: Some(fs::read(DIFFERENT_TEST_IMAGE_FILE).unwrap()),
            s3_object: None,
        };
        let utilities: (RekognitionClient, CompareFacesRequest) =
            get_utilities(source_image, target_image);
        assert_eq!(DIFFERENT, compare_images(
            &utilities.0, utilities.1).unwrap())
    }

    #[test]
    fn test_compare_images_error() {
        let source_image = Image {
            bytes: Some(fs::read(TEST_IMAGE_FILE).unwrap()),
            s3_object: None,
        };
        let target_image = Image {
            bytes: Some(fs::read(TEST_TEXT_FILE).unwrap()),
            s3_object: None,
        };
        let utilities: (RekognitionClient, CompareFacesRequest) =
            get_utilities(source_image, target_image);
        assert!(compare_images(&utilities.0, utilities.1).is_err())
    }
}
