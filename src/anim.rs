use std::f32::consts::PI;
use std::time::Duration;
use bevy::prelude::*;
use rand::{Rng, thread_rng};
use statrs::distribution::{ContinuousCDF, Normal};
use crate::{App, Confetti, ConfettiSpawner, get_tile_pos, Guess, InvalidGuess, Pause, PauseLock, SysLabel, TileType, TypedLetter};
use crate::components::{Tile, TileAssets, TileMap};
use crate::events::EndFlipAnim;
use crate::keyboard::Key;
use crate::util::GetKeyCode;

const JUMP_ANIM_TIME: Duration = Duration::from_millis(100);
const FLIP_ANIM_TIME: Duration = Duration::from_millis(300);
const SHAKE_ANIM_TIME: Duration = Duration::from_millis(500);
const WAVE_ANIM_TIME: Duration = Duration::from_millis(200);
const WAVE_AMPL: f32 = 30.0;

const CONFETTI_COUNT: u32 = 200;

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(setup_confetti_spawners)
		
			.add_system_set(SystemSet::new()
				.label(SysLabel::Anim)
				.after(SysLabel::Logic)
				
				.with_system(color_keyboard)
				.with_system(set_keyboard_color)
				.with_system(keyboard_jump)
				.with_system(spawn_confetti)
				.with_system(confetti_move)
				.with_system(confetti_gravity)
				
				
				.with_system(start_shake)
				.with_system(start_jump)
				.with_system(start_flip)
				.with_system(start_wave)
				
				.with_system(shake_anim)
				.with_system(jump_anim)
				.with_system(flip_anim)
				.with_system(wave_anim)
			)
		;
	}
}

fn start_shake(
	mut commands: Commands,
	mut inv_guess_r: EventReader<InvalidGuess>,
	tile_map: Res<TileMap>,
) {
	for inv_guess in inv_guess_r.iter() {
		let inv_guess: &InvalidGuess = inv_guess;
		
		for entity in tile_map[inv_guess.row].iter() {
			commands.entity(*entity).insert(ShakeAnim::new());
		}
	}
}

fn start_jump(
	mut commands: Commands,
	mut typed_letter_r: EventReader<TypedLetter>,
	tile_map: Res<TileMap>,
) {
	for typed_letter in typed_letter_r.iter() {
		let typed_letter: &TypedLetter = typed_letter;
		
		if typed_letter.valid {
			commands.entity(tile_map[typed_letter.y][typed_letter.x]).insert(JumpAnim::new());
		}
	}
}

fn start_flip(
	mut commands: Commands,
	mut guess_r: EventReader<Guess>,
	tile_map: Res<TileMap>,
	pause: Res<Pause>,
) {
	for guess in guess_r.iter() {
		let guess: &Guess = guess;
		commands.entity(tile_map[guess.row][0]).insert(FlipAnim::new(pause.lock(), guess.into()));
	}
}

fn start_wave(
	mut commands: Commands,
	mut end_flip_anim_r: EventReader<EndFlipAnim>,
	tile_map: Res<TileMap>,
) {
	for end_flip_anim in end_flip_anim_r.iter() {
		let end_flip_anim: &EndFlipAnim = end_flip_anim;
		
		if end_flip_anim.correctness == [TileType::Correct; 5] {
			commands.entity(tile_map[end_flip_anim.row][0]).insert(WaveAnim::new());
		}
	}
}


fn shake_anim(
	mut commands: Commands,
	mut tiles: Query<(Entity, &Tile, &mut Transform, &mut ShakeAnim)>,
	time: Res<Time>,
) {
	for (entity, tile, transform, anim) in tiles.iter_mut() {
		let entity: Entity = entity;
		let tile: &Tile = tile;
		let mut transform: Mut<Transform> = transform;
		let mut anim: Mut<ShakeAnim> = anim;
		
		let mut pos = get_tile_pos(tile.x as i32, tile.y as i32);
		
		if anim.tick(time.delta()) { // Finished
			transform.translation = pos;
			commands.entity(entity).remove::<ShakeAnim>();
			continue;
		}
		
		// Shake
		let shake_offset = (anim.scale() * PI).sin() * 8.0 * (anim.scale() * PI * 8.0).sin();
		pos.x += shake_offset;
		transform.translation = pos;
	}
}

fn jump_anim(
	mut commands: Commands,
	mut tiles: Query<(Entity, &mut Transform, &mut JumpAnim)>,
	time: Res<Time>,
) {
	for (entity, transform, anim) in tiles.iter_mut() {
		let entity: Entity = entity;
		let mut transform: Mut<Transform> = transform;
		let mut anim: Mut<JumpAnim> = anim;
		
		if anim.tick(time.delta()) { // Finished
			transform.scale = Vec3::ONE;
			commands.entity(entity).remove::<JumpAnim>();
			continue;
		}
		
		// Calculate new size
		let size = 1.0 + 0.1 * (PI * anim.scale()).sin();
		transform.scale = Vec3::splat(size);
	}
}

fn flip_anim(
	mut commands: Commands,
	mut tiles: Query<(Entity, &mut Transform, &Tile, &mut Handle<Image>, &mut Sprite, &mut FlipAnim)>,
	time: Res<Time>,
	tile_assets: Res<TileAssets>,
	tile_map: Res<TileMap>,
	mut end_flip_anim_w: EventWriter<EndFlipAnim>,
) {
	for (entity, transform, tile, texture, sprite, anim) in tiles.iter_mut() {
		let entity: Entity = entity;
		let mut transform: Mut<Transform> = transform;
		let tile: &Tile = tile;
		let mut texture: Mut<Handle<Image>> = texture;
		let mut sprite: Mut<Sprite> = sprite;
		let mut anim: Mut<FlipAnim> = anim;
		
		if anim.tick(time.delta()) { // Finished
			transform.scale = Vec3::ONE;
			commands.entity(entity).remove::<FlipAnim>();
			// Give the next tile the flip anim
			let x = tile.x as usize + 1;
			let y = tile.y as usize;
			if x < tile_map[0].len() {
				commands.entity(tile_map[y][x]).insert(FlipAnim::new(anim.pause_lock.clone(), anim.end_event.clone()));
			} else {
				end_flip_anim_w.send(anim.end_event.clone());
			}
			
			continue;
		}
		
		if anim.should_change() {
			*texture = tile_assets.colored.clone();
			sprite.color = tile.tt.color();
		}
		
		let scale = (anim.scale()-0.5).abs() * 2.0;
		transform.scale.y = scale;
	}
}

fn wave_anim(
	mut commands: Commands,
	mut tiles: Query<(Entity, &mut Transform, &Tile, &mut WaveAnim)>,
	time: Res<Time>,
	tile_map: Res<TileMap>,
) {
	for (entity, transform, tile, anim) in tiles.iter_mut() {
		let entity: Entity = entity;
		let mut transform: Mut<Transform> = transform;
		let tile: &Tile = tile;
		let mut anim: Mut<WaveAnim> = anim;
		
		if anim.tick(time.delta()) { // Finished
			// TODO: change fn signature of `get_tile_pos` to use usize.
			transform.translation = get_tile_pos(tile.x as i32, tile.y as i32);
			commands.entity(entity).remove::<WaveAnim>();
			// Give the next tile the flip anim
			let x = tile.x as usize + 1;
			let y = tile.y as usize;
			if x < tile_map[0].len() {
				commands.entity(tile_map[y][x]).insert(WaveAnim::new());
			}
			
			continue;
		}
		
		let mut pos = get_tile_pos(tile.x as i32, tile.y as i32);
		let height = (anim.scale() * PI).sin() * WAVE_AMPL;
		pos.y += height;
		
		transform.translation = pos;
	}
}

fn color_keyboard(
	mut keys_q: Query<(&mut UiColor, &Key)>,
	mut end_flip_anim_r: EventReader<EndFlipAnim>,
) {
	for _ in end_flip_anim_r.iter() {
		for (color, key) in keys_q.iter_mut() {
			let mut color: Mut<UiColor> = color;
			let key: &Key = key;
			
			color.0 = key.tt.color()
		}
	}
}

fn set_keyboard_color(
	mut keys_q: Query<&mut Key>,
	mut guess_r: EventReader<Guess>
) {
	for guess in guess_r.iter() {
		let guess: &Guess = guess;
		
		for mut key in keys_q.iter_mut() {
			// If the guess had this key's letter.
			if let Some(idx) = guess.word.chars().position(|c| c == key.key.to_ascii_lowercase()) {
				// If the guess has better info, update it.
				if key.tt > guess.correctness[idx] {
					key.tt = guess.correctness[idx];
				}
			}
		}
	}
}

fn keyboard_jump(
	mut commands: Commands,
	keys: Res<Input<KeyCode>>,
	keys_q: Query<(Entity, &Key)>
) {
	for key in keys.get_just_pressed() {
		let key: &KeyCode = key;
		
		for (entity, key_c) in keys_q.iter() {
			let entity: Entity = entity;
			let key_c: &Key = key_c;
			
			if key_c.key.get_keycode().unwrap() == *key {
				commands.entity(entity).insert(JumpAnim::new());
			}
		}
	}
}

fn confetti_move(
	mut confetti_q: Query<(&mut Transform, &Confetti)>
) {
	for (transform, confetti) in confetti_q.iter_mut() {
		let mut transform: Mut<Transform> = transform;
		let confetti: &Confetti = confetti;
		
		transform.translation += confetti.velocity;
		transform.rotation = transform.rotation.mul_quat(confetti.rot);
	}
}

fn confetti_gravity(
	mut confetti_q: Query<&mut Confetti>,
	time: Res<Time>,
) {
	for mut confetti in confetti_q.iter_mut() {
		confetti.velocity.y -= time.delta_seconds() * 5.0;
	}
}

fn spawn_confetti(
	mut commands: Commands,
	confetti_spawner_q: Query<(&Transform, &ConfettiSpawner)>,
	mut end_flip_anim_r: EventReader<EndFlipAnim>,
) {
	for end_flip_anim in end_flip_anim_r.iter() {
		let end_flip_anim: &EndFlipAnim = end_flip_anim;
		
		let mut rng = thread_rng();
		
		if end_flip_anim.correctness == [TileType::Correct; 5] {
			for (transform, confetti_spawner) in confetti_spawner_q.iter() {
				let transform: &Transform = transform;
				let confetti_spawner: &ConfettiSpawner = confetti_spawner;
				
				let v_x = confetti_spawner.dir.x;
				let v_y = confetti_spawner.dir.y;
				
				let normal_x = Normal::new(v_x as f64, (v_x.abs() / 2.0) as f64).unwrap();
				let normal_y = Normal::new(v_y as f64, (v_y.abs() / 2.0) as f64).unwrap();
				
				for _i in 0..CONFETTI_COUNT {
					let velocity = Vec3::new(
						normal_x.inverse_cdf(rng.gen_range(0.0..1.0)) as f32,
						normal_y.inverse_cdf(rng.gen_range(0.0..1.0)) as f32,
						1.0,
					);
					
					let rot = Quat::from_rotation_z(rng.gen_range(0.0..0.5));
					
					commands.spawn()
						.insert_bundle(SpriteBundle {
							sprite: Sprite {
								color: Color::rgb_u8(rng.gen(), rng.gen(), rng.gen()),
								custom_size: Some(Vec2::new(10.0, 5.0)),
								..Default::default()
							},
							transform: transform.clone(),
							..Default::default()
						})
						.insert(Confetti {
							velocity,
							rot,
						})
					;
				}
			}
		}
	}
}

// TODO: place these at the edge of the screen
fn setup_confetti_spawners(
	mut commands: Commands,
) {
	commands.spawn()
		.insert(ConfettiSpawner {
			dir: Vec3::new(-5.0, 7.0, 0.0),
		})
		.insert(Transform::from_translation(Vec3::new(550.0, -200.0, 0.0)));
	
	commands.spawn()
		.insert(ConfettiSpawner {
			dir: Vec3::new(5.0, 7.0, 0.0),
		})
		.insert(Transform::from_translation(Vec3::new(-550.0, -200.0, 0.0)));
}

/// The animation for shaking the tiles.
#[derive(Component)]
pub struct ShakeAnim {
	/// Elapsed duration.
	d: Duration,
}

impl ShakeAnim {
	/// Constructs a new [`ShakeAnim`]
	pub fn new() -> Self {
		Self {
			d: Duration::ZERO,
		}
	}
	
	/// Ticks by duration, then returns weather it finished.
	pub fn tick(&mut self, dur: Duration) -> bool {
		self.d += dur;
		self.d > SHAKE_ANIM_TIME
	}
	
	/// Returns a float of how completed the animation is.
	pub fn scale(&self) -> f32 {
		self.d.as_secs_f32() / SHAKE_ANIM_TIME.as_secs_f32()
	}
}

/// The Jump animation for adding a letter.
#[derive(Component)]
pub struct JumpAnim {
	/// Elapsed duration.
	d: Duration,
}


impl JumpAnim {
	/// Constructs a new [`JumpAnim`]
	pub fn new() -> Self {
		Self {
			d: Duration::ZERO,
		}
	}
	
	/// Ticks by duration, then returns weather it finished.
	pub fn tick(&mut self, dur: Duration) -> bool {
		self.d += dur;
		self.d > JUMP_ANIM_TIME
	}
	
	/// Returns a float of how completed the animation is.
	pub fn scale(&self) -> f32 {
		self.d.as_secs_f32() / JUMP_ANIM_TIME.as_secs_f32()
	}
}

/// The animation for flipping the tile color.
#[derive(Component)]
pub struct FlipAnim {
	/// Elapsed duration.
	d: Duration,
	/// Weather this should change.
	should_change: bool,
	/// Weather this has changed.
	changed: bool,
	/// The flip animation should pause the game.
	pause_lock: PauseLock,
	/// The end flip animation event that should be fired once the chain is done.
	end_event: EndFlipAnim,
}

impl FlipAnim {
	/// Constructs a new [`FlipAnim`]
	pub fn new(pause_lock: PauseLock, end_event: EndFlipAnim) -> Self {
		Self {
			d: Duration::ZERO,
			should_change: false,
			changed: false,
			pause_lock,
			end_event,
		}
	}
	
	/// Ticks by duration, then returns weather it finished.
	pub fn tick(&mut self, dur: Duration) -> bool {
		self.d += dur;
		if !self.changed && self.d > FLIP_ANIM_TIME / 2 {
			self.changed = true;
			self.should_change = true;
		}
		
		self.d > FLIP_ANIM_TIME
	}
	
	/// Returns weather the image should change. This will return exactly true once.
	pub fn should_change(&mut self) -> bool {
		let should_change = self.should_change;
		self.should_change = false;
		should_change
	}
	
	/// Returns a float of how completed the animation is.
	pub fn scale(&self) -> f32 {
		self.d.as_secs_f32() / FLIP_ANIM_TIME.as_secs_f32()
	}
}

/// The animation for the wave that the letters do on win.
#[derive(Component)]
pub struct WaveAnim {
	/// Elapsed duration.
	d: Duration,
}

impl WaveAnim {
	/// Constructs a new [`WaveAnim`]
	pub fn new() -> Self {
		Self {
			d: Duration::ZERO,
		}
	}
	
	/// Ticks by duration, then returns weather it finished.
	pub fn tick(&mut self, dur: Duration) -> bool {
		self.d += dur;
		self.d > WAVE_ANIM_TIME
	}
	
	/// Returns a float of how completed the animation is.
	pub fn scale(&self) -> f32 {
		self.d.as_secs_f32() / WAVE_ANIM_TIME.as_secs_f32()
	}
}
