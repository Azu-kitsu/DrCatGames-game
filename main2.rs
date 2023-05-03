use kira::sound::static_sound::{PlaybackState, StaticSoundHandle};
use sdl2::{
    event::Event, 
    keyboard::{Keycode, Scancode, KeyboardState}, 
    rect::{Rect, Point}, 
    render::{Canvas, TextureCreator}, 
    image::{self, LoadTexture, InitFlag, LoadSurface},
    render::{WindowCanvas, Texture},
    pixels::Color,
    surface::{Surface, SurfaceRef}, 
    mouse::{MouseButton, MouseState},
};
use std::{time::{Instant, Duration}, thread::sleep, vec, f64::RADIX, os::windows};
use sdl2::video::WindowContext;
use sdl2::ttf;

use rand::prelude::*;

use kira::{
    manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    track::TrackBuilder,
    tween::Tween,
};

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
fn main() -> Result<(), String> {
    //creating context
    let sdl_context = sdl2::init()?;
    sdl_context.mouse().show_cursor(false);
    let video_subsystem = sdl_context.video()?;

    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let mut window = video_subsystem.window("Dr. Cat Games", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .expect("could not init window (video subsys)");

    // Load the icon image into a surface
    let surface = Surface::load_bmp("assets/logo.bmp")?;
    // Set the icon of the window
    window.set_icon(surface);

    let mut canvas = window.into_canvas().build()
        .expect("could not make canvas");

    //creating loader
    let loader: TextureCreator<_> = canvas.texture_creator();
    
    //defining character

    let mut standing = Animation::new("assets/front.png", 1, 2, vec![(0, 0), (1, 0)], &loader);
    standing.dur = Duration::from_millis(400);

    let right = Animation::new("assets/movements/right.png", 1, 4, vec![(0, 0), (1, 0), (2, 0), (3, 0)], &loader);
    let left = Animation::new("assets/movements/left.png", 1, 4, vec![(0, 0), (1, 0), (2, 0), (3, 0)], &loader);
    let down = Animation::new("assets/movements/front.png", 1, 5, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)], &loader);
    let up = Animation::new("assets/movements/back.png", 1, 6, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)], &loader);

    let frames = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0), (10, 0), (11, 0), (12, 0), (13, 0), (14, 0)];
    let mut death = Animation::new("assets/death.png", 1, 15, frames, &loader);
    death.dur = Duration::from_millis(100);
    death.interruptable = false;
    death.looped = false;
    death.movable = false;

    //dash character animation
    let mut dash_left = Animation::new("assets/movements/dash_left.png", 1, 8, vec![(7, 0), (6, 0), (5, 0), (4, 0), (3, 0), (2, 0), (1, 0), (0, 0)], &loader);
    dash_left.dur = Duration::from_millis(20);
    dash_left.interruptable = false;
    dash_left.looped = false;
    let mut dash_right = Animation::new("assets/movements/dash_right.png", 1, 8, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0)], &loader);
    dash_right.dur = Duration::from_millis(20);
    dash_right.interruptable = false;
    dash_right.looped = false;
    let mut dash_front = Animation::new("assets/movements/dash_front.png", 1, 11, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0), (10, 0)], &loader);
    dash_front.dur = Duration::from_millis(20);
    dash_front.interruptable = false;
    dash_front.looped = false;
    let mut dash_back = Animation::new("assets/movements/dash_back.png", 1, 8, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0)], &loader);
    dash_back.dur = Duration::from_millis(20);
    dash_back.interruptable = false;
    dash_back.looped = false;
    //let dash_back = dash_front.clone(&loader);
    
    //attack
    let mut attack = Animation::new("assets/attack.png", 1, 8, vec![(0,0), (1,0), (2,0), (3, 0), (4,0), (5, 0), (6,0), (7,0)], &loader);
    attack.interruptable = false;
    attack.looped = false;
    attack.movable = false;
    let mut attack_left = Animation::new("assets/attack_left.png", 1, 8, vec![(0,0), (1,0), (2,0), (3, 0), (4,0), (5, 0), (6,0), (7,0)], &loader);
    attack_left.interruptable = false;
    attack_left.looped = false;
    attack_left.movable = false;

    let mut sponge = Entity::from(standing, 0, 0);
    sponge.animations.push(left);
    sponge.animations.push(right);
    sponge.animations.push(up);
    sponge.animations.push(down);
    //5
    sponge.animations.push(death);
    //6-9
    sponge.animations.push(dash_left);
    sponge.animations.push(dash_right);
    sponge.animations.push(dash_back);
    sponge.animations.push(dash_front);

    sponge.animations.push(attack);
    sponge.animations.push(attack_left);
    sponge.w = 100;
    sponge.h = 100;
    sponge.dst();
    sponge.gen_hitbox(Rect::new(sponge.w as i32 / 2 - 15, sponge.h as i32 - 10, 25, 1));
    
    let mut sponge = Character {
        x: 0, 
        y: 0, 
        rep: sponge, 
        dir: Direction::Right, 
        speed: 1,
        dodge_cooldown: Instant::now(),
    };
    
    
    //defining hearts
    let mut heart_loss = Animation::new("assets/selet.png", 1, 5, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)], &loader);
    heart_loss.dur = Duration::from_millis(400);
    heart_loss.looped = false;
    heart_loss.ongoing = false;
    let mut heart = Entity::from(heart_loss.clone(&loader), -((SCREEN_WIDTH / 2) as i32) + 25, -((SCREEN_HEIGHT / 2) as i32) + 25);
    heart.w = 50;
    heart.h = 50;
    heart.dst();

    let mut heart2 = heart.clone(&loader);
    heart2.x += 50;
    heart2.dst();
    
    let mut heart3 = heart.clone(&loader);
    heart3.x += 100;
    heart3.dst();

    //dash cooldown
    let mut dash_anim = Animation::new("assets/dash.png", 7, 1, vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6)], &loader);
    dash_anim.looped = false;
    dash_anim.ongoing = false;
    dash_anim.dur = Duration::from_millis(286);
    let mut dash = Entity::from(dash_anim, (SCREEN_WIDTH as i32 / 2)-44, -(SCREEN_HEIGHT as i32 / 2)+28);
    dash.w *= 3;
    dash.h *= 3;
    dash.dst();

    //logo
    let logo = Animation::new("assets/UI/DRcatgameslogo.png", 1, 1, vec![(0, 0)], &loader);
    let mut logo = Entity::from(logo, 0, 0);
    logo.mult_w(0.5);
    logo.mult_h(0.5);
    logo.center();

    //tree
    let tree = Animation::new("assets/tree.png", 1, 1, vec![(0, 0)], &loader);
    let mut tree = Entity::from(tree, 0, 0);
    tree.w *= 2;
    tree.h *= 2;
    tree.dst();
    tree.z_index = 3;
    let mut tree_2 = tree.clone(&loader);
    tree.gen_hitbox(Rect::new(tree.w as i32 / 2 - 15, tree.h as i32 -10, 25, 10));
    tree.offset_x(100);
    
    
    tree_2.x += 200;
    tree_2.dst();
    tree_2.gen_hitbox(Rect::new(tree_2.w as i32 / 2 - 15, tree_2.h as i32 -10, 25, 10));


    let mut cat = Animation::new("assets/TX Player.png", 1, 1, vec![(0, 0)], &loader);
    cat.current_frame.2 = Some(Rect::new(5, 13, 23, 45));
    let mut cat = Entity::from(cat, -5670, -350);
    cat.mult_w(0.7);
    cat.z_index = 2;
    cat.gen_hitbox(Rect::new(0, cat.h as i32 - 30, cat.w, 30));
    let cat = Animal::from(3, cat);

    let mut enemy = Entity::from(
        Animation::new("assets/enemy/down.png", 1, 8, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7,0)], &loader),
        -5670, -350
    );
    enemy.animations.push(Animation::new("assets/enemy/down.png", 1, 8, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7,0)], &loader));
    enemy.animations.push(Animation::new("assets/enemy/up.png", 1, 8, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7,0)], &loader));
    enemy.mult_h(2.0);
    enemy.mult_w(2.0);
    enemy.z_index = 2;
    let mut enemy = Enemy::from(3, enemy);
    enemy.entity.switch_to(2);
    

    //map test
    let mut map = Animation::new("assets/map_base.png", 1, 1, vec![(0, 0)], &loader);
    let mut map = Entity::from(map, 0, 0);
    map.mult_h(7.0);
    map.mult_w(7.0);
    //map.z_index = 0;
    map.gen_hitbox(Rect::new(0, 0, 0, 0));
    let mut map = ComplexHitbox::new(map);
    map.add_hitbox(Rect::new(416, 2922, 1284, 1213));
    map.add_hitbox(Rect::new(1130, 3350, 77, 33));


    let mut world = World::from(map, vec![tree, tree_2], 3, &loader);
    world.x = 5670;
    world.y = 370;
    world.add(cat);
    world.add(enemy);

    //Interactables

    let mut test = Interactable::new(
        Rect::new(1128, 3383, 86, 36),
        Some({
            fn test() -> bool {
                println!("working?");
                true
            } test
        }),
        world.map.base.dst,
    );
    world.add_interaction(test);

    //text

    let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;


    let mut text = Text::from(
        "JÁTÉK KEZDÉSE", 
        "assets/fonts/Aiden-v7DO.otf", 
        100, 
        Color::RGB(255, 255, 255),
        Point::new(10, 10),
        &ttf, &loader);
    text.center();
    text.dst.y -= 100;

    //sound
    
    // Create an audio manager. This plays sounds and manages resources.
    /* 
    let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
    let track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let track2 = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let track3 = manager.add_sub_track(TrackBuilder::default()).unwrap();
    
    let sound_data = StaticSoundData::from_file("assets/sounds/running_in_grass.mp3", StaticSoundSettings::new().track(&track)).unwrap();
    let mut sound = manager.play(sound_data.clone()).unwrap();
    //sound.set_volume(1f64, Tween { ..Default::default() });
    let mut running = Audio {data: sound_data, current: sound};
    
    let sound_data = StaticSoundData::from_file("assets/sounds/slash.mp3", StaticSoundSettings::new().track(&track2)).unwrap();
    let sound = manager.play(sound_data.clone()).unwrap();
    let mut slash = Audio {data: sound_data, current: sound};
    
    //let sound_data = StaticSoundData::from_file("assets/sounds/background.mp3", StaticSoundSettings::default()).unwrap();
    //let mut sound = manager.play(sound_data.clone()).unwrap();
   // sound.set_volume(0.2f64, Tween { ..Default::default() });
    //let mut background = Audio {data: sound_data, current: sound};

  //  background.play(&mut manager);

    // After a couple seconds...
    // Cloning the sound data will not use any extra memory.
*/
    //UI

    let mut test_button = Button::new(
        Rect::new(0, 0, 400, 100), 
        Some({
                fn simple_callback() -> bool {
                    false
        } simple_callback
        }), 
        text.current);
    test_button.center();


    

    //game loop
    let mut event_pump = sdl_context.event_pump()?;
    let mut i: u8 = 0;
    let mut menu = true;
    'running: for i in 0..255 {
        //event handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(255-i, 255-i, 255-i));
        canvas.clear();

        logo.present(&mut canvas,0,0)?;

        canvas.present();
        //ticks
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    'running: loop {
        
        canvas.clear();

        //get mouse
        let mouse = event_pump.mouse_state();
        //get keyboard
        let keyboard = event_pump.keyboard_state();


        if menu {
            if let Some(res) = test_button.exec(&mouse) {
                let res = res();
                println!("{:?}", res);
                menu = res;
            }

            test_button.present(&mut canvas)?;


        }

        
        else if !menu {

            if keyboard.is_scancode_pressed(Scancode::K) {
                heart.play(None);
                heart2.play(None);
                heart3.play(None);
                sponge.rep.play(Some(5));
            }
            if keyboard.is_scancode_pressed(Scancode::Right) {
                //slash.play(&mut manager);
                sponge.rep.play(Some(10));
            }
            else if keyboard.is_scancode_pressed(Scancode::Left) {
                //slash.play(&mut manager);
                sponge.rep.play(Some(11));
            }

            let mut moved = false;
            if sponge.rep.animations[sponge.rep.active].movable {
                //checking for sprint
                sponge.speed(&keyboard, &mut dash);

                //movement
                moved = sponge.movement(&keyboard, &mut world);
            }
            //if not moved then stand
            if !moved {
                sponge.rep.switch_to(0);
                //running.stop(0);
            } else {
                //running.play(&mut manager);
            }

            world.reorder_char(&mut sponge);
            //play next frame of animations
            sponge.rep.next();
            heart.next();
            heart2.next();
            heart3.next();
            dash.next();
            world.do_behaviours(&sponge);

            //map.dst();


            //new_frame
            i = (i + 1) % 255;
            //rendering

            canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        

            world.present(&mut canvas, &sponge)?;

            if let Some((x, i)) = world.check_interact(&sponge, &keyboard, &mut canvas) {
                println!("{:?} {}", x, i)
            }

            heart.present(&mut canvas, 0, 0)?;
            heart2.present(&mut canvas, 0, 0)?;
            heart3.present(&mut canvas, 0, 0)?;
            dash.present(&mut canvas, 0, 0)?;

        }

        //event handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.present();
        //ticks
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
struct ComplexHitbox<'a> {
    base: Entity<'a>,
    ideal: Vec<Rect>,
    real: Vec<Rect>
}
impl<'a> ComplexHitbox<'a> {
    fn create_real(&self, hitbox: Rect) -> Rect {
        Rect::new(
            self.base.dst.x() + hitbox.x(),
            self.base.dst.y() + hitbox.y(),
            hitbox.width(),
            hitbox.height()
        )
    }
    fn add_hitbox(&mut self, ideal: Rect) {
        let real = self.create_real(ideal);
        self.real.push(real);
        self.ideal.push(ideal);
    }
    fn add_hitboxes(&mut self, ideals: Vec<Rect>) {
        for ideal in ideals {
            self.add_hitbox(ideal)
        }
    }
    fn collide_all(&self, other: Rect, world_x: i32, world_y: i32) -> bool {
        for real in &self.real {
            let mut copy = *real;
            copy.set_x(copy.x() + world_x);
            copy.set_y(copy.y() + world_y);
            if let Some(x) = copy.intersection(other) {
                if x != other {
                    println!("{}", copy.has_intersection(other));
                    return true
                }
            }
        }
        false
    }
    fn new(base: Entity<'a>) -> Self {
        let base_hitbox = base.hitbox;
        let new = ComplexHitbox { base, ideal: vec![], real: vec![base_hitbox] };
        new
    }
}
struct Interactable {
    hitbox: Rect,
    callback: Option<fn() -> bool>,
}
impl Interactable{
    fn new(hitbox: Rect, callback: Option<fn() -> bool>, map_dst: Rect) -> Self {
        let hitbox = Rect::new(
            map_dst.x() + hitbox.x(),
            map_dst.y() + hitbox.y(),
            hitbox.width(),
            hitbox.height()
        );
        Interactable {
            hitbox, callback
        }
    }
    fn check(&self, world_x: i32, world_y: i32, hitbox: Rect) -> bool {
        let mut interactable_hitbox = self.hitbox;
        interactable_hitbox.x += world_x;
        interactable_hitbox.y += world_y;
        if hitbox.has_intersection(interactable_hitbox) {
            println!("yoooo");
            return true
        }
        false
    }
    fn exec(&self, keyboard: &KeyboardState) -> Option<fn() -> bool> {
        if keyboard.is_scancode_pressed(Scancode::E) {
            return self.callback
        }
        None
    }
    
}
struct Button<'a, T> {
    hitbox: Rect,
    callback: Option<fn() -> T>,
    appearance: Texture<'a>
}
impl<'a, T> Button<'a, T> {
    fn new(hitbox: Rect, callback: Option<fn() -> T>, appearance: Texture<'a>) -> Self {
        Button {
            hitbox, callback, appearance
        }
    }
    fn check(&self, mouse: &MouseState) -> bool {
        self.hitbox.contains_point(Point::new(mouse.x(), mouse.y()))
    }
    fn exec(&self, mouse: &MouseState) -> Option<fn() -> T> {
        if self.check(mouse) && mouse.left() {
            return self.callback
        }
        None
    }
    fn present(&self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.copy(&self.appearance, None, self.hitbox)?;
        Ok(())
    }
    fn center(&mut self) {
        self.hitbox.x = (SCREEN_WIDTH as i32 - self.hitbox.w as i32) / 2;
        self.hitbox.y = (SCREEN_HEIGHT as i32 - self.hitbox.h as i32) / 2;
    }
}
struct Audio {
    data: kira::sound::static_sound::StaticSoundData,
    current: kira::sound::static_sound::StaticSoundHandle
}
impl Audio {
    fn play(&mut self, manager: &mut kira::manager::AudioManager) {
        if self.current.state() != PlaybackState::Playing {
            self.current = manager.play(self.data.clone()).unwrap();
        }
    }
    fn stop(&mut self, time: u64) {
        if self.current.state() == PlaybackState::Playing {
            self.current.stop(Tween {
                duration: Duration::from_secs(time),
                ..Default::default()
            }).unwrap();
        }    
    }
}
struct Text<'a> {
    content: &'a str,
    size: u16,
    color: Color,
    current: Texture<'a>,
    dst: Rect
}
impl<'a> Text<'a> {
    fn from(content: &'a str, path: &str, size: u16, color: Color, point: Point, ttf: &'a ttf::Sdl2TtfContext, loader: &'a TextureCreator<WindowContext>) -> Self {
        let font = ttf.load_font(path, size).map_err(|e| e.to_string()).unwrap();

        let surface = font
            .render(content)
            .blended(color)
            .map_err(|e| e.to_string())
            .unwrap();

        let text = loader
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let (width, height) = text.get_size();
        let dst = Rect::new(point.x(), point.y(), width, height);
        Text {content: &content, size, color, current: text, dst}
    }
    fn present(&self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.copy(&self.current, None, self.dst)?;
        Ok(())
    }
    fn center(&mut self) {
        self.dst.x = (SCREEN_WIDTH as i32 - self.dst.w as i32) / 2;
        self.dst.y = (SCREEN_HEIGHT as i32 - self.dst.h as i32) / 2;
    }
    fn to_entity(some: Text) -> Entity {
        let anim = Animation::from_texture(some.current, 1, 1, vec![(0, 0)]);
        Entity::from(anim, some.dst.x(), some.dst.y())
    }
}
struct Static<'a> {
    img: Texture<'a>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    dst: Rect
} impl<'a> Static<'a> {
    fn from(img: Texture<'a>, x: i32, y: i32) -> Self {
        let (unit_w, unit_h) = img.get_size();
        let dst = Rect::new(x, y, unit_w, unit_h);
        Static {
            img,
            x,
            y,
            w: unit_w,
            h: unit_h,
            dst
        }
    }
    fn center(&mut self) {
        self.x = (SCREEN_WIDTH as i32 - self.w as i32) / 2;
        self.y = (SCREEN_HEIGHT as i32 - self.h as i32) / 2;
        self.dst.x = self.x;
        self.dst.y = self.y;
    }
    fn dst(&mut self) {
        self.dst = Rect::new(self.x, self.y, self.w, self.h);
    }
}
trait Presentable {
    fn get_hitbox(&self) -> Rect;
    fn get_z_index(&self) -> u8;
    fn present(&self, canvas: &mut WindowCanvas, world_x: i32, world_y: i32) -> Result<(), String>;
    fn check_move(&mut self, x: i32, y: i32, hitbox: Rect) -> bool;
    fn behave(&mut self, char: &Character, x: i32, y: i32);
}
impl Presentable for Entity<'_> {
    fn present(&self, canvas: &mut WindowCanvas, world_x: i32, world_y: i32) -> Result<(), String> {
        let mut destination = self.dst;
        destination.x += world_x;
        destination.y += world_y;

        canvas.copy(
            &self.animations[self.active].sheet, 
            self.animations[self.active].get_src(),
            destination)?;
        //canvas.fill_rect(self.hitbox)?;
        Ok(())
    }
    fn check_move(&mut self, x: i32, y: i32, hitbox: Rect) -> bool {
        let mut entity_hitbox = self.hitbox;
        entity_hitbox.x += x;
        entity_hitbox.y += y;
        if hitbox.has_intersection(entity_hitbox) {
            return false
        }
        true
    }
    fn get_hitbox(&self) -> Rect {
        self.hitbox
    }
    fn get_z_index(&self) -> u8 {
        self.z_index
    }
    fn behave(&mut self, char: &Character, x: i32, y: i32) {
        return
    }
}
fn random(start: i32, end: i32) -> i32 {
    let mut rng = thread_rng();
    rng.gen_range(start..end)
}
fn chance(percent: u8) -> bool {
    random(1, 101) < percent as i32
}
struct Animal<'a> {
    can_move: bool,
    dir: Direction,
    speed: i32,
    entity: Entity<'a>
}
impl<'a> Animal<'a> {
    fn from(speed: i32, entity: Entity<'a>) -> Self {
        return Animal {can_move: false, dir: Direction::Down, speed, entity};
    }
    fn decide(&mut self) {
        if chance(95) {return}
        let new_dir: u8 = random(0, 4) as u8;
        self.dir = match new_dir {
            0 => Direction::Right,
            1 => Direction::Left,
            2 => Direction::Up,
            3 => Direction::Down,
            _ => panic!()
        }
    }
    fn move_dir(&mut self) {
        if chance(50) {return}
        match self.dir {
            Direction::Down => {self.entity.offset_y(self.speed)},
            Direction::Up => {self.entity.offset_y(-self.speed)},
            Direction::Right => {self.entity.offset_x(self.speed)},
            Direction::Left => {self.entity.offset_x(-self.speed)},
        }
    }
    fn can_move_self(&mut self, char: &Character, x: i32, y: i32) -> bool {
        let mut hitbox = self.entity.hitbox;
        hitbox.x += x;
        hitbox.y += y;

        match self.dir {
            Direction::Down => {
                //hitbox.y += char.speed;
                hitbox.h += self.speed + char.speed;
            }
            Direction::Up => {
                hitbox.y -= self.speed + char.speed;
                hitbox.h += self.speed + char.speed;
            }
            Direction::Right => {
                hitbox.w += self.speed + char.speed;
            }
            Direction::Left => {
                hitbox.x -= self.speed + char.speed;
                hitbox.w += self.speed + char.speed;
            }
        }
        if hitbox.has_intersection(char.rep.hitbox) {
            return false;
        }
        true
    }
}
impl Presentable for Animal<'_> {
    fn present(&self, canvas: &mut WindowCanvas, world_x: i32, world_y: i32) -> Result<(), String> {
        self.entity.present(canvas, world_x, world_y)
    }
    fn check_move(&mut self, x: i32, y: i32, hitbox: Rect) -> bool {
        let result = self.entity.check_move(x, y, hitbox);
        self.can_move = result;
        result
    }
    fn get_hitbox(&self) -> Rect {
        self.entity.hitbox
    }
    fn get_z_index(&self) -> u8 {
        self.entity.z_index
    }
    fn behave(&mut self, char: &Character, x: i32, y: i32) {
        self.decide();
        if self.can_move_self(char, x, y) {
            self.move_dir();
        }
    }
}
struct Enemy<'a> {
    can_move: bool,
    dir: Direction,
    speed: i32,
    entity: Entity<'a>
}
impl<'a> Enemy<'a> {
    fn from(speed: i32, entity: Entity<'a>) -> Self {
        return Enemy {can_move: false, dir: Direction::Down, speed, entity};
    }
    fn decide(&mut self) {
        if chance(98) {return}
        let new_dir: u8 = random(0, 4) as u8;
        self.dir = match new_dir {
            0 => Direction::Right,
            1 => Direction::Left,
            2 => Direction::Up,
            3 => Direction::Down,
            _ => panic!()
        }
    }
    fn move_dir(&mut self) {
        if chance(50) {return}
        match self.dir {
            Direction::Down => {self.entity.offset_y(self.speed);self.entity.play(Some(1));            },
            Direction::Up => {self.entity.offset_y(-self.speed); self.entity.play(Some(2))},
            Direction::Right => {self.entity.offset_x(self.speed)},
            Direction::Left => {self.entity.offset_x(-self.speed)},
        }
    }
    fn can_move_self(&mut self, char: &Character, x: i32, y: i32) -> bool {
        let mut hitbox = self.entity.hitbox;
        hitbox.x += x;
        hitbox.y += y;

        match self.dir {
            Direction::Down => {
                //hitbox.y += char.speed;
                hitbox.h += self.speed + char.speed;
            }
            Direction::Up => {
                hitbox.y -= self.speed + char.speed;
                hitbox.h += self.speed + char.speed;
            }
            Direction::Right => {
                hitbox.w += self.speed + char.speed;
            }
            Direction::Left => {
                hitbox.x -= self.speed + char.speed;
                hitbox.w += self.speed + char.speed;
            }
        }
        if hitbox.has_intersection(char.rep.hitbox) {
            return false;
        }
        true
    }
}
impl Presentable for Enemy<'_> {
    fn present(&self, canvas: &mut WindowCanvas, world_x: i32, world_y: i32) -> Result<(), String> {
        self.entity.present(canvas, world_x, world_y)
    }
    fn check_move(&mut self, x: i32, y: i32, hitbox: Rect) -> bool {
        let result = self.entity.check_move(x, y, hitbox);
        self.can_move = result;
        result
    }
    fn get_hitbox(&self) -> Rect {
        self.entity.hitbox
    }
    fn get_z_index(&self) -> u8 {
        self.entity.z_index
    }
    fn behave(&mut self, char: &Character, x: i32, y: i32) {
        self.decide();
        self.entity.next();
        if self.can_move_self(char, x, y) {
            self.move_dir();
        }
    }
}
struct World<'a> {
    x: i32,
    y: i32,
    map: ComplexHitbox<'a>,
    entities: Vec<Vec<Box<dyn Presentable + 'a>>>,
    interactables: Vec<Interactable>,
    e: Entity<'a>,
}


impl<'a> World<'a> {
    fn from(map: ComplexHitbox<'a>, entities: Vec<Entity<'a>>, highest: u8, loader: &'a TextureCreator<WindowContext>) -> Self {
        let mut entities_n: Vec<Vec<Box<dyn Presentable>>> = vec![vec![]];
        for _ in 0..highest + 1 {
            entities_n.push(vec![]);
        }
        for entity in entities {
            entities_n[entity.z_index as usize].push(Box::new(entity));
        }
        World {
            x: 0,
            y: 0,
            map,
            entities: entities_n,
            interactables: vec![],
            e: Entity::from(
                Animation::new("assets/E.png", 1, 1, vec![(0, 0)], loader),
                0, 0
            )
        }
    }
    fn add_interaction(&mut self, interactable: Interactable) {
        self.interactables.push(interactable);
    }
    fn add<T: Presentable + 'a>(&mut self, entity: T) {
        self.entities[entity.get_z_index() as usize].push(Box::new(entity));
    }
    fn can_move(&mut self, char: &mut Character, dir: Direction) -> bool {
        let mut can_move = true;
        let mut hitbox = char.rep.hitbox;

        match dir {
            Direction::Down => {
                //hitbox.y += char.speed;
                hitbox.h += char.speed;
            }
            Direction::Up => {
                hitbox.y -= char.speed;
                hitbox.h += char.speed;
            }
            Direction::Right => {
                hitbox.w += char.speed;
            }
            Direction::Left => {
                hitbox.x -= char.speed;
                hitbox.w += char.speed;
            }
        }

        if self.map.collide_all(hitbox, self.x, self.y) {
            return false
        }
        for layer in &mut self.entities {
            for entity in layer {
                if !entity.check_move(self.x, self.y, hitbox) {
                    return false
                }
            }
        }

        true
    }
    fn do_behaviours(&mut self, char: &Character ) {
        for layer in &mut self.entities {
            for entity in layer {
                entity.behave(char, self.x, self.y);
            }
        }
    }
    fn reorder_char(&mut self, char: &mut Character) {
        for layer in &self.entities {
            for entity in layer {
                
                let mut entity_hitbox = entity.get_hitbox();
                entity_hitbox.x += self.x;
                entity_hitbox.y += self.y;
                let entity_z_index = entity.get_z_index();
                if entity_hitbox.top_left().y() > char.rep.hitbox.top_left().y() && entity_z_index < char.rep.z_index { //if char is above (visually) the entity
                    //println!("{}, {}", char.rep.z_index, entity.z_index);
                    //println!("yoo");
                    char.rep.z_index = entity_z_index; //because char will always be the lowest in the layer
                } else if entity_hitbox.top_left().y() < char.rep.hitbox.top_left().y() {
                    char.rep.z_index = entity_z_index + 1;
                    
                    //println!("zoo");
                } //char is below the entity

            }
        }
    }
    fn check_interact(&mut self, char: &Character, keyboard: &KeyboardState, canvas: &mut WindowCanvas) -> Option<(fn() -> bool, u8)> {
        let mut i = 0;
        for act in &self.interactables {
            if act.check(self.x, self.y, char.rep.hitbox) {
                self.present_interact(canvas, act.hitbox).unwrap();
                if let Some(x) = act.exec(keyboard) {
                    return Some((x, i))
                }
            }
        }
        None
    }
    fn present(&self, canvas: &mut WindowCanvas, char: &Character) -> Result<(), String> {
        //self.tiles.present(canvas, self.x, self.y)?;
        self.map.base.present(canvas, self.x, self.y)?;
        let mut i = 0;
        for layer in &self.entities {
            if char.rep.z_index == i {
                char.rep.present(canvas, 0, 0)?;
            }
            i += 1;
            for entity in layer {
                entity.present(canvas, self.x, self.y)?;
                //canvas.fill_rect(hitbox)?;
            }
        }
        Ok(())
    }
    fn present_interact(&self, canvas: &mut WindowCanvas, other: Rect) -> Result<(), String> {
        self.e.present(canvas, 
            self.x + other.x() + (other.width() / 2) as i32, 
            self.y + other.y() + (other.height() / 2) as i32)?;
        Ok(())
    }

}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left = 1,
    Right = 2,
    Up = 3,
    Down = 4,
}
struct Character<'a> {
    x: i32, //x in the world's coordinate system
    y: i32, //y in the world's coordinate system
    rep: Entity<'a>, //the character's representation: an entity
    dir: Direction, //the direction the character is facing
    speed: i32,
    dodge_cooldown: Instant
}
impl<'a> Character<'a> {
    fn move_world(&mut self, dir: Direction, part: &mut i32) {
        *part += self.speed * ((-1i32).pow(dir as u32 + 1 % 2));
        self.dir = dir;
        self.rep.switch_to(dir as usize);
    }
    fn speed(&mut self, keyboard: &KeyboardState, dash: &mut Entity<'a>) -> bool {
        if keyboard.is_scancode_pressed(Scancode::LShift) {
            self.speed = 5;
        } else {
            self.speed = 3
        }
        if self.dodge_cooldown.elapsed() < Duration::from_millis(150) {
            self.speed = 18;
        }
        //checking for dash
        if keyboard.is_scancode_pressed(Scancode::LAlt) {
            if self.dodge_cooldown.elapsed() > Duration::from_secs(2) {
                let anim = self.dir as usize + 5;
                self.rep.play(Some(anim));
                self.speed = 20;
                dash.play(None);
                self.dodge_cooldown = Instant::now();
                return true
            }
        }
        false
    }
    fn movement(&mut self, keyboard: &KeyboardState, world: &mut World) -> bool {
        let mut moved = false;
        if keyboard.is_scancode_pressed(Scancode::D) {
            if world.can_move(self, Direction::Right) {
                self.move_world(Direction::Right, &mut world.x);
                moved = true;
            }
        }
        else if keyboard.is_scancode_pressed(Scancode::A) {
            if world.can_move(self, Direction::Left) {
                self.move_world(Direction::Left, &mut world.x);
                moved = true;
            }
        }
        if keyboard.is_scancode_pressed(Scancode::W) {
            if world.can_move(self, Direction::Up) {
                self.move_world(Direction::Up, &mut world.y);
                moved = true;
            }
        }
        else if keyboard.is_scancode_pressed(Scancode::S) {
            if world.can_move(self, Direction::Down) {
                self.move_world(Direction::Down, &mut world.y);
                moved = true;
            }
        }
        moved
    }
}
struct Entity<'a> {
    x: i32, //x in the coordinate system of the world
    y: i32, //y in the coordinate system of the world
    w: u32, //width of the current entity
    h: u32, //height of the current entity
    active: usize, //the number of the active animation from the animations list
    last: usize, //the number of the previous active animation
    animations: Vec<Animation<'a>>, //a list containing the entity's animations
    dst: Rect, //the destination where SDL2 will put the entity on the screen
    z_index: u8, //the layer number
    hitbox: Rect,
} 
impl<'a> Entity<'a> {
    fn from(base: Animation<'a>, x: i32, y: i32) -> Self {
        let (unit_w, unit_h) = base.get_units();
        //let x_n = ((SCREEN_WIDTH as i32 - unit_w as i32) / 2) + x;
        //let y_n = ((SCREEN_HEIGHT as i32 - unit_h as i32) / 2) + y;
        let dst = Rect::new(x, y, unit_w, unit_h);
        Entity {
            x,
            y,
            w: unit_w,
            h: unit_h,
            active: 0,
            last: 0,
            animations: vec![base],
            dst,
            z_index: 0,
            hitbox: dst,
        }
    }
    fn offset_x(&mut self, value: i32) {
        self.x += value;
        self.hitbox.x += value;
        self.dst();
    }
    fn offset_y(&mut self, value: i32) {
        self.y += value;
        self.hitbox.y += value;
        self.dst();
    }
    fn mult_w(&mut self, value: f32) {
        self.w = (self.w as f32 * value) as u32;
        self.hitbox.w = (self.hitbox.w as f32 * value) as i32;
        self.dst()
    }
    fn mult_h(&mut self, value: f32) {
        self.h = (self.h as f32 * value) as u32;
        self.hitbox.h = (self.hitbox.h as f32 * value) as i32;
        self.dst()
    }
    fn center(&mut self) {
        self.x = (SCREEN_WIDTH as i32 - self.w as i32) / 2;
        self.y = (SCREEN_HEIGHT as i32 - self.h as i32) / 2;
        self.dst.x = self.x;
        self.dst.y = self.y;
    }
    fn dst(&mut self) {
        let x = ((SCREEN_WIDTH as i32 - self.w as i32) / 2) + self.x;
        let y = ((SCREEN_HEIGHT as i32 - self.h as i32) / 2) + self.y;
        self.dst = Rect::new(x, y, self.w, self.h);
    }
    fn switch_to(&mut self, num: usize) {
        if self.active != num && self.animations[self.active].interruptable {
            self.last = self.active;
            self.active = num;
        }
    }
    fn play(&mut self, num: Option<usize>) {
        match num {
            Some(x) => {
                self.switch_to(x)
            }
            None => {()}
        }
        self.animations[self.active].ongoing = true;
    }
    fn force_switch(&mut self, num: usize) {
        self.active = num;
    }
    fn next(&mut self) {
        if self.animations[self.active].next() {
            self.force_switch(self.last);
        }
    }
    fn clone(&self, loader: &'a TextureCreator<WindowContext>) -> Self {
        let mut animations = Vec::new();
        for elem in &self.animations {
            animations.push(elem.clone(loader))
        }
        Entity { x: self.x, y: self.y, w: self.w, h: self.h, active: self.active, last: self.last, animations, dst: self.dst, z_index: self.z_index, hitbox: self.hitbox }
    }
    fn gen_hitbox(&mut self, hitbox: Rect) {
        let real_hitbox = Rect::new(
            self.dst.x + hitbox.x, 
            self.dst.y + hitbox.y, 
            hitbox.width(), hitbox.height());
        self.hitbox = real_hitbox
    }
    fn collide(&self, other: &Entity) -> bool {
        self.hitbox.has_intersection(other.hitbox)
    }
}

struct Animation<'a> {
    sheet: Texture<'a>, //the sheet from which the frames of the animation are sourced
    source: String, //the path to the image file containing the sheet
    rows: u8, //the number of rows (frames) on the sheet
    cols: u8, //the no. of cols (frames) on the sheet
    current_frame: (u8, u8, Option<Rect>), //the coordinate for the frame on the sheet, (x, y) = (col, row)
    current: usize, //number of the current frame out of the total no. of frames
    frames: Vec<(u8, u8, Option<Rect>)>, //a sequence containing the frames' coordinates
    total: usize, //the length of the sequence
    ongoing: bool, //is the animation ongoing
    dur: Duration, //the amount of time that needs to elapse between frames
    elapse: Instant, //stopper-watch used to measure the elapse of time
    interruptable: bool, //can the animation be interrupted by another animation
    looped: bool, //is the animation on a loop
    movable: bool, //can the entity move while displaying this animation
} 
impl<'a> Animation<'a> {
    fn get_units(&self) -> (u32, u32) {
        let (w, h) = self.sheet.get_size();
        let unit_w = w / self.cols as u32;
        let unit_h = h / self.rows as u32;
        (unit_w, unit_h)
    }
    fn get_src(&self) -> Rect {
        if let Some(src) = self.current_frame.2 {
            //println!("{:?}", src);
            return src
        } else {
            //println!("{:?}", self.current_frame.2);
            let (unit_w, unit_h) = self.get_units();
            let (i, j, _) = self.current_frame;
            Rect::new(i as i32 * unit_w as i32, j as i32 * unit_h as i32, unit_w, unit_h)
        }
    }
    fn new(filename: &str, rows: u8, cols: u8, frames: Vec<(u8, u8)>, loader: &'a TextureCreator<WindowContext>) -> Self {
        let total = frames.len() - 1;
        let mut frames_new = vec![];
        for frame in frames {
            let frame_new = (frame.0, frame.1, None);
            frames_new.push(frame_new);
        }
        Animation {
            sheet: loader.load_texture(filename).unwrap(), 
            source: filename.to_string(),
            rows, 
            cols, 
            current_frame: frames_new[0], 
            current: 0, 
            frames: frames_new, 
            total, 
            ongoing: true, 
            dur: Duration::from_millis(100), 
            elapse: Instant::now(),
            interruptable: true,
            looped: true,
            movable: true,
        }
    }
    fn from_texture(texture: Texture<'a>, rows: u8, cols: u8, frames: Vec<(u8, u8)>) -> Self {
        let total = frames.len() - 1;
        let mut frames_new = vec![];
        for frame in frames {
            let frame_new = (frame.0, frame.1, None);
            frames_new.push(frame_new);
        }
        Animation {
            sheet: texture, 
            source: "".to_string(),
            rows, 
            cols, 
            current_frame: frames_new[0], 
            current: 0, 
            frames: frames_new, 
            total, 
            ongoing: true, 
            dur: Duration::from_millis(100), 
            elapse: Instant::now(),
            interruptable: true,
            looped: true,
            movable: true,
        }
    }
    fn next(&mut self) -> bool {
        if self.elapse.elapsed() < self.dur {
            return false
        }
        if !self.ongoing {
            return true
        }
        if self.current == self.total {
            self.current = 0;
            if !self.looped {
                self.current_frame = self.frames[self.current];
                self.ongoing = false;
                return true
            }
        } else {
            self.current += 1;
        }

        self.current_frame = self.frames[self.current];
        self.elapse = Instant::now();
        return false       
    }
    fn clone(&self, loader: &'a TextureCreator<WindowContext>) -> Self {
        Animation {
            sheet: loader.load_texture(self.source.clone()).unwrap(),
            source: self.source.clone(), 
            rows: self.rows, 
            cols: self.cols, 
            current_frame: self.current_frame, 
            current: self.current, 
            frames: self.frames.clone(), 
            total: self.total, 
            ongoing: self.ongoing, 
            dur: self.dur, 
            elapse: self.elapse, 
            interruptable: self.interruptable, 
            looped: self.looped,
            movable: self.movable
        }
    }
}

trait Sized {
    fn get_size(&self) -> (u32, u32);
} impl Sized for Texture<'_> {
    fn get_size(&self) -> (u32, u32) {
        let query = self.query();
        let width = query.width;
        let height = query.height;
        (width, height)
    }
}