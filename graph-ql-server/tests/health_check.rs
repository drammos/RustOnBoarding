// use std::net::TcpListener;

// use graph_ql_server::{updown::startup::run, CONFIG};

// #[tokio::test]
// async fn health_check_works() {
//     let address = spawn_app().await;
//     let client = reqwest::Client::new();

//     let response = client
//         .get(&format!("{}/health-check", &address))
//         .send()
//         .await
//         .expect("Failed to execute request.");

//     assert!(response.status().is_success());
//     assert_eq!(
//         "Service is healthy".to_string(),
//         response.text().await.unwrap()
//     );
// }

// async fn spawn_app() -> String {
//     let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
//     let port = listener.local_addr().unwrap().port();

//     let address = format!("http://127.0.0.1:{}", port);
//     let server = run(listener).expect("Failed to bind address");
//     tokio::spawn(server);
//     address
// }
