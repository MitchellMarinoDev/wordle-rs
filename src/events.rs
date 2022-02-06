use bevy::prelude::*;
use crate::TileType;

pub struct Events;

impl Plugin for Events {
	fn build(&self, app: &mut App) {
		app
			.add_event::<Guess>()
			.add_event::<InvalidGuess>()
			.add_event::<TypedLetter>()
			.add_event::<EndFlipAnim>()
		;
	}
}

/// An event that is fired when the player makes a valid guess.
pub struct Guess {
	pub word: String,
	pub correctness: [TileType; 5],
	pub row: usize,
}

/// An event that is fired when the player makes an invalid guess.
pub struct InvalidGuess {
	pub row: usize,
}

/// An event that is fired when the player types a letter.
pub struct TypedLetter {
	/// The x position of the typed letter
	pub x: usize,
	/// The y position of the typed letter
	pub y: usize,
	/// If the letter was put into the
	pub valid: bool,
	/// The lowercase letter that was just typed
	pub letter: char,
}

pub struct EndFlipAnim;
