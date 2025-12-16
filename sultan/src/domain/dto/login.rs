#[derive(Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
