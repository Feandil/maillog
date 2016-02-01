extern crate maillog;

use std::io;
use std::io::prelude::*;
use maillog::parser::postfix::*;
use maillog::parser::postfix::messages::Message;

struct Counters {
	all: u64,
	ignored: u64,
	bounce: u64,
	pickup: u64,
	forward: u64,
	forwarderror: u64,
	smtpd: u64,
	smtpdforward: u64,
	smtpdlogin: u64,
	cleanup: u64,
	qmgr: u64,
	qmgrremoved: u64,
	qmgrexpired: u64,
	rejects: u64,
}

#[cfg_attr(test, allow(dead_code))]
fn print(counts: &Counters) {
	println!("Read {} lines", counts.all);
	println!("Ignored: {}", counts.ignored);
	println!("Bounce: {}", counts.bounce);
	println!("Pickups: {}", counts.pickup);
	println!("Forwards: {}", counts.forward);
	println!("ForwardErrors: {}", counts.forwarderror);
	println!("Smtpd: {}", counts.smtpd);
	println!("SmtpdForward: {}", counts.smtpdforward);
	println!("SmtpdLogin: {}", counts.smtpdlogin);
	println!("Cleanups: {}", counts.cleanup);
	println!("Qmgr: {}", counts.qmgr);
	println!("QmgrRemoved: {}", counts.qmgrremoved);
	println!("QmgrExpired: {}", counts.qmgrexpired);
	println!("Rejects: {}", counts.rejects);
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
	let mut counts = Counters { all: 0, ignored: 0, bounce: 0, pickup: 0, forward: 0, forwarderror: 0, smtpd: 0, smtpdforward: 0, smtpdlogin: 0, cleanup: 0, qmgr: 0, qmgrremoved: 0, qmgrexpired: 0, rejects: 0 };
	let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string(), "postlicyd".to_string()] };

	let stdin = io::stdin();
	let mut buffer: Vec<u8> = Vec::new();
	loop {
		buffer.clear();
		let len = stdin.lock().read_until(b'\n', &mut buffer).unwrap();
		if len == 0 {
			break;
		}
		let line = String::from_utf8_lossy(&buffer[..len-1]).into_owned();
		counts.all += 1;
		match parse_line(line.clone(), &conf) {
			Ok(None) => counts.ignored += 1,
			Ok(Some(Message::Bounce{m:_})) => counts.bounce += 1,
			Ok(Some(Message::Cleanup{m:_})) => counts.cleanup += 1,
			Ok(Some(Message::Pickup{m:_})) => counts.pickup += 1,
			Ok(Some(Message::Forward{m:_}))=> counts.forward += 1,
			Ok(Some(Message::ForwardError{m:_}))=> counts.forwarderror += 1,
			Ok(Some(Message::Qmgr{m:_})) => counts.qmgr += 1,
			Ok(Some(Message::QmgrRemoved{m:_})) => counts.qmgrremoved += 1,
			Ok(Some(Message::QmgrExpired{m:_})) => counts.qmgrexpired += 1,
			Ok(Some(Message::Smtpd{m:_}))=> counts.smtpd += 1,
			Ok(Some(Message::SmtpdForward{m:_}))=> counts.smtpdforward += 1,
			Ok(Some(Message::SmtpdLogin{m:_}))=> counts.smtpdlogin += 1,
			Ok(Some(Message::Reject{m:_}))=> counts.rejects += 1,
			Err(x) => {print(&counts); panic!("Failure {} on {}", x, line)},
		};
	};
	print(&counts);
}
