use std::{
	fs::{self, File},
	io::Read,
	path::Path,
};

mod util;
use util::loc_counter;

fn f_loc(mut f: File, buf: &mut Vec<u8>) -> usize {
	let cap = buf.capacity();
	// ensure mem is not dirty before mutation,
	// and ensure `reserve` is absolute rather than relative.
	buf.clear();
	match buf.try_reserve_exact(
		f.metadata()
			.map(|m| usize::try_from(m.len()).unwrap_or(usize::MAX))
			.unwrap_or(1),
	) {
		Ok(v) => v,
		Err(e) => {
			eprint!("{e}");
			debug_assert_eq!(cap, buf.capacity());
			// just-in-case the mem-pressure is extreme
			buf.shrink_to(cap.div_ceil(2));
			return 0;
		}
	}

	// NOTE: partial-buffering on an array might be better.
	// mem-maps are an alternative.
	// NOTE: it could be faster to UTF8-validate the bytes
	// as they're loading into the buffer.
	match f.read_to_end(buf) {
		Ok(_) => {
			// NOTE: The optimizer may not merge
			// the UTF8-validation loop with the line-counting loop,
			// so there's no guarantee this is single-pass.
			if let Ok(s) = str::from_utf8(buf) {
				return loc_counter(s);
			}
			// ignore non-UTF8, to be more quiet
		}
		Err(e) => {
			eprintln!("{e}");
		}
	}
	0
}

fn recursive_loc(p: &Path, buf: &mut Vec<u8>) -> usize {
	let mut total = 0;
	// TO-DO: lock before checking if it's a dir
	if !p.is_dir() {
		return f_loc(
			match File::open(p) {
				Ok(f) => f,
				Err(e) => {
					eprintln!("{e}");
					return total;
				}
			},
			buf,
		);
	}
	let dir = match fs::read_dir(p) {
		Ok(d) => d,
		Err(e) => {
			eprintln!("{e}");
			return total;
		}
	};
	for entry in dir {
		let entry = match entry {
			Ok(p) => p,
			Err(e) => {
				eprintln!("{e}");
				continue;
			}
		};
		let path = entry.path();
		// TO-DO: lock before checking if it's a dir
		if path.is_dir() {
			match total.checked_add(recursive_loc(&path, buf)) {
				Some(t) => total = t,
				_ => return usize::MAX,
			}
		} else {
			match File::open(entry.path()) {
				Ok(f) => match total.checked_add(f_loc(f, buf)) {
					Some(t) => total = t,
					_ => return usize::MAX,
				},
				Err(e) => {
					eprintln!("{e}");
				}
			}
		}
	}
	total
}

fn main() {
	let argv: Box<[_]> = std::env::args_os().skip(1).collect();

	// Shared mem "arena". Reduces re-allocs
	let mut pool: Vec<u8> = vec![];

	if argv.is_empty() {
		// Not `ExactSizeIterator`, not even `DoubleEndedIterator`,
		// so the `collect` may allocate multiple times
		let mut ls: Box<[(_, _)]> = std::fs::read_dir("./")
			.unwrap()
			.map(|p| {
				let p = p.unwrap().path();
				(recursive_loc(&p, &mut pool), p)
			})
			.collect();

		// in-place, to reduce probability of OOM `panic`
		ls.sort_unstable_by_key(|(c, _)| *c);
		ls.reverse();

		for (c, p) in ls {
			// TO-DO: fmt the whole list,
			// then print all at once, for better perf
			println!("{}\t{}", c, p.display());
		}
	} else {
		// should user input be canonicalized?
		for p in argv {
			// print the stats as soon as they are computed
			println!("{}\t{}", recursive_loc(p.as_ref(), &mut pool), p.display());
			// TO-DO: use `stdout` to avoid re-locking
		}
	}
}
