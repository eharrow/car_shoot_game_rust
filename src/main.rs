use rand::prelude::*;
use rusty_engine::{prelude::*, sprite};

#[derive(Default)]
struct GameState {
    marbles_left: Vec<String>,
    cars_left: u32,
    spawn_timer: Timer,
    points: u32,
}
const MARBLE_SPEED: f32 = 600.0;
const CAR_SPEED: f32 = 250.0;

fn main() {
    let mut game = Game::new();
    game.window_settings(WindowDescriptor {
        title: "Car Shooter".into(),
        ..Default::default()
    });
    game.audio_manager
        .play_music(MusicPreset::MysteriousMagic, 0.1);

    let player = game.add_sprite("player", SpritePreset::RacingBarrierRed);
    player.rotation = UP;
    player.scale = 0.5;
    player.translation.y = -325.0;
    player.layer = 10.0;

    let game_state = GameState {
        marbles_left: vec!["marble1".into(), "marble2".into(), "marble3".into()],
        spawn_timer: Timer::from_seconds(0.0, false),
        cars_left: 25,
        points: 0,
    };

    let msg = game.add_text("cars left", format!("Cars left: {}", game_state.cars_left));
    msg.translation = Vec2::new(540.0, -320.0);
    let points = game.add_text("points", format!("Points: {}", game_state.points));
    points.translation = Vec2::new(525.0, -345.0);

    game.show_colliders = true;
    game.add_logic(game_logic);
    game.run(game_state);
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    let player = engine.sprites.get_mut("player").unwrap();
    let player_x = player.translation.x;

    // Move cars right across the screen
    for cs in engine.sprites.iter_mut().filter(|c| c.0.starts_with("car")) {
        cs.1.translation.x += CAR_SPEED * engine.delta_f32;
    }

    // Clean up sprites that have moved off the top or the right side of the screen.
    let mut labels_to_delete: Vec<String> = Vec::new();
    for s in engine.sprites.iter() {
        let sprite = s.1;
        if sprite.translation.y > 400.0 || sprite.translation.x > 750.0 {
            labels_to_delete.push(s.0.clone());
        }
    }

    for ld in labels_to_delete.iter() {
        engine.sprites.remove(ld);
    }

    // fire a marble
    if engine.mouse_state.just_pressed(MouseButton::Left) {
        // The left mousebutton is currently pressed -- process some continuous movement
        let len = game_state.marbles_left.len();
        if !game_state.marbles_left.is_empty() {
            println!("{} marbles left!", len);
            let last_marble = game_state.marbles_left.remove(len - 1);
            println!("marble picked is {}", last_marble);
            let marble = engine.add_sprite(last_marble, SpritePreset::RollingBallBlue);
            marble.translation.x = player_x;
            marble.translation.y = -275.0;
            marble.layer = 5.0;
            marble.collision = true;
            engine.audio_manager.play_sfx(SfxPreset::Impact2, 0.7);
        }
    }

    // move marbles up the screen
    for (key, val) in engine
        .sprites
        .iter_mut()
        .filter(|s| s.0.starts_with("marble"))
    {
        val.translation.y += MARBLE_SPEED * engine.delta_f32
    }

    if let Some(location) = engine.mouse_state.location() {
        // println!("mouse here {}", location.x)
    }

    // Spawn a car if the game_state.spawn_timer just finished!
    let car_choices = vec![
        SpritePreset::RacingCarBlack,
        SpritePreset::RacingCarBlue,
        SpritePreset::RacingCarGreen,
        SpritePreset::RacingCarRed,
        SpritePreset::RacingCarYellow,
    ];
    if game_state.spawn_timer.tick(engine.delta).just_finished() {
        game_state.spawn_timer = Timer::from_seconds(thread_rng().gen_range(0.1..1.25), false);

        if game_state.cars_left > 0 {
            game_state.cars_left -= 1;
            let msg = engine.texts.get_mut("cars left").unwrap();
            msg.value = format!("Cars left: {}", game_state.cars_left);

            let car = format!("car{}", game_state.cars_left);
            let car_sprite = engine.add_sprite(
                car,
                car_choices
                    .iter()
                    .choose(&mut thread_rng())
                    .unwrap()
                    .clone(),
            );
            car_sprite.translation.x = -740.0;
            car_sprite.translation.y = thread_rng().gen_range(-100.0..325.0);
            car_sprite.collision = true;
        } else {
            // no more cars so end the game
        }
    }

    // process collisions
    for event in engine.collision_events.drain(..) {
        match event.state {
            CollisionState::Begin => {
                if !event.pair.one_starts_with("marble") {
                    continue;
                }

                game_state.points += 1;
                let points = engine.texts.get_mut("points").unwrap();
                points.value = format!("Points: {}", game_state.points);

                engine.sprites.remove(&event.pair.0);
                engine.sprites.remove(&event.pair.1);
                engine.audio_manager.play_sfx(SfxPreset::Confirmation1, 0.5);
                if event.pair.0.starts_with("marble") {
                    game_state.marbles_left.push(event.pair.0.clone())
                } else if event.pair.1.starts_with("marble") {
                    game_state.marbles_left.push(event.pair.1.clone())
                }
            }
            CollisionState::End => continue,
        }
    }
}
