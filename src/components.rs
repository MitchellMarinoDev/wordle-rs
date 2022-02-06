use std::ops::{Deref, DerefMut};
use std::fmt::Formatter;
use bevy::prelude::Component;
use bevy::ecs::schedule::SystemLabel;
use crate::{Entity, Handle, Image};

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
pub enum TileType {
	Default,
	Correct,
	Close,
	Wrong,
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
	pub grey: Handle<Image>,
	pub yellow: Handle<Image>,
	pub green: Handle<Image>,
}

impl TileAssets {
	pub fn of_correctness(&self, correctness: TileType) -> Handle<Image> {
		match correctness {
			TileType::Correct => self.green.clone(),
			TileType::Close => self.yellow.clone(),
			TileType::Default => self.default.clone(),
			TileType::Wrong => self.grey.clone(),
		}
	}
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
