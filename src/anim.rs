use std::f32::consts::PI;
use std::time::Duration;
use bevy::prelude::*;
use crate::{App, get_tile_pos, Guess, InvalidGuess, TypedLetter};
use crate::components::{Tile, TileAssets, TileMap};

const JUMP_ANIM_TIME: Duration = Duration::from_millis(100);
const FLIP_ANIM_TIME: Duration = Duration::from_millis(300);
const SHAKE_ANIM_TIME: Duration = Duration::from_millis(500);

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
	fn build(&self, app: &mut App) {
		// TODO: label these systems;
		app
			.add_system(start_shake)
			.add_system(start_jump)
			.add_system(start_flip)
			
			.add_system(shake_anim)
			.add_system(jump_anim)
			.add_system(flip_anim)
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
		
		commands.entity(tile_map[typed_letter.y][typed_letter.x]).insert(JumpAnim::new());
	}
}

fn start_flip(
	mut commands: Commands,
	mut guess_r: EventReader<Guess>,
	tile_map: Res<TileMap>,
) {
	for guess in guess_r.iter() {
		let guess: &Guess = guess;
		commands.entity(tile_map[guess.row][0]).insert(FlipAnim::new());
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
	mut tiles: Query<(Entity, &mut Transform, &Tile, &mut Handle<Image>, &mut FlipAnim)>,
	time: Res<Time>,
	tile_assets: Res<TileAssets>,
	tile_map: Res<TileMap>,
) {
	for (entity, transform, tile, texture, anim) in tiles.iter_mut() {
		let entity: Entity = entity;
		let mut transform: Mut<Transform> = transform;
		let tile: &Tile = tile;
		let mut texture: Mut<Handle<Image>> = texture;
		let mut anim: Mut<FlipAnim> = anim;
		
		if anim.tick(time.delta()) { // Finished
			transform.scale = Vec3::ONE;
			commands.entity(entity).remove::<FlipAnim>();
			// Give the next tile the flip anim
			let x = tile.x as usize + 1;
			let y = tile.y as usize;
			if x < tile_map[0].len() {
				commands.entity(tile_map[y][x]).insert(FlipAnim::new());
			}
			
			continue;
		}
		
		if anim.should_change() {
			*texture = tile_assets.of_correctness(tile.tt);
		}
		
		let scale = (anim.scale()-0.5).abs() * 2.0;
		transform.scale.y = scale;
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
}

impl FlipAnim {
	/// Constructs a new [`FlipAnim`]
	pub fn new() -> Self {
		Self {
			d: Duration::ZERO,
			should_change: false,
			changed: false
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
