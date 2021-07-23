use anyhow::Result;
use log::info;
use serde_json::Value;
use tide::{Request, Result as TideResult};

#[async_std::main]
async fn main() -> Result<()> {
    let mut app = tide::new();
    app.at("/").get(send_checkin);

    println!("Hello, world!");
    Ok(())
}

async fn send_checkin(mut req: Request<()>) -> TideResult {
    let body: Value = req.body_json().await?;
    info!("{:?}", body);
    Ok("".into())
}
