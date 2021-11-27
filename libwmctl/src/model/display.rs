pub struct Display {
    pub width: u16,
    pub height: u16,
    pub number: usize,
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn display() {
        let display = model::Display {
            width: 2560,
            height: 1600,
            number: 0,
        };
        assert_eq!(2560, display.width);
        assert_eq!(2560, display.height);
        assert_eq!(0, display.number);
    }
}