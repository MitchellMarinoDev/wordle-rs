mod util;
mod anim;
mod keyboard;

use bevy::prelude::*;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::anim::{AnimPlugin, FlipAnim, JumpAnim, ShakeAnim};
use crate::util::GetChar;

const TILE_SIZE: f32    = 100.0;
const TILE_MARGIN: f32  = 10.0;
const TILE_TOTAL: f32 = TILE_SIZE + TILE_MARGIN;

const TEXT_SIZE: f32 = 30.0;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.insert_resource(ClearColor(Color::rgb_u8(0x12, 0x12, 0x13)))
		
		.add_startup_system(setup)
		
		.add_system(get_input)
		.add_system(update_tile_graphic)
		
		.add_plugin(AnimPlugin)
		.run();
}

struct WordDic(Vec<String>);

#[derive(Copy, Clone)]
#[derive(Debug)]
enum TileType {
	Default,
	Correct,
	Close,
	Wrong,
}

#[derive(Component)]
struct Tile {
	tt: TileType,
	c: Option<char>,
	x: u32,
	y: u32,
}

#[derive(Clone)]
struct TileAssets {
	default: Handle<Image>,
	grey: Handle<Image>,
	yellow: Handle<Image>,
	green: Handle<Image>,
}

impl TileAssets {
	fn of_correctness(&self, correctness: TileType) -> Handle<Image> {
		match correctness {
			TileType::Correct => self.green.clone(),
			TileType::Close => self.yellow.clone(),
			TileType::Default => self.default.clone(),
			TileType::Wrong => self.grey.clone(),
		}
	}
}

struct Word(String);

struct Cursor {
	x: usize,
	y: usize,
}

impl Cursor {
	fn next_line(&mut self) -> bool {
		self.y += 1;
		self.x = 0;
		true // TODO: return conditionally
	}
}

struct TileMap {
	tiles: [[Entity; 5]; 6],
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	// Camera
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());
	
	commands.insert_resource(Cursor {x: 0, y: 0});
	
	// Dictionary
	let dic_raw = std::fs::read_to_string("assets/dictionary.txt")
		.expect("Check Dictionary File");
	let dic: Vec<String> = dic_raw.trim().lines().filter(|w| w.chars().count() == 5).map(|w| w.to_owned()).collect();
	println!("word count: {}", dic.len());
	
	
	let mut rng = thread_rng();
	commands.insert_resource(Word(dic.choose(&mut rng).unwrap().to_owned()));
	commands.insert_resource(WordDic(dic));
	
	let tile_assets = TileAssets {
		default: asset_server.load("tiles/default.png"),
		grey: asset_server.load("tiles/grey.png"),
		yellow: asset_server.load("tiles/yellow.png"),
		green: asset_server.load("tiles/green.png"),
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
	let mut tile_map = TileMap {tiles: [[Entity::from_bits(0); 5]; 6]};
	
	for y in 0..6 {
		for x in 0..5 {
			tile_map.tiles[y as usize][x as usize] =
			spawn_tile(&mut commands, tile_assets.default.clone(), font.clone(), x, y);
		}
	}
	
	commands.insert_resource(tile_map);
}

fn get_input(
	mut commands: Commands,
	mut tiles: Query<&mut Tile>,
	keys: Res<Input<KeyCode>>,
	tile_map: Res<TileMap>,
	mut cursor: ResMut<Cursor>,
	dic: Res<WordDic>,
	word: Res<Word>,
) {
	for k in keys.get_just_pressed() {
		let k: &KeyCode = k;
		
		if *k == KeyCode::Back {
			if cursor.x > 0 {
				cursor.x -= 1;
			}
			
			let entity = tile_map.tiles[cursor.y][cursor.x];
			let mut tile = tiles.get_mut(entity).unwrap();
			tile.c = None;
		}
		
		if *k == KeyCode::Return {
			if cursor.x == 5 {
				// Check Dictionary
				let guess: String = tile_map.tiles[cursor.y].iter()
					.map(|e| tiles.get(*e).unwrap().c.unwrap())
					.collect::<String>()
					.to_lowercase();
				
				if dic.0.contains(&guess) {
					let tile_iter = tile_map.tiles[cursor.y].iter();
					
					let correctness = correctness(&*word.0, &*guess);
					println!("{:?}", correctness);
					for (e, c) in tile_iter.zip(correctness) {
						let mut tile = tiles.get_mut(*e).unwrap();
						tile.tt = c;
					}
					
					commands.entity(tile_map.tiles[cursor.y][0]).insert(FlipAnim::new());
					
					cursor.next_line();
					if cursor.y == 6 {
						println!("Word was: {}", word.0);
					}
					continue;
				}
			}
			
			// Shake row
			for entity in tile_map.tiles[cursor.y].iter() {
				commands.entity(*entity).insert(ShakeAnim::new());
			}
		}
		
		if let Some(c) = k.get_char() {
			if let Some(&entity) = tile_map.tiles[cursor.y].get(cursor.x) {
				let mut tile = tiles.get_mut(entity).unwrap();
				tile.c = Some(c);
				// Start the animation.
				commands.entity(entity).insert(JumpAnim::new());
				
				cursor.x += 1;
				cursor.x = cursor.x.clamp(0, 5);
			}
		}
	}
}

/// Updates the graphics of the tiles (text and background).
fn update_tile_graphic(
	tiles: Query<(&Tile, &Children), Changed<Tile>>,
	mut text_q: Query<&mut Text>,
) {
	for (tile, children) in tiles.iter() {
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
	
	let mut correctness = [TileType::Wrong; 5];
	// TODO: do multiples
	for (i, c) in guess.chars().enumerate() {
		let correct_c = correct.chars().nth(i).unwrap();
		if c == correct_c {
			correctness[i] = TileType::Correct;
			// TODO: other
		} else if correct.contains(c) {
			correctness[i] = TileType::Close;
		}
	}
	
	correctness
}
