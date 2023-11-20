
    // BASE NODE
    // ----------------------------------------------------------------
    // For the node types that don't have a specific ui 

    pub fn add_base_node_ui(
        mut events: EventReader<NodeSpawnedEvent>,

        mut commands: Commands,
    
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut view_data: ResMut<ViewData>,
    ){
        commands.entity(ev.entity).insert((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
                transform: Transform::from_translation(Vec3::new(
                    ev.position.x + rng.gen_range(-10.0..10.0),
                    ev.position.y + rng.gen_range(-10.0..10.0),
                    view_data.top_z,
                )),
                ..default()
            },
        ));
        // Update the view_data so we can keep track of which zindex is the topmost
        view_data.top_z += 0.0001;
    }
    
    // FOLDER/DIRECTORY NODE
    // ----------------------------------------------------------------

    // FILE NODE
    // ----------------------------------------------------------------

    // IMAGE NODE
    // ----------------------------------------------------------------

    // parent.spawn((SpriteBundle {
    //     texture: image_handle,
    //     sprite: Sprite {
    //         anchor: bevy::sprite::Anchor::TopLeft,
    //         ..default()
    //     },
    
    //     transform: Transform {
    //         translation: Vec3::new(0.0, -30.0, 0.1),
    //         scale: Vec3::new(1.0, 1.0, 0.05),
    //         ..default()
    //     },
    //     ..default()
    // },
    // PickableBundle {
    //     pickable: Pickable {
    //         should_block_lower: false,
    //         ..default()
    //     },
    //     ..default()
    // }
    // ));   

    // TEXT NODE
    // ----------------------------------------------------------------

    // SVG NODE
    // ----------------------------------------------------------------

    pub fn add_svg_node_ui(

    ){

    }