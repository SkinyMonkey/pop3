//! CPSCR — original AI script bytecode decoder.
//!
//! orig: AI_RunScript `0x4c5eb0` (interpreter). Format derived & corpus-validated in
//! `docs/specs/v2/ai_script_vm.md` (§3 memory map, §4 opcodes) and
//! `ai_script_commands.md` (§2.2 stream arity).
//!
//! Each `cpscr*.dat` is a **dumped per-tribe script-context image** of exactly
//! `0x3108` bytes (the context stride):
//!
//! | Region | Bytes | Contents |
//! |---|---|---|
//! | bytecode | `0x0002 ..= SCRIPT-END` | 16-bit LE token stream (opcodes + operand indices) |
//! | descriptors | `0x2000 .. 0x3000` | operand table: 8-byte `{ i32 kind; i32 value }` entries |
//! | variables | `0x3000 ..` | script-local i32 slots |
//!
//! Token `[0]` is a header/length word; the IP starts at token `[1]` (= ctx+0x2).
//! Operand tokens in the bytecode are **indices** into the descriptor table, not
//! immediates — resolution (descriptor kind → value) is the VM's job, not the parser's.

const BYTECODE_START: usize = 1; // token index; ctx+0x2
const DESC_TABLE_OFF: usize = 0x2000; // byte offset of the operand-descriptor table
const VAR_TABLE_OFF: usize = 0x3000; // byte offset where descriptors end / vars begin

/// A statement opcode (`ai_script_vm.md` §4). `Command` carries the catalog
/// sub-opcode dispatched through `AI_ExecuteScriptCommand` (§7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    If,
    Else,
    EndIf, // 0x3ea — found by corpus survey, omitted from the original §4 draft
    BlockBegin,
    BlockEnd,
    Guard,
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Lt,
    Eq,
    Ne,
    Ge,
    Le,
    And,
    Or,
    Command(u16),
}

impl Op {
    /// Map a statement token to an `Op` (excludes `COMMAND` 0x3ee, handled inline,
    /// and `SCRIPT-END` 0x3fb, which terminates decoding).
    fn from_token(t: u16) -> Option<Op> {
        Some(match t {
            0x3e8 => Op::If,
            0x3e9 => Op::Else,
            0x3ea => Op::EndIf,
            0x3eb => Op::BlockBegin,
            0x3ec => Op::BlockEnd,
            0x3ed => Op::Guard,
            0x3ef => Op::Set,
            0x3f0 => Op::Add,
            0x3f1 => Op::Sub,
            0x3f4 => Op::Gt,
            0x3f5 => Op::Lt,
            0x3f6 => Op::Eq,
            0x3f7 => Op::Ne,
            0x3f8 => Op::Ge,
            0x3f9 => Op::Le,
            0x3fc => Op::And,
            0x3fd => Op::Or,
            0x401 => Op::Mul,
            0x402 => Op::Div,
            _ => return None,
        })
    }
}

/// Number of stream operand tokens an assignment/comparison statement consumes.
/// `Guard` is variable (1 or 2) and handled separately in [`decode`].
fn statement_operands(op: Op) -> usize {
    match op {
        Op::Set | Op::Add | Op::Sub => 2,
        // MUL/DIV consume 3 stream tokens (corpus-measured, unanimous), not the 2 the
        // spec's `dst OP= value` implied — likely a ternary `dst = a OP b` form (they
        // route through AI_ProcessSubroutineCall 0x4c8590, not the SET/ADD/SUB handler).
        Op::Mul | Op::Div => 3,
        Op::Gt | Op::Lt | Op::Eq | Op::Ne | Op::Ge | Op::Le => 2,
        _ => 0,
    }
}

/// Stream-token arity of a command sub-opcode = evaluated operands **plus** inline
/// literal tokens (set/clear selectors, mode sub-tokens). Measured from all 59
/// shipped scripts (unanimous, no ambiguity); see `ai_script_commands.md` §2.2.
/// Opcodes never emitted by a shipped script default to 0.
pub fn command_arity(sub: u16) -> usize {
    match sub {
        0x404 | 0x405 | 0x406 | 0x407 | 0x408 | 0x409 | 0x40a | 0x40b | 0x40c | 0x40d => 1,
        0x40f | 0x411 | 0x413 | 0x414 | 0x415 | 0x416 | 0x417 | 0x418 | 0x41a | 0x41b => 1,
        0x40e => 3,
        0x423 => 13,
        0x428 => 3,
        0x42a | 0x42b => 1,
        0x42c => 4,
        0x42d => 2,
        0x431 | 0x432 => 1,
        0x434 | 0x435 => 3,
        0x439 => 1,
        0x43a => 2,
        0x43c => 7,
        0x43d => 2,
        0x43e => 1,
        0x443 => 7,
        0x444 | 0x445 => 4,
        0x446 => 1,
        0x447 | 0x448 => 2,
        0x449 => 3,
        0x44a => 3,
        0x44d => 6,
        0x44e => 1,
        0x452 => 6,
        0x454 => 6,
        0x457 => 3,
        0x45b => 2,
        0x462 => 5,
        0x46b => 3,
        0x46c | 0x46f => 1,
        0x472 => 2,
        0x473 | 0x474 => 1,
        0x476 => 2,
        0x477 => 1,
        0x478 => 2,
        0x47f => 1,
        0x483 | 0x484 | 0x487 | 0x488 => 1,
        0x48b => 2,
        0x48c | 0x48d => 1,
        0x48e => 4,
        0x490 => 2,
        0x493 => 2,
        0x494 => 1,
        0x495 => 2,
        0x496 | 0x498 => 1,
        0x499 => 4,
        0x49b => 1,
        0x4a6 => 3,
        0x4a8 => 2,
        0x4ac => 1,
        0x4af => 3,
        0x4b0 => 1,
        0x4b2 => 1,
        0x4b4 => 1,
        0x4b8 => 1,
        0x4b9 => 4,
        0x4ba | 0x4bb => 3,
        0x4bd => 5,
        0x4be => 4,
        0x4c0 => 3,
        0x4c5 | 0x4c6 | 0x4c7 => 1,
        _ => 0,
    }
}

/// An operand-descriptor table entry (`ctx+0x2000`, 8 bytes). `kind`: 0=immediate,
/// 1=variable slot, 2=internal game-state code (`ai_script_vm.md` §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    pub kind: i32,
    pub value: i32,
}

/// A decoded bytecode statement. `operands` are raw descriptor indices (or inline
/// literal tokens for flag/selector commands); resolve them against [`Self`] only
/// in the VM, where the descriptor `kind` is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub op: Op,
    pub operands: Vec<u16>,
}

/// A parsed CPSCR script-context image.
#[derive(Debug, Clone)]
pub struct CpscrScript {
    pub instructions: Vec<Instruction>,
    pub descriptors: Vec<Descriptor>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CpscrError {
    /// Ran off the end of the token stream mid-instruction (no `SCRIPT-END`).
    Truncated,
    /// A token that is neither a known statement opcode nor a `COMMAND` prefix
    /// appeared where one was expected — a decode desync (offset = token index).
    UnexpectedToken { token_index: usize, token: u16 },
}

impl std::fmt::Display for CpscrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => write!(f, "cpscr bytecode truncated (no SCRIPT-END)"),
            Self::UnexpectedToken { token_index, token } => {
                write!(f, "cpscr decode desync at token {token_index}: unexpected 0x{token:x}")
            }
        }
    }
}
impl std::error::Error for CpscrError {}

fn read_u16(bytes: &[u8], tok: usize) -> Option<u16> {
    let b = tok * 2;
    Some(u16::from_le_bytes([*bytes.get(b)?, *bytes.get(b + 1)?]))
}

/// Decode the bytecode region into instructions, stopping at `SCRIPT-END` (0x3fb).
fn decode(bytes: &[u8]) -> Result<Vec<Instruction>, CpscrError> {
    let mut out = Vec::new();
    let mut i = BYTECODE_START;
    loop {
        let tok = read_u16(bytes, i).ok_or(CpscrError::Truncated)?;
        i += 1;
        if tok == 0x3fb {
            return Ok(out); // SCRIPT-END
        }
        let (op, n) = if tok == 0x3ee {
            // COMMAND: next token is the catalog sub-opcode.
            let sub = read_u16(bytes, i).ok_or(CpscrError::Truncated)?;
            i += 1;
            (Op::Command(sub), command_arity(sub))
        } else if let Some(op) = Op::from_token(tok) {
            if op == Op::Guard {
                // descA, plus optional descB present iff the next token isn't `{`.
                let a = read_u16(bytes, i).ok_or(CpscrError::Truncated)?;
                i += 1;
                let mut operands = vec![a];
                let peek = read_u16(bytes, i).ok_or(CpscrError::Truncated)?;
                if peek != 0x3eb {
                    operands.push(peek);
                    i += 1;
                }
                out.push(Instruction { op, operands });
                continue;
            }
            (op, statement_operands(op))
        } else {
            return Err(CpscrError::UnexpectedToken { token_index: i - 1, token: tok });
        };
        let mut operands = Vec::with_capacity(n);
        for _ in 0..n {
            operands.push(read_u16(bytes, i).ok_or(CpscrError::Truncated)?);
            i += 1;
        }
        out.push(Instruction { op, operands });
    }
}

/// Parse the operand-descriptor table (`0x2000 .. 0x3000`). Tolerates a short image
/// by reading as many whole 8-byte entries as the buffer holds.
fn parse_descriptors(bytes: &[u8]) -> Vec<Descriptor> {
    let end = bytes.len().min(VAR_TABLE_OFF);
    let mut out = Vec::new();
    let mut b = DESC_TABLE_OFF;
    while b + 8 <= end {
        let kind = i32::from_le_bytes(bytes[b..b + 4].try_into().unwrap());
        let value = i32::from_le_bytes(bytes[b + 4..b + 8].try_into().unwrap());
        out.push(Descriptor { kind, value });
        b += 8;
    }
    out
}

/// Parse a raw `cpscr*.dat` context image into its bytecode + descriptor table.
pub fn parse(bytes: &[u8]) -> Result<CpscrScript, CpscrError> {
    Ok(CpscrScript {
        instructions: decode(bytes)?,
        descriptors: parse_descriptors(bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a bytecode-only image from tokens (header + tokens), zero-padded so the
    /// descriptor region is addressable.
    fn image(tokens: &[u16]) -> Vec<u8> {
        let mut v = vec![0u8; VAR_TABLE_OFF + 8];
        v[0..2].copy_from_slice(&0xCu16.to_le_bytes()); // header word
        for (k, &t) in tokens.iter().enumerate() {
            let b = (BYTECODE_START + k) * 2;
            v[b..b + 2].copy_from_slice(&t.to_le_bytes());
        }
        v
    }

    #[test]
    fn decodes_if_block_command_and_assignment() {
        // { IF (==  #0 #1) { CMD 0x495 #2 #3 } } END
        let toks = [
            0x3eb, 0x3e8, 0x3f6, 0, 1, 0x3eb, 0x3ee, 0x495, 2, 3, 0x3ec, 0x3ec, 0x3fb,
        ];
        let s = parse(&image(&toks)).unwrap();
        let ops: Vec<Op> = s.instructions.iter().map(|i| i.op).collect();
        assert_eq!(
            ops,
            vec![
                Op::BlockBegin, Op::If, Op::Eq, Op::BlockBegin,
                Op::Command(0x495), Op::BlockEnd, Op::BlockEnd,
            ]
        );
        assert_eq!(s.instructions[2].operands, vec![0, 1]); // ==  #0 #1
        assert_eq!(s.instructions[4].operands, vec![2, 3]); // CMD 0x495 (arity 2)
    }

    #[test]
    fn stops_at_script_end() {
        // SET= #5 #6 ; END ; (garbage after END must be ignored)
        let toks = [0x3ef, 5, 6, 0x3fb, 0x9999, 0x8888];
        let s = parse(&image(&toks)).unwrap();
        assert_eq!(s.instructions.len(), 1);
        assert_eq!(s.instructions[0].op, Op::Set);
    }

    #[test]
    fn guard_is_variable_arity() {
        // GUARD #48 #108 { ... }  (descB present, next token is not `{`)
        let two = parse(&image(&[0x3ed, 48, 108, 0x3eb, 0x3ec, 0x3fb])).unwrap();
        assert_eq!(two.instructions[0].operands, vec![48, 108]);
        // GUARD #48 { ... }  (descB absent, `{` follows immediately)
        let one = parse(&image(&[0x3ed, 48, 0x3eb, 0x3ec, 0x3fb])).unwrap();
        assert_eq!(one.instructions[0].operands, vec![48]);
    }

    #[test]
    fn flag_op_consumes_its_selector_literal() {
        // CMD 0x409 [0x3ff]  — arity 1 (the set/clear selector token), then END
        let s = parse(&image(&[0x3ee, 0x409, 0x3ff, 0x3fb])).unwrap();
        assert_eq!(s.instructions[0].op, Op::Command(0x409));
        assert_eq!(s.instructions[0].operands, vec![0x3ff]);
    }

    #[test]
    fn desync_on_unknown_token_errors() {
        // SET= #5 #6, then 0x0000 (zero-pad) where a statement opcode is expected.
        let toks: [u16; 3] = [0x3ef, 5, 6];
        assert!(matches!(
            parse(&image(&toks)),
            Err(CpscrError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn truncated_buffer_errors() {
        // Only the header word — no room for even one token.
        assert!(matches!(parse(&0xCu16.to_le_bytes()), Err(CpscrError::Truncated)));
    }

    /// Integration: decode a real shipped script if the game data is present.
    /// All 59 shipped `cpscr*.dat` decode with no unrecognised tokens given the
    /// corpus-measured arity table (incl. MUL/DIV = 3 operands); see
    /// `ai_script_vm.md` §4.0.
    #[test]
    fn decodes_real_cpscr012_if_present() {
        let path = "../../data/original_game/levels/cpscr012.dat";
        let Ok(bytes) = std::fs::read(path) else {
            return; // game data not linked in this environment — skip
        };
        let s = parse(&bytes).expect("cpscr012 should decode cleanly");
        assert!(s.instructions.len() > 100, "expected a substantial script");
        // First real statement is BLOCK-BEGIN (header word then 0x3eb).
        assert_eq!(s.instructions[0].op, Op::BlockBegin);
        // Descriptor table is the 0x2000..0x3000 region = 512 entries of 8 bytes.
        assert_eq!(s.descriptors.len(), 512);
    }
}
