pub mod inventory;

pub use inventory::Inventory;
pub use inventory::Material;

pub fn get_greeting() -> String {
    "Construct Logic Ready".to_string()
}
