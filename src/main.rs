use std::{
	fs::{self, File},
	io::{self, Read, Write},
	path::Path,
};

mod util;
use util::loc_counter;

fn f_loc(mut f: File, buf: &mut Vec<u8>, err: &mut io::StderrLock) -> io::Result<usize> {
	let cap = buf.capacity();
	// ensure mem is not dirty before mutation,
	// and ensure `reserve` is absolute rather than relative.
	buf.clear();
	match buf.try_reserve_exact(
		f.metadata()
			.map(|m| usize::try_from(m.len()).unwrap_or(usize::MAX))
			.unwrap_or(0),
	) {
		Ok(v) => v,
		Err(e) => {
			writeln!(err, "{e}")?;
			err.flush()?;
			debug_assert_eq!(cap, buf.capacity());
			// just-in-case the mem-pressure is extreme
			buf.shrink_to(cap.div_ceil(2));
			return Ok(0);
		}
	}

	// TO-DO: UTF8-validate the bytes as they're loading into a buffer,
	// to return early.
	// NOTE: consider mem-maps as an alt to buffering.
	match f.read_to_end(buf) {
		Ok(_) => {
			// NOTE: The optimizer may not merge
			// the UTF8-validation loop with the line-counting loop,
			// so there's no guarantee this is single-pass.
			if let Ok(s) = str::from_utf8(buf) {
				return Ok(loc_counter(s));
			}
			// ignore non-UTF8, to be more quiet
		}
		Err(e) => {
			writeln!(err, "{e}")?;
			err.flush()?;
		}
	}
	Ok(0)
}

fn recursive_loc(p: &Path, buf: &mut Vec<u8>, err: &mut io::StderrLock) -> io::Result<usize> {
	let mut total = 0;
	// TO-DO: lock before checking if it's a dir
	if !p.is_dir() {
		return f_loc(
			match File::open(p) {
				Ok(f) => f,
				Err(e) => {
					writeln!(err, "{e}")?;
					err.flush()?;
					return Ok(total);
				}
			},
			buf,
			err,
		);
	}
	let dir = match fs::read_dir(p) {
		Ok(d) => d,
		Err(e) => {
			writeln!(err, "{e}")?;
			err.flush()?;
			return Ok(total);
		}
	};
	for entry in dir {
		let entry = match entry {
			Ok(p) => p,
			Err(e) => {
				writeln!(err, "{e}")?;
				err.flush()?;
				continue;
			}
		};
		let path = entry.path();
		// TO-DO: lock before checking if it's a dir
		if path.is_dir() {
			match total.checked_add(recursive_loc(&path, buf, err)?) {
				Some(t) => total = t,
				_ => return Ok(usize::MAX),
			}
		} else {
			// assume as regular file.
			// this may be wrong.
			match File::open(entry.path()) {
				Ok(f) => match total.checked_add(f_loc(f, buf, err)?) {
					Some(t) => total = t,
					_ => return Ok(usize::MAX),
				},
				Err(e) => {
					writeln!(err, "{e}")?;
					err.flush()?;
				}
			}
		}
	}
	Ok(total)
}

fn main() -> io::Result<()> {
	let argv: Box<[_]> = std::env::args_os().skip(1).collect();

	// Shared mem "arena". Reduces re-allocs
	let mut pool: Vec<u8> = vec![];

	let mut out = io::stdout().lock();
	let mut err = io::stderr().lock();

	if argv.is_empty() {
		let mut ls = vec![];
		for p in fs::read_dir("./").unwrap() {
			let p = p.unwrap().path();
			ls.push((recursive_loc(&p, &mut pool, &mut err)?, p));
		}
		// Repeated insort is slower:
		// https://docs.python.org/3/library/bisect.html#bisect.insort_left

		// in-place, to reduce probability of OOM `panic`
		ls.sort_unstable_by_key(|(c, _)| *c);
		ls.reverse();

		out.write_all(
			ls.into_iter()
				.map(|(c, p)| format!("{}\t{}\n", c, p.display()))
				.collect::<Box<str>>() // auto-join
				.as_bytes(),
		)?;
		// implicit flush
	} else {
		// should user input be canonicalized?
		for p in argv {
			writeln!(
				out,
				"{}\t{}",
				recursive_loc(p.as_ref(), &mut pool, &mut err)?,
				p.display()
			)?;
			out.flush()?;
		}
	}
	Ok(())
}
