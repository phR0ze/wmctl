use x11rb::connection::Connection;
use tracing::{debug};

pub fn info() {
    debug!("");

    // Open the connection to the X server. Use the DISPLAY environment variable.
    let (conn, screen_num) = x11rb::connect(None).unwrap();

    // Get the screen #screen_num
    let screen = &conn.setup().roots[screen_num];
 
    println!();
    println!("Informations of screen {}:", screen.root);
    println!("  width.........: {}", screen.width_in_pixels);
    println!("  height........: {}", screen.height_in_pixels);
    println!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
