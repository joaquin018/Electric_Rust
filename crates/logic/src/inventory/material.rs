#[derive(Debug, Clone)]
pub struct Material {
    pub id: String,
    pub name: String,
    pub quantity: f64,
}

impl Material {
    pub fn new(id: &str, name: &str, quantity: f64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            quantity,
        }
    }
}
