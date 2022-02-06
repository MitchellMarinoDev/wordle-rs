use std::ops::{Deref, DerefMut};
use std::fmt::Formatter;
use std::sync::Arc;
use bevy::prelude::{Component, Res};
use bevy::ecs::schedule::{ShouldRun, SystemLabel};
use crate::{Color, D_GREY, Entity, GREEN, Handle, Image, L_GREY, YELLOW};

///! Contains components and resources.
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
#[derive(Debug)]
#[derive(Hash)]
#[derive(SystemLabel)]
pub enum SysLabel {
	Anim,
	Input,
	Logic,
	Graphics,
	Setup,
}

#[derive(Clone)]
pub struct WordDic(pub Vec<String>);

impl Deref for WordDic {
	type Target = Vec<String>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for WordDic {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Ord, PartialOrd)]
pub enum TileType {
	Correct,
	Close,
	Wrong,
	Default,
}

impl TileType {
	pub fn color(&self) -> Color {
		match self {
			TileType::Default => *L_GREY,
			TileType::Correct => *GREEN,
			TileType::Close => *YELLOW,
			TileType::Wrong => *D_GREY,
		}
	}
}

#[derive(Component)]
pub struct Tile {
	pub tt: TileType,
	pub c: Option<char>,
	pub x: u32,
	pub y: u32,
}

#[derive(Clone)]
pub struct TileAssets {
	pub default: Handle<Image>,
	pub colored: Handle<Image>,
}

#[derive(Clone)]
#[derive(PartialOrd, Ord, PartialEq, Eq)]
#[derive(Hash, Debug, Default)]
pub struct Word(pub String);

impl std::fmt::Display for Word {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.0, f)
	}
}

impl Deref for Word {
	type Target = String;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Word {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub struct Cursor {
	pub x: usize,
	pub y: usize,
}

impl Cursor {
	pub fn next_line(&mut self) {
		self.y += 1;
		self.x = 0;
	}
	
	pub fn next_char(&mut self) {
		self.x += 1;
		self.x = self.x.clamp(0, 5);
	}
}

pub struct TileMap {
	tiles: [[Entity; 5]; 6],
}

impl TileMap {
	/// Creates a new null filled [`TileMap`].
	/// Make sure to initialize all tiles.
	pub fn null() -> Self {
		TileMap {
			tiles: [[Entity::from_bits(0); 5]; 6],
		}
	}
}

impl Deref for TileMap {
	type Target = [[Entity; 5]; 6];
	
	fn deref(&self) -> &Self::Target {
		&self.tiles
	}
}

impl DerefMut for TileMap {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.tiles
	}
}

/// Can be used to pause the game as long as a [`PauseLock`] exists.
/// This is done using reference counting.
pub struct Pause {
	lock: Arc<()>,
}

impl Pause {
	pub fn new() -> Self {
		Pause {
			lock: Arc::new(()),
		}
	}
	
	pub fn lock(&self) -> PauseLock {
		PauseLock(self.lock.clone())
	}
	
	pub fn paused(&self) -> bool {
		Arc::strong_count(&self.lock) > 1
	}
}

#[derive(Clone)]
pub struct PauseLock(Arc<()>);

pub fn not_paused(
	pause: Res<Pause>,
) -> ShouldRun {
	if pause.paused() { ShouldRun::No } else { ShouldRun::Yes }
}
