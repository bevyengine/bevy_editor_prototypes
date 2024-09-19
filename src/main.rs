use bevy::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(DefaultPlugins).run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let foo = true;
        assert!(foo, "it works!");
    }
}
