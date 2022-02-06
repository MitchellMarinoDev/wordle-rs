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
			.add_event::<GameWin>()
		;
	}
}

/// An event that is fired when the player makes a valid guess.
#[derive(Clone)]
pub struct Guess {
	/// The word that was guessed.
	pub word: String,
	/// The correctness of the guess.
	pub correctness: [TileType; 5],
	/// The row that was guessed on.
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

#[derive(Clone)]
pub struct EndFlipAnim {
	/// The word that was guessed.
	pub word: String,
	/// The correctness of the guess.
	pub correctness: [TileType; 5],
	/// The row that was guessed on.
	pub row: usize,
}

impl Into<EndFlipAnim> for &Guess {
	fn into(self) -> EndFlipAnim {
		EndFlipAnim {
			row: self.row,
			word: self.word.clone(),
			correctness: self.correctness,
		}
	}
}

pub struct GameWin {
	/// The correct word.
	pub word: String,
	/// The row that the game was won on
	pub row: usize,
}
