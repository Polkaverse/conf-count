use actix_web::{AsyncResponder, Error, Form, HttpResponse};
use futures::Future;
use futures::future::result;
use log::error;
use serde_json::json;

use crate::constants::{RESPONSE, WRONG_EMAIL_FORMAT, WRONG_PASSWORD_FORMAT};
use crate::utils::{check_email_format, check_password_format, validate_reset_password};

#[derive(Deserialize)]
pub struct PasswordData {
    email: String,
    password: String,
    confirm_password: String,
}

/// Returns password reset message
///
/// # Argument
///
/// * `password_data` - Email, Password and Confirm Password
///
/// # Return
///
/// Returns the password reset message
pub fn handle_reset_password(
    password_data: Form<PasswordData>
) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let response: &str;
    if check_email_format(password_data.email.as_str()) {
        if check_password_format(password_data.password.as_str()) &&
            check_password_format(password_data.confirm_password.as_str()) {
            response = validate_reset_password(password_data.email.as_str(),
                                               password_data.password.as_str(),
                                               password_data.confirm_password.as_str())
        } else {
            error!("{}", WRONG_PASSWORD_FORMAT);
            response = WRONG_PASSWORD_FORMAT;
        }
    } else {
        error!("{}", WRONG_EMAIL_FORMAT);
        response = WRONG_EMAIL_FORMAT;
    }
    result(Ok(HttpResponse::Ok()
        .json(json!({RESPONSE: response}))))
        .responder()
}

#[cfg(test)]
mod test {
    use actix_web::Form;
    use futures::future::Future;

    use crate::constants::{EMAIL, PASSWORD};
    use crate::request_handlers::reset_password_handler::{handle_reset_password, PasswordData};

    #[test]
    fn test_handle_reset_password_wrong_email_format()
    {
        let password_data = PasswordData
            {
                email: EMAIL.to_string(),
                password: PASSWORD.to_string(),
                confirm_password: PASSWORD.to_string(),
            };
        assert!(handle_reset_password(Form(password_data)).wait().is_ok());
    }

    #[test]
    fn test_handle_reset_password_wrong_password_format()
    {
        let password_data = PasswordData
            {
                email: EMAIL.to_string(),
                password: "abcd".to_string(),
                confirm_password: PASSWORD.to_string(),
            };
        assert!(handle_reset_password(Form(password_data)).wait().is_ok());
    }

    #[test]
    fn test_handle_reset_password_success()
    {
        let password_data = PasswordData
            {
                email: "test@knoldus.in".to_string(),
                password: PASSWORD.to_string(),
                confirm_password: PASSWORD.to_string(),
            };
        assert!(handle_reset_password(Form(password_data)).wait().is_ok());
    }
}
