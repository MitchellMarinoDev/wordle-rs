mod util;
mod anim;
mod keyboard;
mod events;
mod components;

use bevy::prelude::*;
use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use components::*;
use crate::anim::AnimPlugin;
use crate::events::{Events, GameWin, Guess, InvalidGuess, TypedLetter};
use crate::keyboard::KeyboardPlugin;
use crate::TileType::Correct;
use crate::util::GetChar;

const TILE_SIZE: f32 = 100.0;
const TILE_MARGIN: f32 = 10.0;
const TILE_TOTAL: f32 = TILE_SIZE + TILE_MARGIN;

lazy_static! {
	static ref L_GREY: Color = Color::rgb_u8(0x81, 0x83, 0x84);
	static ref D_GREY: Color = Color::rgb_u8(0x3a, 0x3a, 0x3c);
	static ref YELLOW: Color = Color::rgb_u8(0xb5, 0x9f, 0x3b);
	static ref GREEN:  Color = Color::rgb_u8(0x53, 0x8d, 0x4e);
}

const TEXT_SIZE: f32 = 30.0;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb_u8(0x12, 0x12, 0x13)))
		.add_plugins(DefaultPlugins)
		
		.insert_resource(Pause::new())
		
		.add_plugin(Events)
		.add_plugin(AnimPlugin)
		.add_plugin(KeyboardPlugin)
		
		.add_startup_system(setup.label(SysLabel::Setup))
		
		// LOGIC SET
		.add_system_set(SystemSet::new()
			.label(SysLabel::Logic)
			.after(SysLabel::Input)
			
			.with_system(update_chars)
		)
		
		.add_system(get_input.label(SysLabel::Input).with_run_criteria(not_paused))
		.add_system(update_tile_chars.label(SysLabel::Graphics))
		
		.run();
}

// TODO: change camera to scale

// TODO: MILESTONES
//      game win events/anim
//      Show Word on fail
// 		restart button

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let mut camera = OrthographicCameraBundle::new_2d();
	camera.transform.translation.y = -20.0;
	commands.spawn_bundle(camera);
	
	commands.spawn_bundle(UiCameraBundle::default());
	
	commands.insert_resource(Cursor {x: 0, y: 0});
	
	// Dictionary
	let dic_raw = std::fs::read_to_string("assets/dictionary_reduced.txt")
		.expect("Check Dictionary File");
	let dic: Vec<String> = dic_raw.trim().lines().map(|w| w.to_owned()).collect();
	println!("word count: {}", dic.len());
	
	let mut rng = thread_rng();
	let correct_word = dic.choose(&mut rng).unwrap().to_owned();
	// println!("Word is: {}", correct_word);
	commands.insert_resource(Word(correct_word));
	commands.insert_resource(WordDic(dic));
	
	let tile_assets = TileAssets {
		default: asset_server.load("tiles/outline.png"),
		colored: asset_server.load("tiles/colored.png"),
	};
	
	commands.insert_resource(tile_assets.clone());
	
	// Title
	let font = asset_server.load("fonts/Swansea.ttf");
	let title_style = TextStyle {
		font: font.clone(),
		font_size: TILE_SIZE/2.0,
		color: Color::WHITE,
	};
	let alignment = TextAlignment {
		vertical: VerticalAlign::Center,
		horizontal: HorizontalAlign::Center,
	};
	
	commands.spawn()
		.insert_bundle(Text2dBundle {
			text: Text::with_section("WORDLE", title_style, alignment),
			transform: Transform::from_translation(Vec3::new(0.0, TILE_TOTAL * 4.0 + TILE_MARGIN, 0.0)),
			..Default::default()
		});
	
	// create a tile map of all "null"s
	let mut tile_map = TileMap::null();
	
	for y in 0..6 {
		for x in 0..5 {
			tile_map[y as usize][x as usize] =
			spawn_tile(&mut commands, tile_assets.default.clone(), font.clone(), x, y);
		}
	}
	
	commands.insert_resource(tile_map);
}


fn update_chars(
	mut tiles_q: Query<&mut Tile>,
	mut typed_letter_r: EventReader<TypedLetter>,
	tile_map: Res<TileMap>,
) {
	for typed_letter in typed_letter_r.iter() {
		let typed_letter: &TypedLetter = typed_letter;
		
		if typed_letter.valid {
			let tile_e = tile_map[typed_letter.y][typed_letter.x];
			let mut tile = tiles_q.get_mut(tile_e).unwrap();
			tile.c = Some(typed_letter.letter);
		}
	}
}

fn get_input(
	mut tiles_q: Query<&mut Tile>,
	keys: Res<Input<KeyCode>>,
	tile_map: Res<TileMap>,
	mut cursor: ResMut<Cursor>,
	dic: Res<WordDic>,
	word: Res<Word>,
	
	mut inv_guess_w: EventWriter<InvalidGuess>,
	mut guess_w: EventWriter<Guess>,
	mut typed_letter_w: EventWriter<TypedLetter>,
	mut game_win_w: EventWriter<GameWin>,
) {
	for k in keys.get_just_pressed() {
		let k: &KeyCode = k;
		
		if *k == KeyCode::Back {
			if cursor.x > 0 {
				cursor.x -= 1;
			}
			
			let entity = tile_map[cursor.y][cursor.x];
			let mut tile = tiles_q.get_mut(entity).unwrap();
			tile.c = None;
		}
		
		if *k == KeyCode::Return {
			if cursor.x == 5 {
				// Compile the guess into a string.
				let guess: String = tile_map[cursor.y].iter()
					.map(|e| tiles_q.get(*e).unwrap().c.unwrap())
					.collect::<String>()
					.to_lowercase();
				
				// Check Dictionary
				if dic.binary_search(&guess).is_ok() {
					// Check Correctness
					let tile_iter = tile_map[cursor.y].iter();
					
					let correctness = correctness(&*word, &*guess);
					for (e, c) in tile_iter.zip(correctness) {
						let mut tile = tiles_q.get_mut(*e).unwrap();
						tile.tt = c;
					}
					
					// send event.
					guess_w.send(Guess {
						word: guess.clone(),
						row: cursor.y,
						correctness,
					});
					
					if correctness == [TileType::Correct; 5] {
						// Game won
						game_win_w.send(GameWin {
							word: guess,
							row: cursor.y,
						})
					} else {
						// Game not won
						if cursor.y == 5 {
							println!("Word was: {}", *word);
						}
						cursor.next_line();
					}
					
					continue;
				}
			}
			
			// Invalid guess; send event.
			inv_guess_w.send(InvalidGuess {
				row: cursor.y,
			});
			
		}
		
		// if the key is a character
		if let Some(c) = k.get_char() {
			let valid = cursor.x < 5;
			// send event
			typed_letter_w.send(TypedLetter {
				x: cursor.x,
				y: cursor.y,
				valid,
				letter: c,
			});
			if valid {
				cursor.next_char();
			}
			
		}
	}
}

/// Updates the characters of the tiles.
fn update_tile_chars(
	tiles_q: Query<(&Tile, &Children), Changed<Tile>>,
	mut text_q: Query<&mut Text>,
) {
	for (tile, children) in tiles_q.iter() {
		let children: &Children = children;
		let tile: &Tile = tile;
		
		// The only child should be the entity holding the text
		let text_entity = children.iter().next().unwrap();
		
		let mut text = text_q.get_mut(*text_entity).unwrap();
		let val = &mut text.sections[0].value;
		
		val.clear();
		if let Some(c) = tile.c {
			val.push(c);
		}
	}
}

fn spawn_tile(
	commands: &mut Commands,
	texture: Handle<Image>,
	font: Handle<Font>,
	x: i32,
	y: i32,
) -> Entity {
	let alignment = TextAlignment {
		vertical: VerticalAlign::Center,
		horizontal: HorizontalAlign::Center,
	};
	
	let style = TextStyle {
		font,
		font_size: TEXT_SIZE,
		color: Color::WHITE,
	};
	
	let pos = get_tile_pos(x, y);
	
	commands.spawn()
		.with_children(|c| {
			c.spawn()
				.insert_bundle(Text2dBundle {
					text: Text::with_section("", style, alignment),
					transform: Transform::from_translation(Vec3::new(0.0, TEXT_SIZE * 0.27, 1.0)),
					..Default::default()
				});
		})
		.insert_bundle(SpriteBundle {
			texture,
			sprite: Sprite {
				// color,
				custom_size: Some(Vec2::splat(TILE_SIZE)),
				..Default::default()
			},
			transform: Transform {
				translation: pos,
				..Default::default()
			},
			
			..Default::default()
		})
		.insert(Tile { tt: TileType::Default, x: x as u32, y: y as u32, c: None})
		.id()
}

fn get_tile_pos(x: i32, y: i32) -> Vec3 {
	Vec3::new(
		(x-2) as f32 * TILE_TOTAL,
		(3 - y) as f32 * TILE_TOTAL,
		0.0,
	)
}

fn correctness(correct: &str, guess: &str) -> [TileType; 5] {
	assert_eq!(correct.len(), 5);
	assert_eq!(guess.len(), 5);
	
	let guess_chars: Vec<_> = guess.chars().collect();
	let mut correct_chars: Vec<_> = correct.chars().collect();
	
	let mut correctness = [TileType::Wrong; 5];
	
	// Check correct first
	for idx in 0..5 {
		if guess_chars[idx] == correct_chars[idx] {
			correctness[idx] = TileType::Correct;
			correct_chars[idx] = '-'; // Make sure the char doesn't get matched again
		}
	}
	
	// Now check for wrong spot
	for idx in 0..5 {
		// If this character was already found to be correct, skip it.
		if correctness[idx] == Correct { continue; }
		
		let c = guess_chars[idx];
		if let Some(c_idx) = correct_chars.iter().position(|o| *o == c) {
			correctness[idx] = TileType::Close;
			correct_chars[c_idx] = '-';
		}
	}
	
	correctness
}

#[test]
fn test_correctness_logic() {
	use TileType::*;
	assert_eq!(correctness("hello", "lemon"), [Close, Correct, Wrong, Close, Wrong]);
	assert_eq!(correctness("hello", "ppplp"), [Wrong, Wrong, Wrong, Correct, Wrong]);
	assert_eq!(correctness("hello", "shark"), [Wrong, Close, Wrong, Wrong, Wrong]);
	assert_eq!(correctness("mints", "mmmmm"), [Correct, Wrong, Wrong, Wrong, Wrong]);
	assert_eq!(correctness("slips", "ssssk"), [Correct, Close, Wrong, Wrong, Wrong]);
}
