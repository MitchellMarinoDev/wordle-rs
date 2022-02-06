use std::f32::consts::PI;
use std::time::Duration;
use bevy::prelude::*;
use crate::{App, get_tile_pos, Guess, InvalidGuess, Pause, PauseLock, SysLabel, TypedLetter};
use crate::components::{Tile, TileAssets, TileMap};
use crate::events::EndFlipAnim;
use crate::keyboard::Key;

const JUMP_ANIM_TIME: Duration = Duration::from_millis(100);
const FLIP_ANIM_TIME: Duration = Duration::from_millis(300);
const SHAKE_ANIM_TIME: Duration = Duration::from_millis(500);

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
	fn build(&self, app: &mut App) {
		app.add_system_set(SystemSet::new()
			.label(SysLabel::Anim)
			.after(SysLabel::Logic)
			
			.with_system(color_keyboard)
			.with_system(set_keyboard_color)
			
			.with_system(start_shake)
			.with_system(start_jump)
			.with_system(start_flip)
			
			.with_system(shake_anim)
			.with_system(jump_anim)
			.with_system(flip_anim)
		);
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
	pause: Res<Pause>
) {
	for guess in guess_r.iter() {
		let guess: &Guess = guess;
		commands.entity(tile_map[guess.row][0]).insert(FlipAnim::new(pause.lock()));
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
	mut tiles: Query<(Entity, &mut Transform, &mut JumpAnim), With<Tile>>,
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
				commands.entity(tile_map[y][x]).insert(FlipAnim::new(anim.pause_lock.clone()));
			} else {
				end_flip_anim_w.send(EndFlipAnim);
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

// TODO: should not replace a green.
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
}

impl FlipAnim {
	/// Constructs a new [`FlipAnim`]
	pub fn new(pause_lock: PauseLock) -> Self {
		Self {
			d: Duration::ZERO,
			should_change: false,
			changed: false,
			pause_lock,
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
