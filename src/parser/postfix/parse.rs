use super::ParserConfig;
use super::ParseError;
use super::messages::*;

pub fn parse_line(raw: String, conf: &ParserConfig) -> Result<Option<Message>, ParseError> {
	let (inner, start) = match Inner::parse(conf, raw) {
		Err(error) => return Err(error),
		Ok(None) => return Ok(None),
		Ok(Some((x,y))) => (x,y)
	};
	match inner.process {
		Process::Pickup => match Pickup::parse(inner, start) {
			Err(error) => Err(error),
			Ok(None) => Ok(None),
			Ok(Some(m)) => Ok(Some(Message::Pickup { m:m }))
		},
		Process::Pipe => match Forward::parse(inner, start) {
			Err(error) => Err(error),
			Ok(None) => Ok(None),
			Ok(Some(m)) => Ok(Some(Message::Forward { m:m }))
		},
		Process::Smtp => match Forward::parse(inner, start) {
			Err(error) => Err(error),
			Ok(None) => Ok(None),
			Ok(Some(m)) => Ok(Some(Message::Forward { m:m }))
		},
		Process::Local => match Forward::parse(inner, start) {
			Err(error) => Err(error),
			Ok(None) => Ok(None),
			Ok(Some(m)) => Ok(Some(Message::Forward { m:m }))
		},
		s => panic!("Unsupported process: {:?}", s)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::ParserConfig;
	use super::super::ParseError;
	use super::super::messages::*;

	fn parse(s: String) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		parse_line(s, &conf)
	}

	#[test]
	fn ignore() {
		match parse("Sep  3 00:00:03 yuuai clamsmtpd:".to_string()) {
			Ok(None) => (),
			Err(x) => panic!("Wrong Error (Should have been ignored): {}", x),
			_ => panic!("Should have been ignored")
		}
	}
}

