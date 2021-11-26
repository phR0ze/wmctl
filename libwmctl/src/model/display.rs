struct Display {
    pub width: u16,
    pub height: u16,
}

#[cfg(test)]
mod tests {
    #[test]
    fn display() {
        let display1 = Display {
            width: 2560,
            height: 1600
        };
        assert_eq!(2 + 2, 4);
    }
}