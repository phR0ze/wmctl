/// Property provides a convenient way to store window properties
pub struct Property {
    pub id: u32,       // atom id of the property
    pub name: String,  // atom name of the property
    pub value: String, // value of the property
}

impl Property {
    /// Create a new property
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            value: "".to_string(),
        }
    }
}
