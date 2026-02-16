pub mod material;

pub use material::Material;

pub struct Inventory {
    pub items: Vec<Material>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: Material) {
        self.items.push(item);
    }
}
