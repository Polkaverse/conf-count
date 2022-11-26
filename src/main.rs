
use actix_web::{App, middleware, server};
use actix_web::http::Method;

use conf_count::request_handlers::admin_handler::{handle_admin_timeout,
                                                       handle_conference_addition,
                                                       handle_conference_deletion,
                                                       handle_conference_filtration,
                                                       handle_conference_updation,
                                                       handle_user_deletion,
                                                       handle_user_filtration, handle_user_updation,
                                                       load_admin_dashboard};
use conf_count::request_handlers::camera_handler::handle_camera;
use conf_count::request_handlers::conference_handler::{fetch_conference_details,
                                                            fetch_registered_conferences};
use conf_count::request_handlers::forgot_password_handler::handle_forgot_password;
use conf_count::request_handlers::login_handler::{handle_login, load_login_form};
use conf_count::request_handlers::logout_handler::handle_logout;
use conf_count::request_handlers::reset_password_handler::handle_reset_password;
use conf_count::request_handlers::signup_handler::{handle_user_signup, load_registration_form};
use conf_count::request_handlers::user_details_handler::{handle_conference_registration,
                                                              handle_user_details,
                                                              handle_user_details_updation};

static SOCKET: &str = "127.0.0.1:8088";
static LOGIN: &str = "/login";
static USER_DETAILS: &str = "/user-details";
static CONFERENCE_DETAILS: &str = "/conference-details";
static USER_CONFERENCE_DETAILS: &str = "/user-conference-details";
static LOGOUT: &str = "/logout";
static FORGOT_PASSWORD: &str = "/forgot";
static RESET_PASSWORD: &str = "/reset-password";
static TRIGGER_CAMERA: &str = "/trigger-camera";
static ADMIN: &str = "/admin";
static ADMIN_USER: &str = "/admin-user";
static ADMIN_CONFERENCE: &str = "/admin-conference";
static FILTER_CONFERENCE: &str = "/filter-conference";
static FILTER_USER: &str = "/filter-user";
static SESSION_TIMEOUT: &str = "/session-timeout";
static SIGNUP: &str = "/signup";

/// This is a web application for Conference Records
#[cfg_attr(tarpaulin, skip)]
fn main() {
    env_logger::init();
    server::new(
        || App::new()
            .middleware(middleware::Logger::default())
            .resource(
                LOGIN, |resource| {
                    resource.method(Method::GET).f(load_login_form);
                    resource.method(Method::POST).with(handle_login);
                })
            .resource(
                USER_DETAILS, |resource| {
                    resource.method(Method::GET).f(handle_user_details);
                    resource.method(Method::PUT).f(handle_user_details_updation);
                    resource.method(Method::POST).f(handle_conference_registration);
                })
            .resource(
                CONFERENCE_DETAILS, |resource| {
                    resource.method(Method::GET).f(fetch_conference_details);
                })
            .resource(
                USER_CONFERENCE_DETAILS, |resource| {
                    resource.method(Method::GET).f(fetch_registered_conferences);
                })
            .resource(
                LOGOUT, |response| {
                    response.method(Method::GET).f(handle_logout);
                })
            .resource(
                FORGOT_PASSWORD, |response| {
                    response.method(Method::POST).f(handle_forgot_password);
                })
            .resource(
                RESET_PASSWORD, |response| {
                    response.method(Method::POST).with(handle_reset_password);
                })
            .resource(
                TRIGGER_CAMERA, |resource| {
                    resource.method(Method::GET).f(handle_camera);
                })

            .resource(ADMIN, |resource| {
                resource.method(Method::GET).f(load_admin_dashboard);
            })
            .resource(ADMIN_USER, |resource| {
                resource.method(Method::DELETE).with(handle_user_deletion);
                resource.method(Method::PUT).f(handle_user_updation);
            })
            .resource(ADMIN_CONFERENCE, |resource| {
                resource.method(Method::DELETE).with(handle_conference_deletion);
                resource.method(Method::PUT).with(handle_conference_updation);
                resource.method(Method::POST).with(handle_conference_addition);
            })
            .resource(FILTER_CONFERENCE, |resources| {
                resources.method(Method::POST).with(handle_conference_filtration);
            })
            .resource(FILTER_USER, |resources| {
                resources.method(Method::POST).with(handle_user_filtration);
            })
            .resource(SESSION_TIMEOUT, |resources| {
                resources.method(Method::GET).f(handle_admin_timeout);
            })
            .resource(SIGNUP, |resource| {
                resource.method(Method::GET).f(load_registration_form);
                resource.method(Method::POST).f(handle_user_signup);
            })
    )
        .bind(SOCKET).unwrap()
        .run();
}
