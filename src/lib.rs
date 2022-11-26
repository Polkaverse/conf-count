#[macro_use]
extern crate serde_derive;

pub mod request_handlers {
    pub mod login_handler;
    pub mod signup_handler;
    pub mod user_details_handler;
    pub mod conference_handler;
    pub mod camera_handler;
    pub mod admin_handler;
    pub mod forgot_password_handler;
    pub mod reset_password_handler;
    pub mod logout_handler;
}

pub mod connection;

pub mod constants;

pub mod utils;

pub mod request_generator;

pub mod image_comparison;

pub mod db_operations;

pub mod user_data;

pub mod email_service;

pub mod response_service;

pub mod user_data_upload_operations;
