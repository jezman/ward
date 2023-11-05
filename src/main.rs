use ward::Camera;

fn main() {
    let camera = Camera::new();

    match camera.list_numbers() {
        Ok(c) => println!("{:#?}", c),
        Err(e) => println!("{}", e),
    }
}
