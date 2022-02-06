use bevy::prelude::KeyCode;

pub trait GetChar {
	fn get_char(&self) -> Option<char>;
}

impl GetChar for KeyCode {
	fn get_char(&self) -> Option<char> {
		match self {
			KeyCode::A => Some('A'),
			KeyCode::B => Some('B'),
			KeyCode::C => Some('C'),
			KeyCode::D => Some('D'),
			KeyCode::E => Some('E'),
			KeyCode::F => Some('F'),
			KeyCode::G => Some('G'),
			KeyCode::H => Some('H'),
			KeyCode::I => Some('I'),
			KeyCode::J => Some('J'),
			KeyCode::K => Some('K'),
			KeyCode::L => Some('L'),
			KeyCode::M => Some('M'),
			KeyCode::N => Some('N'),
			KeyCode::O => Some('O'),
			KeyCode::P => Some('P'),
			KeyCode::Q => Some('Q'),
			KeyCode::R => Some('R'),
			KeyCode::S => Some('S'),
			KeyCode::T => Some('T'),
			KeyCode::U => Some('U'),
			KeyCode::V => Some('V'),
			KeyCode::W => Some('W'),
			KeyCode::X => Some('X'),
			KeyCode::Y => Some('Y'),
			KeyCode::Z => Some('Z'),
			_ => None,
		}
	}
}
