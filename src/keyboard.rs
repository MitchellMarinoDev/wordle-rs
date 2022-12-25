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

		// Change detection is conservative. There is no guarantee that the value actually changed.
		// Therefore, we should keep track of the old value and make sure it really changed.
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
	
	let keyboard = commands
		.spawn(NodeBundle {
			background_color: Color::NONE.into(),
			style: Style {
				align_self: AlignSelf::FlexEnd,
				size: Size::new(Val::Percent(100.0), Val::Percent(24.0)),
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Column,
				..Default::default()
			},
			..Default::default()
		})
		.insert(Name::new("Keyboard"))
		.id();
	
	spawn_row("QWERTYUIOP", texture.clone(), font.clone(), &mut commands, keyboard);
	spawn_row("ASDFGHJKL",  texture.clone(), font.clone(), &mut commands, keyboard);
	spawn_row("ZXCVBNM",    texture.clone(), font.clone(), &mut commands, keyboard);
	// TODO: add enter and backspace
}

fn spawn_row(
	letters: &str,
	texture: Handle<Image>,
	font: Handle<Font>,
	commands: &mut Commands,
	keyboard: Entity,
) {
	// Keyboard
	commands.entity(keyboard)
		.with_children(|keyboard_cb| {
			keyboard_cb
				.spawn(NodeBundle {
					background_color: Color::NONE.into(),
					style: Style {
						padding: UiRect::all(Val::Px(2.0)),
						justify_content: JustifyContent::Center,
						flex_direction: FlexDirection::Row,
						..Default::default()
					},
					..Default::default()
				})
				.insert(Name::new(format!("{} row", letters)))
				// Spawn each key
				.with_children(|row_cb| {
					for letter in letters.chars() {
						row_cb.spawn(ButtonBundle {
							image: UiImage(texture.clone()),
							// node: Node{size: Vec2::new(KEY_SIZE, KEY_SIZE)},
							background_color: BackgroundColor(*L_GREY),
							style: Style {
								margin: UiRect::all(Val::Px(5.0)),
								align_items: AlignItems::Center,
								size: Size::new(Val::Px(KEY_SIZE), Val::Px(KEY_SIZE)),
								// size: Size::new(Val::Auto, Val::Auto),
								flex_grow: 0.0,
								flex_shrink: 1.0,
								..Default::default()
							},
							..Default::default()
						})
						.insert(Key::new(letter))
						.insert(Name::new(format!("{} key", letter)))
						.with_children(|key_cb| {
							// Text component
							key_cb.spawn(TextBundle {
								style: Style {
									margin: UiRect::all(Val::Auto),
									position: UiRect {
										bottom: Val::Px(0.27*KEY_TEXT_SIZE),
										..Default::default()
									},
									// padding: Rect::all(Val::Px(32.0)),
									..Default::default()
								},
								text: Text::from_section(
									letter,
									TextStyle {
										font: font.clone(),
										font_size: KEY_TEXT_SIZE,
										color: Color::WHITE,
									}
								).with_alignment(TextAlignment {
									horizontal: HorizontalAlign::Center,
									vertical: VerticalAlign::Center,
								}),
								
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
