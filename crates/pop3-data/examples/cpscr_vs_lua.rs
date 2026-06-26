//! External validation: compare each original AI bytecode script (`cpscr*.dat`)
//! against its decompiled-to-Lua twin (`cpscr*.lua`).
//!
//! The Lua is the trust-level-6 reimplementation (the rewrite target); the bytecode
//! is ground truth. A faithful decompilation must preserve the program's structure,
//! so we compare two representation-independent metrics per pair:
//!   * **ifs**   — number of `IF` branches (unambiguous on both sides)
//!   * **stmts** — commands + assignments (the cmd/assignment split itself is
//!                 representation-dependent — `ATTR_X = n` is an assignment in Lua but
//!                 may be a command in bytecode — so we compare their *sum*, not each).
//! Pairs whose metrics diverge beyond a threshold are flagged for human review.
//!
//! Run: `cargo run -p pop3-data --example cpscr_vs_lua [BASE_DIR]`  (BASE_DIR default ".")

use pop3_data::data::cpscr::{self, Op};
use std::path::{Path, PathBuf};

struct Counts {
    cmd: usize,
    asg: usize,
    ifs: usize,
}
impl Counts {
    fn stmts(&self) -> usize {
        self.cmd + self.asg
    }
}

fn bytecode_counts(bytes: &[u8]) -> Result<Counts, cpscr::CpscrError> {
    let s = cpscr::parse(bytes)?;
    let mut c = Counts { cmd: 0, asg: 0, ifs: 0 };
    for ins in &s.instructions {
        match ins.op {
            Op::Command(_) => c.cmd += 1,
            Op::Set | Op::Add | Op::Sub | Op::Mul | Op::Div => c.asg += 1,
            Op::If => c.ifs += 1,
            _ => {}
        }
    }
    Ok(c)
}

/// Strip a trailing `-- comment` and surrounding whitespace.
fn code(line: &str) -> &str {
    let l = match line.find("--") {
        Some(i) => &line[..i],
        None => line,
    };
    l.trim()
}

fn starts_with_word(s: &str, w: &str) -> bool {
    s.strip_prefix(w)
        .is_some_and(|rest| rest.is_empty() || !rest.as_bytes()[0].is_ascii_alphanumeric())
}

/// `NAME(` where NAME is an UPPERCASE identifier — a PopScript command call.
fn is_command_call(s: &str) -> bool {
    let name: &str = s.split('(').next().unwrap_or("").trim_end();
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        && name.bytes().next().is_some_and(|b| b.is_ascii_uppercase() || b == b'_')
        && s.contains('(')
}

/// `lhs = rhs` (a single `=`, not `==`/`<=`/`>=`/`~=`) with an identifier lhs.
fn is_assignment(s: &str) -> bool {
    let Some(eq) = s.find('=') else { return false };
    let before = s.as_bytes().get(eq.wrapping_sub(1)).copied();
    let after = s.as_bytes().get(eq + 1).copied();
    if eq == 0
        || matches!(before, Some(b'<' | b'>' | b'~' | b'!' | b'='))
        || after == Some(b'=')
    {
        return false;
    }
    let lhs = s[..eq].trim();
    !lhs.is_empty()
        && lhs
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn lua_counts(src: &str) -> Counts {
    let mut c = Counts { cmd: 0, asg: 0, ifs: 0 };
    for line in src.lines() {
        let s = code(line);
        if s.is_empty() {
            continue;
        }
        if starts_with_word(s, "if") || starts_with_word(s, "elseif") {
            c.ifs += 1;
        } else if is_command_call(s) {
            c.cmd += 1;
        } else if is_assignment(s) {
            c.asg += 1;
        }
    }
    c
}

fn main() {
    let base = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let levels = Path::new(&base).join("data/original_game/levels");
    let scripts = Path::new(&base).join("data/scripts");

    let mut pairs: Vec<PathBuf> = std::fs::read_dir(&levels)
        .unwrap_or_else(|e| panic!("read {levels:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("cpscr") && n.ends_with(".dat"))
        })
        .collect();
    pairs.sort();

    println!(
        "{:<10}  {:>16}  {:>16}  {:>5} {:>6}  flag",
        "file", "bytecode if/stmt", "lua if/stmt", "Δif", "Δstmt"
    );
    let (mut flagged, mut undecodable, mut missing) = (0, 0, 0);
    for dat in &pairs {
        let stem = dat.file_stem().unwrap().to_str().unwrap();
        let lua = scripts.join(format!("{stem}.lua"));
        let Ok(lua_src) = std::fs::read_to_string(&lua) else {
            missing += 1;
            continue;
        };
        let bc = match bytecode_counts(&std::fs::read(dat).unwrap()) {
            Ok(c) => c,
            Err(e) => {
                println!("{stem:<10}  bytecode does not decode cleanly: {e}");
                undecodable += 1;
                continue;
            }
        };
        let lc = lua_counts(&lua_src);
        let d_if = lc.ifs as i64 - bc.ifs as i64;
        let d_st = lc.stmts() as i64 - bc.stmts() as i64;
        // Flag: control-flow drift, or >15% statement drift.
        let flag = d_if.abs() >= 5 || (d_st.abs() * 100) > (bc.stmts() as i64 * 15);
        if flag {
            flagged += 1;
        }
        println!(
            "{:<10}  {:>7}/{:<8}  {:>7}/{:<8}  {:>+5} {:>+6}  {}",
            stem,
            bc.ifs,
            bc.stmts(),
            lc.ifs,
            lc.stmts(),
            d_if,
            d_st,
            if flag { "*** DIVERGENT" } else { "" }
        );
    }
    println!(
        "\n{} pairs: {} flagged divergent, {} bytecode-undecodable, {} missing .lua",
        pairs.len(),
        flagged,
        undecodable,
        missing
    );
}
