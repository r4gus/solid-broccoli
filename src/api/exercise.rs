#[get("/hello/<name>")]
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}
