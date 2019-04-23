use bitbar::{
    Menu,
    MenuItem
};

fn bitbar() -> Menu {
    Menu(vec![
        MenuItem::new("disk: NYI"),
        MenuItem::Sep,
        MenuItem::new("not yet implemented")
    ])
}

fn main() {
    print!("{}", bitbar());
}
