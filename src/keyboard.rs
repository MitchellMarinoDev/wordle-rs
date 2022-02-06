use bevy::prelude::*;
use crate::Interaction::Clicked;
use crate::{L_GREY, SysLabel, TileType};
use crate::util::GetKeyCode;

const KEY_SIZE: f32 = 75.0;
const KEY_TEXT_SIZE: f32 = 16.0;

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(setup_keyboard.label(SysLabel::Setup))
			.add_system(simulate_keyboard.before(SysLabel::Input))
		;
	}
}

#[derive(Component)]
pub struct Key {
	pub key: char,
	pub old: Interaction,
	pub tt: TileType,
}

impl Key {
	pub fn new(key: char) -> Self {
		Key { key, old: Interaction::None, tt: TileType::Default }
	}
}

fn simulate_keyboard(
	mut key_q: Query<(&Interaction, &mut Key), (Changed<Interaction>, With<Button>)>,
	mut keys: ResMut<Input<KeyCode>>,
) {
	for (interaction, key) in key_q.iter_mut() {
		let interaction: Interaction = *interaction;
		let mut key: Mut<Key> = key;
		
		if interaction == Clicked && key.old != Clicked {
			keys.press(key.key.get_keycode().unwrap())
		}
		else if interaction != Clicked && key.old == Clicked {
			keys.release(key.key.get_keycode().unwrap())
		}
		
		key.old = interaction;
	}
}

fn setup_keyboard(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let texture = asset_server.load("tiles/key_tile.png");
	let font = asset_server.load("fonts/Swansea.ttf");
	
	let keyboard = commands.spawn()
		.insert_bundle(NodeBundle {
			color: Color::NONE.into(),
			style: Style {
				size: Size::new(Val::Percent(100.0), Val::Px(265.0)),
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::ColumnReverse,
				..Default::default()
			},
			..Default::default()
		}).id();
	
	
	spawn_row("QWERTYUIOP".chars(), texture.clone(), font.clone(), &mut commands, keyboard);
	spawn_row("ASDFGHJKL".chars(),  texture.clone(), font.clone(), &mut commands, keyboard);
	spawn_row("ZXCVBNM".chars(),    texture.clone(), font.clone(), &mut commands, keyboard);
	// TODO: add enter and backspace
}

fn spawn_row(
	letters: impl Iterator<Item=char> + Clone,
	texture: Handle<Image>,
	font: Handle<Font>,
	commands: &mut Commands,
	keyboard: Entity,
) {
	// Keyboard
	commands.entity(keyboard)
		.with_children(|keyboard_cb| {
			keyboard_cb.spawn()
				.insert_bundle(NodeBundle {
					color: Color::NONE.into(),
					style: Style {
						padding: Rect::all(Val::Px(2.0)),
						justify_content: JustifyContent::Center,
						flex_direction: FlexDirection::Row,
						..Default::default()
					},
					..Default::default()
				})
				// Spawn each key
				.with_children(|row_cb| {
					for letter in letters {
						row_cb.spawn_bundle(ButtonBundle {
							image: UiImage(texture.clone()),
							// node: Node{size: Vec2::new(KEY_SIZE, KEY_SIZE)},
							color: UiColor(*L_GREY),
							style: Style {
								margin: Rect::all(Val::Px(5.0)),
								align_items: AlignItems::Center,
								size: Size::new(Val::Px(KEY_SIZE), Val::Px(KEY_SIZE)),
								..Default::default()
							},
							..Default::default()
						})
						.insert(Key::new(letter))
						.with_children(|key_cb| {
							// Text component
							key_cb.spawn_bundle(TextBundle {
								style: Style {
									margin: Rect {
										left: Val::Auto,
										right: Val::Auto,
										top: Val::Auto,
										bottom: Val::Auto,
									},
									position: Rect {
										bottom: Val::Px(0.27*KEY_TEXT_SIZE),
										..Default::default()
									},
									// padding: Rect::all(Val::Px(32.0)),
									..Default::default()
								},
								text: Text::with_section(
									letter,
									TextStyle {
										font: font.clone(),
										font_size: KEY_TEXT_SIZE,
										color: Color::WHITE,
									},
									TextAlignment {
										horizontal: HorizontalAlign::Center,
										vertical: VerticalAlign::Center,
									}
								),
								
								..Default::default()
							});
						})
						;
					}
				})
			;
		})
	;
}
