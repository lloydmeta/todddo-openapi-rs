#![feature(async_await)]

pub mod handlers {
    pub mod todo_routes_handler;
}

pub mod controllers {
    pub mod todo_controller;
}

pub mod models {
    pub mod common;
    pub mod todo;
}

use crate::controllers::todo_controller;
use crate::controllers::todo_controller::TodoControllerImpl;
use actix_web::middleware::Logger;
use actix_web::*;
use domain::services::todo_service;
use domain::services::todo_service::TodoServiceImpl;
use handlers::todo_routes_handler;
use infra::in_mem::todo_repo;
use infra::in_mem::todo_repo::InMemTodoRepo;
use log::*;
use paperclip::actix::{
    // use this instead of actix_web::web
    web,
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt,
};

static WEB_BIND_ADDR_KEY: &str = "WEB_BIND_ADDR";

// This allows us to use a generated (via build.rs) file
// that bakes these static files into our binary.
use std::collections::HashMap;
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn run_server() -> Result<(), std::io::Error> {
    let todo_repo = todo_repo::new();
    type Controller = TodoControllerImpl<TodoServiceImpl<InMemTodoRepo>>;
    let server = HttpServer::new(move || {
        let todo_service = todo_service::new(todo_repo.clone());
        let todo_controller = todo_controller::new(todo_service);
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::Compress::default())
            .data(todo_controller)
            .service(actix_web_static_files::ResourceFiles::new(
                "/swagger",
                generate(),
            ))
            .wrap_api()
            .with_json_spec_at("/api/spec")
            .route(
                "/tasks",
                web::get().to_async(todo_routes_handler::list::<Controller>),
            )
            .route(
                "/tasks",
                web::post().to_async(todo_routes_handler::create::<Controller>),
            )
            .route(
                "/tasks/{id}",
                web::get().to_async(todo_routes_handler::get::<Controller>),
            )
            .route(
                "/tasks/{id}",
                web::delete().to_async(todo_routes_handler::delete::<Controller>),
            )
            .route(
                "/tasks/{id}",
                web::put().to_async(todo_routes_handler::update::<Controller>),
            )
            .build()
    });

    let bind_to = std::env::var(WEB_BIND_ADDR_KEY).unwrap_or("127.0.0.1:8080".to_string());
    info!(
        "Binding to [{}], change by setting the {} env var.",
        bind_to, WEB_BIND_ADDR_KEY
    );
    Ok(server.bind(bind_to)?.run()?)
}
