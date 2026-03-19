pub mod tool {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Basic -> Copper -> Iron -> Gold -> Iridium)]
    pub struct ToolProgression;
}

pub mod soil {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Untilled -> Tilled -> Watered)]
    pub struct SoilProgression;
}

pub mod animal {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Baby -> Adult)]
    pub struct AnimalGrowth;
}

pub mod relationship {
    use ironclad::game_lifecycle;
    #[game_lifecycle(Stranger -> Acquaintance -> Friend -> CloseFriend -> Dating -> Engaged -> Married)]
    pub struct RelationshipProgression;
}

pub mod building {
    use ironclad::game_lifecycle;
    #[game_lifecycle(None -> Basic -> Big -> Deluxe)]
    pub struct BuildingProgression;
}
