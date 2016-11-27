use calx_resource::ResourceStore;

/// Init the static terrain assets.
pub fn terrain() {
    use terrain::{Form, Id, Kind, Tile};

    // Void, terrain 0 is special.
    Tile::insert_resource(Id::Empty as u8, Tile::new("blank_floor", Kind::Block, Form::Void));
    // "Level exit", a visible portal tile.
    Tile::insert_resource(Id::Gate as u8, Tile::new("gate", Kind::Ground, Form::Gate));
    Tile::insert_resource(Id::Ground as u8, Tile::new("ground", Kind::Ground, Form::Floor));
    Tile::insert_resource(Id::Grass as u8, Tile::new("grass", Kind::Ground, Form::Floor));
    Tile::insert_resource(Id::Water as u8, Tile::new("water", Kind::Water, Form::Floor));
    Tile::insert_resource(Id::Tree as u8, Tile::new("tree", Kind::Block, Form::Prop));
    Tile::insert_resource(Id::Wall as u8, Tile::new("wall", Kind::Block, Form::Wall));
    Tile::insert_resource(Id::Rock as u8, Tile::new("rock", Kind::Block, Form::Blob));
    Tile::insert_resource(Id::Corridor as u8, Tile::new("ground", Kind::Corridor, Form::Floor));
    Tile::insert_resource(Id::OpenDoor as u8, Tile::new("open_door", Kind::Ground, Form::Wall));
    Tile::insert_resource(Id::Door as u8, Tile::new("door", Kind::Door, Form::Wall));
}
