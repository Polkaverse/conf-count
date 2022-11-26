use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use log::error;

static SUBJECT: &str = "Conference Mail";
static BODY: &str = "You were absent";
static SUCCESS: &str = "Email Sent";
static FAILURE: &str = "Invalid Email";

/// The function send_email sends E-mail to the absent participants
///
/// # Arguments
///
/// * `sender` - This is the sender's E-mail
///
/// * `receiver` - This is the receiver's E-mail
///
/// # Return
///
///  This function returns Success or Failure message for the E-mail
pub fn send_email(sender: &str, receiver: &str) -> &'static str {
    match Email::builder()
        .to(receiver)
        .from(sender)
        .subject(SUBJECT)
        .body(BODY)
        .build() {
        Ok(email) => {
            let mailer: SmtpClient = SmtpClient::new_unencrypted_localhost()
                .expect("Something's not right with SMTP Client");
            mailer.transport().send(email.into())
                .expect("Something's not right with E-mail service");
            SUCCESS
        }
        Err(error) => {
            error!("{}", error);
            FAILURE
        }
    }
}

#[cfg(test)]
mod test {
    use crate::email_service::{FAILURE, send_email, SUCCESS};

    static SENDER: &str = "alok.jha@knoldus.in";
    static INVALID_SENDER: &str = "pankaj@gmail.com";
    static RECEIVER: &str = "pankaj.chaudhary@knoldus.in";

    #[test]
    fn test_send_email_success() {
        assert_eq!(send_email(
            SENDER, RECEIVER), SUCCESS);
    }

    #[test]
    fn test_send_email_failure() {
        assert_eq!(send_email(
            INVALID_SENDER, RECEIVER), FAILURE);
    }
}
