#[macro_use]
extern crate specs_declaration;
#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate bracket_lib;

use bracket_lib::prelude::*;
use specs::prelude::*;
use hibitset::BitSet;
use game_features::*;
use shrev::*;

add_wasm_support!();

pub const SCREEN_WIDTH: u32 = 200;
pub const SCREEN_HEIGHT: u32 = 65;
pub const WIDTH: u32 = 200;
pub const HEIGHT: u32 = 60;

/// Component wrapper for types not implementing Component
#[derive(new)]
pub struct Comp<T>(T);
impl<T: Send+Sync+'static> Component for Comp<T> {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Component)]
pub struct Tower;
#[derive(Component)]
pub struct Creep;
#[derive(Component)]
pub struct CreepSpawner(u32);
#[derive(Component)]
pub struct Base;
#[derive(Component, Default, new, Clone, Copy)]
pub struct MiningMapTag;
#[derive(Component)]
pub struct Sprite {
    pub glyph: u16,
    pub fg: RGBA,
    pub bg: RGBA,
}
#[derive(Default, Clone, Debug, new)]
pub struct Progress {
    pub current_mine: u32,
    pub block_progress: u32,
    pub money_per_block: u32,
    pub mined: u64,
    pub money: u64,
}
#[derive(Component, new)]
pub struct MultiSprite {
    pub tile: MultiTileSprite,
}
#[derive(Component, new)]
pub struct AiPath {
    pub path: NavigationPath,
}

#[derive(Component, new)]
pub struct AiDestination {
    pub target: Point,
}
pub struct Spawner<F: Fn(&mut World)> {
    f: F,
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

pub struct GameSpeed(f32);

impl Default for GameSpeed {
    fn default() -> Self {
        GameSpeed(1.0)
    }
}

// Collision stuff
// Coords starts at the upper right corner
/// Collision of a single tile entity
#[derive(Component)]
pub struct Collision;
/// Collision of a multi tile entity. Not necessarily colliding everywhere.
/// Can be both used as a global resource and as a component for individual entities.
#[derive(Component)]
pub struct CollisionMap {
    bitset: BitSet,
    width: u32,
    height: u32,
}

impl CollisionMap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            bitset: BitSet::with_capacity(width * height),
            width,
            height,
        }
    }

    pub fn set(&mut self, x: u32, y: u32) {
        self.bitset.add(self.index_of(x, y));
    }

    pub fn unset(&mut self, x: u32, y: u32) {
        self.bitset.remove(self.index_of(x, y));
    }

    pub fn is_set(&self, x: u32, y: u32) -> bool {
        self.bitset.contains(self.index_of(x, y))
   }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn clear(&mut self) {
        self.bitset.clear();
    }

    pub fn index_of(&self, x: u32, y: u32) -> u32 {
        y * self.width + x
    }

    pub fn position_of(&self, idx: u32) -> (u32, u32) {
        (idx % self.width, idx / self.width)
    }
}

#[cfg(test)]
mod tests {
    use crate::CollisionMap;
    #[test]
    fn collmap() {
        let mut m = CollisionMap::new(3, 3);
        m.set(2, 2);
        assert!(m.is_set(2, 2));
        assert_eq!(m.index_of(2,2), 8);
        assert_eq!(m.position_of(8), (2, 2));
    }
}

#[derive(new)]
pub struct CollisionResource {
    pub map: CollisionMap,
    pub position: Point,
}

impl CollisionResource {
    pub fn is_inside(&self, p: &Point) -> bool {
        position_inside_rect(p.x - self.position.x, p.y - self.position.y, 0, 0, self.map.size().0, self.map.size().1)
    }
    /// Check is_inside before calling this.
    pub fn relative_point(&self, p: &Point) -> (u32, u32) {
        ((p.x - self.position.x) as u32, (p.y - self.position.y) as u32)
    }
}

impl Default for CollisionResource {
    fn default() -> Self {
        Self {
            map: CollisionMap::new(WIDTH, HEIGHT),
            position: Point::new(0, 0),
        }
    }
}

#[derive(new)]
pub struct Camera {
    pub position: Point,
    pub size: Point,
}

pub fn position_inside_rect(pos_x: i32, pos_y: i32, rect_x: i32, rect_y: i32, size_x: u32, size_y: u32) -> bool {
    pos_x >= rect_x &&
    pos_y >= rect_y &&
    pos_x < rect_x + size_x as i32 &&
    pos_y < rect_y + size_y as i32
}

system!(CombineCollisionSystem, |positions: ReadStorage<'a, Point>, collisions: ReadStorage<'a, Collision>, maps: ReadStorage<'a, CollisionMap>, global_map: Write<'a, CollisionResource>| {
    global_map.map.clear();

    for (pos, _) in (&positions, &collisions).join() {
        let (x, y) = (pos.x, pos.y);
        if position_inside_rect(x, y, global_map.position.x, global_map.position.y, global_map.map.size().0, global_map.map.size().1) {
            let (t_x, t_y) = (global_map.position.x, global_map.position.y);
            global_map.map.set((x - t_x) as u32, (y - t_y) as u32);
        }
    }

    for (pos, coll) in (&positions, &maps).join() {
        for i in 0..coll.size().0 as i32{
            for j in 0..coll.size().1 as i32 {
                let (x, y) = (pos.x + i, pos.y + j);
                if coll.is_set(i as u32, j as u32) && position_inside_rect(x, y, global_map.position.x, global_map.position.y, global_map.map.size().0, global_map.map.size().1) {
                    let (t_x, t_y) = (global_map.position.x, global_map.position.y);
                    global_map.map.set((x - t_x) as u32, (y - t_y) as u32);
                }
            }
        }
    }
});

// non portable
//system!(UpdateCollisionResourceSystem, |global_map: Write<'a, CollisionResource>, positions: ReadStorage<'a, Point>, players: ReadStorage<'a, Player>| {
//    for j in 0..50usize {
//        MAP[j].char_indices().for_each(|(i, c)| {
//            if c == '#' {
//                global_map.map.set(i as u32, j as u32);
//            } else {
//                global_map.map.unset(i as u32, j as u32);
//            }
//        });
//    }
//    for (pos, _) in (&positions, &players).join() {
//        global_map.position.x = pos.x - 40;
//        global_map.position.y = pos.y - 25;
//    }
//});

system!(CreepSpawnerSystem, |entities: Entities<'a>, positions: WriteStorage<'a, Point>, spawners: WriteStorage<'a, CreepSpawner>, creeps: WriteStorage<'a, Creep>,
        ai_destinations: WriteStorage<'a, AiDestination>, ai_paths: WriteStorage<'a, AiPath>, sprites: WriteStorage<'a, Sprite>| {
    let mut v = vec![];
    for (pos, mut spawner) in (&positions, &mut spawners).join() {
        if spawner.0 == 0 {
            spawner.0 = 20;
            // spawn
            v.push(pos.clone());
        }
        spawner.0 -= 1;
    }
    v.into_iter().for_each(|pos| {
        let creep = entities.create();
        positions.insert(creep, pos.clone()).unwrap();
        creeps.insert(creep, Creep).unwrap();
        ai_paths.insert(creep, AiPath::new(NavigationPath::new())).unwrap();
        ai_destinations.insert(creep, AiDestination::new(Point::new(39, 25))).unwrap();
        sprites.insert(creep, Sprite {
                glyph: to_cp437('c'),
                fg: RGBA::named(YELLOW),
                bg: RGBA::named(BLACK),
            }).unwrap();
    });
});

//system!(AiPathingSystem, |dests: ReadStorage<'a, AiDestination>, global_map: Read<'a, CollisionResource>, positions: ReadStorage<'a, Point>, paths: WriteStorage<'a, AiPath>| {
//    for (pos, dest, path) in (&positions, &dests, &mut paths).join() {
//        if pos.x == dest.target.x && pos.y == dest.target.y {
//            continue;
//        }
//        // TODO Safety check for < 0 or out of map bounds
//        let d = global_map.map.index_of((pos.x - global_map.position.x) as u32, (pos.y - global_map.position.y) as u32);
//        let t = global_map.map.index_of((dest.target.x - global_map.position.x) as u32, (dest.target.y - global_map.position.y) as u32);
//        let p = a_star_search(d, t, &global_map.map);
//        path.path = p;
//    }
//});

#[derive(new)]
pub struct PlayerMovementRes {
    pub reader: ReaderId<VirtualKeyCode>,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left, Down, Right, Up,
}

system!(PlayerMovementSystem, |positions: WriteStorage<'a, Point>, players: ReadStorage<'a, Player>, global_map: Read<'a, CollisionResource>, channel: Read<'a, EventChannel<VirtualKeyCode>>,
        res: WriteExpect<'a, PlayerMovementRes>| {
    // doesn't handle two entities that want to go to the same tile.
    for key in channel.read(&mut res.reader) {
        for (mut pos, _) in (&mut positions, &players).join() {
            match key {
                VirtualKeyCode::H => try_move(&mut pos, &global_map, Direction::Left),
                VirtualKeyCode::J => try_move(&mut pos, &global_map, Direction::Down),
                VirtualKeyCode::L => try_move(&mut pos, &global_map, Direction::Right),
                VirtualKeyCode::K => try_move(&mut pos, &global_map, Direction::Up),
                _ => {},
            }
        }
    }
});


pub fn try_move(p: &mut Point, map: &CollisionResource, dir: Direction) {
    let (rel_x, rel_y) = (p.x - map.position.x, p.y - map.position.y);
    let mut n = p.clone();
    match dir {
        Direction::Left => if rel_x > 0 {n.x -= 1},
        Direction::Down => if rel_y < map.map.size().1 as i32 - 1 {n.y += 1},
        Direction::Right => if rel_x < map.map.size().0 as i32 - 1 {n.x += 1},
        Direction::Up => if rel_y > 0 {n.y -= 1},
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

    for (pos, sprite) in (&positions, &multi_sprites).join() {
        sprite.tile.render(ctx, Point::new(pos.x - camera.position.x, pos.y - camera.position.y));
    }
    for (pos, sprite) in (&positions, &sprites).join() {
        ctx.set(pos.x - camera.position.x, pos.y - camera.position.y, sprite.fg, sprite.bg, sprite.glyph);
    }

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
    let context = BTermBuilder::new()
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .with_font("terminal8x8.png", 8, 8)
        .with_title("World Digger")
        .with_vsync(false)
        .with_advanced_input(true)
        .build()?;
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(CombineCollisionSystem, "combine_collision", &[])
        //.with(UpdateCollisionResourceSystem, "update_collision_res", &["combine_collision"])
        .with(CreepSpawnerSystem, "creep_spawner", &[])
        .with(PlayerMovementSystem, "player_movement", &[])
        .with(ResetSystem, "reset", &[])
        .with(MineSystem, "mine", &[])
        //.with(AiPathingSystem, "ai_pathing", &["update_collision_res"])
        //.with(AiMovementSystem, "ai_movement", &["ai_pathing"])
        .build();
    dispatcher.setup(&mut world);


    world.register::<MultiSprite>();
    world.register::<Sprite>();
    world.register::<Comp<StatSet<Stats>>>();
    let mut channel = EventChannel::<VirtualKeyCode>::new();
    let reader = channel.register_reader();
    let reader2 = channel.register_reader();
    let reader3 = channel.register_reader();
    world.insert(channel);
    world.insert(PlayerMovementRes::new(reader));
    world.insert(MineRes::new(reader2));
    world.insert(ResetRes::new(reader3));

    world.insert(CollisionResource::default());
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

