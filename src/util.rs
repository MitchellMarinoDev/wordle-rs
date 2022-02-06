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

pub trait GetKeyCode {
	fn get_keycode(&self) -> Option<KeyCode>;
}

impl GetKeyCode for char {
	fn get_keycode(&self) -> Option<KeyCode> {
		match self.to_ascii_lowercase() {
			'a' => Some(KeyCode::A),
			'b' => Some(KeyCode::B),
			'c' => Some(KeyCode::C),
			'd' => Some(KeyCode::D),
			'e' => Some(KeyCode::E),
			'f' => Some(KeyCode::F),
			'g' => Some(KeyCode::G),
			'h' => Some(KeyCode::H),
			'i' => Some(KeyCode::I),
			'j' => Some(KeyCode::J),
			'k' => Some(KeyCode::K),
			'l' => Some(KeyCode::L),
			'm' => Some(KeyCode::M),
			'n' => Some(KeyCode::N),
			'o' => Some(KeyCode::O),
			'p' => Some(KeyCode::P),
			'q' => Some(KeyCode::Q),
			'r' => Some(KeyCode::R),
			's' => Some(KeyCode::S),
			't' => Some(KeyCode::T),
			'u' => Some(KeyCode::U),
			'v' => Some(KeyCode::V),
			'w' => Some(KeyCode::W),
			'x' => Some(KeyCode::X),
			'y' => Some(KeyCode::Y),
			'z' => Some(KeyCode::Z),
			_ => None,
		}
	}
}
