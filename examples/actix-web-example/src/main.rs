// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::sync::Arc;

use actix_web::{middleware, App, HttpServer, Responder};
use shenyu_client_rust::{actix_web_impl::ShenYuRouter, IRouter};
use shenyu_client_rust::ci::_CI_CTRL_C;
use shenyu_client_rust::config::ShenYuConfig;
use shenyu_client_rust::core::ShenyuClient;
use shenyu_client_rust::{register_once, shenyu_router};
use tokio::sync::Mutex;

async fn health_handler() -> impl Responder {
    "OK"
}

async fn create_user_handler() -> impl Responder {
    "User created"
}

async fn index() -> impl Responder {
    "Welcome!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Spawn a thread to listen for Ctrl-C events and shutdown the server
    std::thread::spawn(_CI_CTRL_C);
    // Initialize tracing
    tracing_subscriber::fmt::init();

    HttpServer::new(move || {
        let mut router = ShenYuRouter::new("shenyu_client_app");
        let mut app = App::new().wrap(middleware::Logger::default());
        let config = ShenYuConfig::from_yaml_file("shenyu-client-rust/config.yml").unwrap();
        // fixme the handler method name, should be `actix-web-example::health_handler`
        shenyu_router!(
            router,
            app,
            "/health" => get(health_handler)
            "/create_user" => post(create_user_handler)
            "/" => get(index)
        );
        let app_name = router.app_name();
        let routers = router.uri_infos();
        // register_once!(config, router, 4000);
        let mut client = {
            let res = ShenyuClient::new(
                config,
                app_name,
                routers,
                4000,
            );
            res.map(|t|Mutex::new(t)).unwrap()
        };
        client.get_mut().register().expect("Failed to register");
        let client = Arc::new(client).clone();
        actix_web::rt::spawn(async move {
            // Add shutdown hook
            tokio::select! {
                _ = actix_web::rt::signal::ctrl_c() => {
                    client.lock().await.offline_register();
                }
            }
        });

        app
    })
    .bind(("0.0.0.0", 4000))
    .expect("Can not bind to 4000")
    .run()
    .await
}
