use std::env;
use std::process::Command;

use log::error;

static CAMERA_SHUTTER_SPEED: &str = "3000";
static INVALID_CLICKED_IMAGE: &str = "Unable to capture image";
static RASPISTILL: &str = "raspistill";
static CLICKED_IMAGE_PATH: &str = "Clicked_Image_Path";
static QUALITY: &str = "100";
static CONTRAST: &str = "50";
static SHARPNESS: &str = "30";
static BRIGHTNESS: &str = "60";

/// Triggers camera
///
/// # Return
///
/// Returns camera image
pub fn trigger_camera() -> Result<String, &'static str> {
    let clicked_image_path: String = env::var(CLICKED_IMAGE_PATH)
        .expect("Clicked Image path not not exported");

    match Command::new(RASPISTILL)
        .args(&[
            "-t",
            CAMERA_SHUTTER_SPEED,
            "-q",
            QUALITY,
            "-co",
            CONTRAST,
            "-sh",
            SHARPNESS,
            "-br",
            BRIGHTNESS,
            "-o",
            clicked_image_path.as_str(),
        ])
        .output()
        {
            Ok(_) => Ok(clicked_image_path),
            Err(error) => {
                error!("{}", error);
                Err(INVALID_CLICKED_IMAGE)
            }
        }

    //  Ok(clicked_image_path)
}

#[cfg(test)]
mod test {
    use crate::user_data::{INVALID_CLICKED_IMAGE, trigger_camera};

    #[test]
    fn test_camera_invalid_image() {
        assert_eq!(trigger_camera().unwrap_err(), INVALID_CLICKED_IMAGE)
    }
}
