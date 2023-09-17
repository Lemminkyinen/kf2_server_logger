mod database;
mod kf2;
mod parser;

use kf2::Kf2Logger;

#[tokio::main]
async fn main() {
    let kf2 = Kf2Logger::new_session(
        "http://192.168.1.7:8081".to_string(),
        "".to_string(),
        "".to_string(),
    )
    .await
    .unwrap();
}
