use minigene::*;

pub const SCREEN_WIDTH: u32 = 200;
pub const SCREEN_HEIGHT: u32 = 65;
pub const WIDTH: u32 = 200;
pub const HEIGHT: u32 = 60;

#[derive(Component)]
pub struct Base;
#[derive(Component, Default, new, Clone, Copy)]
pub struct MiningMapTag;

#[derive(Clone, Debug, new)]
pub struct Mine {
    pub name: String,
    pub block_name: String,
    pub unlock_cost: u64,
    pub money_per_block: u64,
    pub tick_per_block: u32,
}

//fn gen_mines() -> Vec<Mine> {
//    vec![
//        Mine::new("A", "Cobblestone", 0, 5, 5),
//        Mine::new("B", "Cobblestone", 0, 5, 5),
//        Mine::new("C", "Cobblestone", 0, 5, 5),
//        Mine::new("D", "Cobblestone", 0, 5, 5),
//        Mine::new("E", "Cobblestone", 0, 5, 5),
//        Mine::new("F", "Cobblestone", 0, 5, 5),
//        Mine::new("G", "Cobblestone", 0, 5, 5),
//        Mine::new("H", "Cobblestone", 0, 5, 5),
//        Mine::new("I", "Cobblestone", 0, 5, 5),
//        Mine::new("J", "Cobblestone", 0, 5, 5),
//        Mine::new("K", "Cobblestone", 0, 5, 5),
//        Mine::new("L", "Cobblestone", 0, 5, 5),
//        Mine::new("M", "Cobblestone", 0, 5, 5),
//        Mine::new("N", "Cobblestone", 0, 5, 5),
//        Mine::new("O", "Cobblestone", 0, 5, 5),
//        Mine::new("P", "Cobblestone", 0, 5, 5),
//        Mine::new("Q", "Cobblestone", 0, 5, 5),
//        Mine::new("R", "Cobblestone", 0, 5, 5),
//        Mine::new("S", "Cobblestone", 0, 5, 5),
//        Mine::new("T", "Cobblestone", 0, 5, 5),
//        Mine::new("U", "Cobblestone", 0, 5, 5),
//        Mine::new("V", "Cobblestone", 0, 5, 5),
//        Mine::new("W", "Cobblestone", 0, 5, 5),
//        Mine::new("X", "Cobblestone", 0, 5, 5),
//        Mine::new("Y", "Cobblestone", 0, 5, 5),
//        Mine::new("Z", "Cobblestone", 0, 5, 5),
//    ]
//}

#[derive(Default, Clone, Debug, new)]
pub struct Progress {
    pub current_mine: u32,
    pub block_progress: u32,
    pub money_per_block: u32,
    pub mined: u64,
    pub money: u64,
}

#[derive(Component)]
pub struct Player;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Stats {
    Health,
    Defense,
    Attack,
    Mana,
}

#[derive(new)]
pub struct PlayerMovementRes {
    pub reader: ReaderId<VirtualKeyCode>,
}

system!(PlayerMovementSystem, |positions: WriteStorage<'a, Point>, players: ReadStorage<'a, Player>, global_map: ReadExpect<'a, CollisionResource>, channel: Read<'a, EventChannel<VirtualKeyCode>>,
        res: WriteExpect<'a, PlayerMovementRes>| {
    // doesn't handle two entities that want to go to the same tile.
    for key in channel.read(&mut res.reader) {
        for (mut pos, _) in (&mut positions, &players).join() {
            match key {
                VirtualKeyCode::H => try_move(&mut pos, &global_map, Direction::West),
                VirtualKeyCode::J => try_move(&mut pos, &global_map, Direction::South),
                VirtualKeyCode::L => try_move(&mut pos, &global_map, Direction::East),
                VirtualKeyCode::K => try_move(&mut pos, &global_map, Direction::North),
                _ => {},
            }
        }
    }
});


pub fn try_move(p: &mut Point, map: &CollisionResource, dir: Direction) {
    let (rel_x, rel_y) = (p.x - map.position.x, p.y - map.position.y);
    let mut n = p.clone();
    match dir {
        Direction::West => if rel_x > 0 {n.x -= 1},
        Direction::South => if rel_y < map.map.size().1 as i32 - 1 {n.y += 1},
        Direction::East => if rel_x < map.map.size().0 as i32 - 1 {n.x += 1},
        Direction::North => if rel_y > 0 {n.y -= 1},
        _ => unreachable!()
    }
    if !map.map.is_set((n.x - map.position.x) as u32, (n.y - map.position.y) as u32) {
        p.x = n.x;
        p.y = n.y;
    }
}

#[derive(new)]
pub struct MineRes {
    pub reader: ReaderId<VirtualKeyCode>,
}

system!(MineSystem, |positions: ReadStorage<'a, Point>, players: ReadStorage<'a, Player>, maps: WriteStorage<'a, CollisionMap>, tags: ReadStorage<'a, MiningMapTag>, 
        channel: Read<'a, EventChannel<VirtualKeyCode>>,
        res: WriteExpect<'a, MineRes>,
        progress: Write<'a, Progress>| {
    for key in channel.read(&mut res.reader) {
        match key {
            VirtualKeyCode::P => {
                for (player_pos, _) in (&positions, &players).join() {
                    for (map_pos, map, _) in (&positions, &mut maps, &tags).join() {
                        if position_inside_rect(player_pos.x, player_pos.y, 0, 0, map.size().0, map.size().1) {
                            let (rel_x, rel_y) = ((player_pos.x - map_pos.x) as u32, (player_pos.y + 1 - map_pos.y) as u32);
                            if rel_y < map.size().1 && map.is_set(rel_x, rel_y) {
                                progress.block_progress += 1;
                                if progress.block_progress > 5 {
                                    // TODO remove money_per_block from here
                                    progress.money_per_block = 5;
                                    progress.block_progress = 0;
                                    progress.mined += 1;
                                    progress.money += progress.money_per_block as u64;
                                    map.unset(rel_x, rel_y);
                                }
                            }
                            //if map.is_inside(pos) {
                            //    let (x, y) = map.relative_point(pos);
                            //    if map.is_set(
                            //}
                        }
                    }
                }
            },
            _ => {},
        }
    }
});

#[derive(new)]
pub struct ResetRes {
    pub reader: ReaderId<VirtualKeyCode>,
}

system!(ResetSystem, |positions: WriteStorage<'a, Point>, players: ReadStorage<'a, Player>,
        maps: WriteStorage<'a, CollisionMap>, tags: ReadStorage<'a, MiningMapTag>, 
        channel: Read<'a, EventChannel<VirtualKeyCode>>,
        res: WriteExpect<'a, ResetRes>,
        progress: Write<'a, Progress>| {
    for key in channel.read(&mut res.reader) {
        match key {
            VirtualKeyCode::R => {
                for (mut map, _) in (&mut maps, &tags).join() {
                    init_collision_map(&mut map);
                }
                for (mut p, _) in (&mut positions, &players).join() {
                    p.x = WIDTH as i32 / 2;
                    p.y = 10;
                }
                progress.block_progress = 0;
            },
            _ => {},
        }
    }
});

fn render<'a>(ctx: &mut BTerm, camera: &Camera, positions: ReadStorage<'a, Point>, multi_sprites: ReadStorage<'a, MultiSprite>, sprites: ReadStorage<'a, Sprite>, map: &CollisionResource,
              progress: &Progress) {
    ctx.cls();
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            if map.map.is_set(i, j) {
                ctx.print(i, j, "#");
            } else {
                ctx.print(i, j, " ");
            }
        }
    }

    render_sprites(ctx, camera, positions, multi_sprites, sprites);

    if progress.block_progress == 0 {
        ctx.draw_bar_horizontal(0, SCREEN_HEIGHT - 5, SCREEN_WIDTH, 1, 1, WHITE, BLACK);
    } else {
        ctx.draw_bar_horizontal(0, SCREEN_HEIGHT - 5, SCREEN_WIDTH, progress.block_progress, 5, WHITE, BLACK);
    }
    ctx.print(0, SCREEN_HEIGHT - 4, format!("Blocks Mined: {}, Mine Level: {}", progress.mined, progress.current_mine));
    ctx.print(0, SCREEN_HEIGHT - 3, format!("Money: {}$", progress.money));
    ctx.print(0, SCREEN_HEIGHT - 2, format!("Mine Material: Cobblestone, Tool: Standard Pickaxe, Money Per Block: {}$, Respawns In: 0s", progress.money_per_block));
    ctx.print(0, SCREEN_HEIGHT - 1, format!("Remaining Before Next Unlock: 0$"));
}

struct State {
    pub world: World,
    pub dispatcher: Dispatcher<'static, 'static>,
}
impl GameState for State {
    fn tick(&mut self, ctx : &mut BTerm) {
        // Input
        let mut input = INPUT.lock();
        for key in input.key_pressed_set().iter() {
            self.world.fetch_mut::<EventChannel<VirtualKeyCode>>().single_write(*key);
        }
        //self.world.insert(ctx.key.clone());
        self.dispatcher.dispatch(&mut self.world);
        render(ctx, &self.world.read_resource(), self.world.read_storage(), self.world.read_storage(), self.world.read_storage(), &self.world.read_resource(), &self.world.read_resource());
        self.world.maintain();
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
}

fn init_collision_map(coll: &mut CollisionMap) {
    for i in 0..WIDTH {
        for j in 0..20 {
            coll.unset(i, j);
        }
        for j in 20..HEIGHT {
            coll.set(i, j);
        }
    }
}

fn main() -> BError {
    let mut builder = DispatcherBuilder::new()
        .with(CombineCollisionSystem, "combine_collision", &[])
        //.with(UpdateCollisionResourceSystem, "update_collision_res", &["combine_collision"])
        .with(PlayerMovementSystem, "player_movement", &[])
        .with(ResetSystem, "reset", &[])
        .with(MineSystem, "mine", &[]);
    let (mut world, mut dispatcher, mut context) = 
        mini_init(SCREEN_WIDTH, SCREEN_HEIGHT, "World Digger", builder);

    world.register::<MultiSprite>();
    world.register::<Sprite>();
    world.register::<Comp<StatSet<Stats>>>();
    
    // overwrite the default channel
    let mut channel = EventChannel::<VirtualKeyCode>::new();
    let reader = channel.register_reader();
    let reader2 = channel.register_reader();
    let reader3 = channel.register_reader();
    world.insert(channel);
    world.insert(PlayerMovementRes::new(reader));
    world.insert(MineRes::new(reader2));
    world.insert(ResetRes::new(reader3));

    world.insert(CollisionResource::new(CollisionMap::new(WIDTH, HEIGHT), Point::new(0, 0)));
    world.insert(Camera::new(Point::new(0,0), Point::new(160, 60)));
    let stat_defs = StatDefinitions::from(vec![
        StatDefinition::new(Stats::Health, String::from("health"), String::from("HP"), 100.0),
        StatDefinition::new(Stats::Defense, String::from("defense"), String::from("Defense"), 0.0),
        StatDefinition::new(Stats::Attack, String::from("attack"), String::from("Attack"), 10.0),
        StatDefinition::new(Stats::Mana, String::from("mana"), String::from("MP"), 100.0),
    ]);

    // player
    world.create_entity()
        .with(Point::new(WIDTH / 2, 10))
        //.with(MultiSprite::new(MultiTileSprite::from_string("@@", 1, 2)))
        .with(Comp(stat_defs.to_statset()))
        .with(Sprite {
            glyph: to_cp437('@'),
            fg: RGBA::named(YELLOW),
            bg: RGBA::named(BLACK),
        })
        .with(Player)
        .build();

    world.insert(stat_defs);

    // single tile test
    //world.create_entity()
    //    .with(Point::new(5, 5))
    //    .with(Sprite {
    //        glyph: to_cp437('x'),
    //        fg: RGBA::named(YELLOW),
    //        bg: RGBA::named(BLACK),
    //    })
    //    .build();
    // creep spawner
    //world.create_entity()
    //    .with(Point::new(55, 10))
    //    .with(CreepSpawner(0))
    //    .build();
    //world.create_entity()
    //    .with(Point::new(25, 10))
    //    .with(CreepSpawner(0))
    //    .build();

    let mut coll = CollisionMap::new(WIDTH, HEIGHT);
    init_collision_map(&mut coll);
    world.create_entity()
        .with(Point::new(0, 0))
        .with(coll)
        .with(MiningMapTag)
        .build();

    //for i in 10..30 {
    //    world.create_entity()
    //        .with(Point::new(i, 49))
    //        .with(CreepSpawner(i))
    //        .build();
    //    world.create_entity()
    //        .with(Point::new(i, 1))
    //        .with(CreepSpawner(i))
    //        .build();
    //}

    let gs = State {
        world,
        dispatcher,
    };

    main_loop(context, gs)
}

