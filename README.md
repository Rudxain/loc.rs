# LOC
Recursively counts non-empty lines that contain at least 1 non-whitespace character.

It uses the [Unicode definition of "whitespace" (according to Rust-std-lib)](https://doc.rust-lang.org/std/primitive.char.html#method.is_whitespace), and the [POSIX definition of "line"](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap03.html#tag_03_185). Yeah, very inconsistent, I know.

> [!WARNING]
> This might stay sem-ver unstable for a while. Even the **commit hashes are subject to change**

I have no plans of supporting source-comments (exclude comments from counts), as that requires language-awareness. I want this to be useful enough, while being "simple" and lang-agnostic.

If you really want lang-support, I recommend [Tokei](https://github.com/XAMPPRocky/tokei). One of the reasons why I wrote this program was because [`cloc`](https://github.com/AlDanial/cloc) is too complex and slow. See also: [`uwc`](https://github.com/dead10ck/uwc) and [`sloc`](https://dwheeler.com/sloccount).

## Usage

### Install
This needs a `rustc` and `cargo` (tested on `nightly`, but should work with `stable`). Recommended command:
```sh
cargo install --path . --config 'build.rustflags="-C target-cpu=native"'
```
Assuming you've downloaded and `cd`ed into the repo

### Run
Invoke the program by passing the paths whose counts you want to get:
```sh
# example
loc file.txt smol-directory 'BIG dir/'
16	file.txt
255	smol-directory
20069	BIG dir/
# stats are printed as soon as each count is computed
```
Or simply pass nothing, if you want _sorted_ stats about WD (equivalent to `loc ./* .[!.]* | sort -rn`), but you'll have to wait until all results are computed. This is kinda similar to how `du` works, even the output format is similar; both are intentional design decisions.

This program is single-threaded, as it's IO-bound.

Unlike most CLIs, this program doesn't recognize any options or flags (yet), so arguments prefixed with `-` are treated as any other arg. I might add `--help` & `-h` flags, but that seems overkill.

Non-UTF8 args are supported, to allow arbitrary file-names, and for lower startup overhead. However, **non-UTF8 files are excluded** from counts, as the concept of "non-blank line" doesn't exist in raw-binary (according to Unicode, not POSIX). Sadly, this also excludes UTF-{16,32}.

> Side-note: I was **blown away** by how blazing-fast this program is, **even in debug mode!**
> At least, when compared to [the equivalent shell-script](loc.sh).
>
> No wonder [`rg`](https://github.com/BurntSushi/ripgrep) is so fast! 🚀
