use bitcoin::{opcodes::all::{OP_PUSHNUM_13, OP_RETURN}, script::{Builder, PushBytesBuf}, ScriptBuf};
use candid::CandidType;
use leb128::write;
use serde::{Deserialize, Serialize};



#[allow(dead_code)]
const MAX_DIVISIBILITY: u8 = 38;
#[allow(dead_code)]
const MAX_SPACERS: u32 = 0b00000111111111111111111111111111;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    #[allow(dead_code)]
    Body = 0,
    Flags = 2,
    Rune = 4,
    Premine = 6,
    Cap = 8,
    Amount = 10,
    HeightStart = 12,
    HeightEnd = 14,
    OffsetStart = 16,
    OffsetEnd = 18,
    #[allow(dead_code)]
    Mint = 20,
    #[allow(dead_code)]
    Pointer = 22,
    #[allow(dead_code)]
    Cenotaph = 126,
    // Odd tags
    Divisibility = 1,
    Spacers = 3,
    Symbol = 5,
    #[allow(dead_code)]
    Nop = 127,

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flag {
    Etching = 0,
    Terms = 1,
    Turbo = 2,
    #[allow(dead_code)]
    Cenotaph = 127,
}

impl Flag {
    fn mask(self) -> u128 {
        let position = match self {
            Flag::Etching => 0,
            Flag::Terms => 1,
            Flag::Turbo => 2,
            Flag::Cenotaph => 127,
        };
        1 << position
    }
}

pub fn encode_leb128(value : u64)->Vec<u8>{
    let mut buf = Vec::new();
    write::unsigned(&mut buf, value).unwrap();
    buf
}

pub fn encode_rune_name(name : &str)->Result<u64,String>{
    if name.is_empty(){
        return Err("Rune name cannot be empty".to_string());
    }

    let mut value = 0u64;
    for(i,ch) in name.chars().enumerate(){
        if i >=28{
            return Err("Rune name cannot exceed 28 characters".to_string());
        }
        if !ch.is_ascii_uppercase() {
            return Err("Rune name must contain only uppercase letters A-Z".to_string());
        }
        let digit = (ch as u8 - b'A') as u64;
        if i == 0{
            value = digit;
        }else{
            value = value
            .checked_add(1)
            .and_then(|v| v.checked_mul(26))
            .and_then(|v|v.checked_add(digit))
            .ok_or("Rune Name Value Overflow")?;
        }
    }
    Ok(value)
}
pub struct Etching {
    pub divisibility: u8,
    pub premine: u128,
    pub rune_name: String,
    pub symbol: Option<char>,
    pub terms: Option<Terms>,
    pub turbo: bool,
    pub spacers: u32,
    pub edicts: Option<Vec<Edict>>,
}

#[derive(Debug,Clone,Deserialize,CandidType,Serialize)]
pub struct Edict{
    pub amount: u128, // Amount to mint,
    pub output : u32
}
pub struct Terms {
    pub amount: Option<u128>,               // Amount per mint
    pub cap: Option<u128>,                  // Maximum number of mints
    pub height: (Option<u64>, Option<u64>), // Absolute block height range
    pub offset: (Option<u64>, Option<u64>), // Relative block height range
}


pub fn build_etching_script(etching : &Etching)->Result<ScriptBuf,String>{
    let mut payload = Vec::new();
    let encoded_name = encode_rune_name(&etching.rune_name)?;
    let mut flags = Flag::Etching.mask();
    if etching.terms.is_some(){
        flags |=Flag::Terms.mask();
    }

    if etching.turbo{
        flags |= Flag::Turbo.mask();

    }

    if etching.divisibility > 0{
        payload.extend_from_slice(&encode_leb128(Tag::Divisibility as u64));
        payload.extend_from_slice(&encode_leb128(etching.divisibility as u64));
    }
    payload.extend_from_slice(&encode_leb128(Tag::Flags as u64));
    payload.extend_from_slice(&encode_leb128(flags as u64));

    if etching.spacers > 0 {
        payload.extend_from_slice(&encode_leb128(Tag::Spacers as u64));
        payload.extend_from_slice(&encode_leb128(etching.spacers as u64));
    }

    // Tag 4: Rune name
    payload.extend_from_slice(&encode_leb128(Tag::Rune as u64));
    payload.extend_from_slice(&encode_leb128(encoded_name as u64));

    // Tag 5: Symbol (odd tag)
    if let Some(symbol) = etching.symbol {
        payload.extend_from_slice(&encode_leb128(Tag::Symbol as u64));
        payload.extend_from_slice(&encode_leb128(symbol as u64));
    }

    // Tag 6: Premine
    if etching.premine > 0 {
        payload.extend_from_slice(&encode_leb128(Tag::Premine as u64));
        payload.extend_from_slice(&encode_leb128(etching.premine as u64));
    }

    // Tag 0: Body marker
    payload.extend_from_slice(&encode_leb128(0));

   


if let Some(terms) = &etching.terms {
    if terms.amount.is_some() {
        payload.extend_from_slice(&encode_leb128(Tag::Amount as u64));
        payload.extend_from_slice(&encode_leb128(terms.amount.unwrap() as u64));
    }
    if terms.cap.is_some() {
        payload.extend_from_slice(&encode_leb128(Tag::Cap as u64));
        payload.extend_from_slice(&encode_leb128(terms.cap.unwrap() as u64));
    }
    if let Some(start) = terms.height.0 {
        payload.extend_from_slice(&encode_leb128(Tag::HeightStart as u64));
        payload.extend_from_slice(&encode_leb128(start));
    }
    if let Some(end) = terms.height.1 {
        payload.extend_from_slice(&encode_leb128(Tag::HeightEnd as u64));
        payload.extend_from_slice(&encode_leb128(end));
    }
    if let Some(start) = terms.offset.0 {
        payload.extend_from_slice(&encode_leb128(Tag::OffsetStart as u64));
        payload.extend_from_slice(&encode_leb128(start));
    }
    if let Some(end) = terms.offset.1 {
        payload.extend_from_slice(&encode_leb128(Tag::OffsetEnd as u64));
        payload.extend_from_slice(&encode_leb128(end));
    }
}


// 4. Place all edicts after the body marker
if let Some(edicts) = &etching.edicts {
    for edict in edicts {
        payload.extend_from_slice(&encode_leb128(0));
        payload.extend_from_slice(&encode_leb128(0));
        payload.extend_from_slice(&encode_leb128(edict.amount as u64));
        payload.extend_from_slice(&encode_leb128(edict.output as u64));
    }
}
let mut builder = Builder::new().push_opcode(OP_RETURN);

    // Add OP_13 marker
    builder = builder.push_opcode(OP_PUSHNUM_13);
    ic_cdk::println!("Final payload bytes: {:?}", payload);
    for (i, byte) in payload.iter().enumerate() {
    ic_cdk::println!("{:02}: {}", i, byte);
}

    // Add the entire payload as a single data push.
    // Critical: All runestone data must be in one push after OP_13,
    // not split into multiple chunks, per the Runes protocol specification.
    let mut push_bytes = PushBytesBuf::new();
    push_bytes
        .extend_from_slice(&payload)
        .map_err(|_| "Failed to create push bytes - payload may be too large")?;
    builder = builder.push_slice(&push_bytes);

    Ok(builder.into_script())
}