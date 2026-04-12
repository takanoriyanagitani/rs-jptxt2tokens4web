use std::sync::OnceLock;
use std::sync::RwLock;

use std::io;

use io::Cursor;
use io::Write;

use lindera::token::Token;
use lindera::tokenizer::Tokenizer;

use lindera::mode::Mode;
use lindera::segmenter::Segmenter;

use lindera::dictionary::Dictionary;
use lindera::dictionary::load_dictionary;

static INPUT_TEXT: RwLock<[u8; 65536]> = RwLock::new([0; 65536]);

static OUTPUT_JSON_ARR: RwLock<[u8; 65536]> = RwLock::new([0; 65536]);

static TOKENS: RwLock<Vec<String>> = RwLock::new(vec![]);

static TOKENIZER: OnceLock<Option<Tokenizer>> = OnceLock::new();

pub fn input_txt_ptr(i: &[u8]) -> *const u8 {
    i.as_ptr()
}

pub fn output_txt_ptr(o: &[u8]) -> *const u8 {
    o.as_ptr()
}

pub fn input_txt_ptr_s() -> Result<*const u8, io::Error> {
    let guard = INPUT_TEXT
        .try_read()
        .map_err(|_| io::Error::other("unable to read lock"))?;
    let i: &[u8; 65536] = &guard;
    Ok(input_txt_ptr(&i[..]))
}

pub fn output_txt_ptr_s() -> Result<*const u8, io::Error> {
    let guard = OUTPUT_JSON_ARR
        .try_read()
        .map_err(|_| io::Error::other("unable to read lock"))?;
    let o: &[u8; 65536] = &guard;
    Ok(output_txt_ptr(&o[..]))
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn input_txt_ptr_e() -> *const u8 {
    input_txt_ptr_s().unwrap_or(std::ptr::null())
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn output_txt_ptr_e() -> *const u8 {
    output_txt_ptr_s().unwrap_or(std::ptr::null())
}

pub fn tokenizer() -> Option<&'static Tokenizer> {
    TOKENIZER
        .get_or_init(|| {
            let dict: Dictionary = load_dictionary("embedded://ipadic").ok()?;
            let segm: Segmenter = Segmenter::new(Mode::Normal, dict, None);
            Some(Tokenizer::new(segm))
        })
        .as_ref()
}

pub fn tokens2json2writer<W>(tokens: &[String], wtr: &mut W) -> Result<(), io::Error>
where
    W: Write,
{
    serde_json::to_writer(wtr, tokens)?;
    Ok(())
}

pub fn tokens2json2page(tokens: &[String], page: &mut [u8; 65536]) -> Result<u64, io::Error> {
    let mut cursor: Cursor<_> = Cursor::new(&mut page[..]);
    tokens2json2writer(tokens, &mut cursor)?;
    Ok(cursor.position())
}

pub fn tokens2json2page_s() -> Result<u64, io::Error> {
    let guard = TOKENS
        .try_read()
        .map_err(|_| io::Error::other("unable to read lock"))?;
    let tokens: &[String] = &guard;

    let mut mguard = OUTPUT_JSON_ARR
        .try_write()
        .map_err(|_| io::Error::other("unable to write lock"))?;
    let page: &mut [u8; 65536] = &mut mguard;

    tokens2json2page(tokens, page)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn tokens2json2page_e() -> i32 {
    tokens2json2page_s()
        .ok()
        .and_then(|u| u.try_into().ok())
        .unwrap_or(-1)
}

pub fn txt2tokens(
    txt: &str,
    tok: &'static Tokenizer,
    buf_tokens: &mut Vec<String>,
) -> Result<usize, io::Error> {
    let tokens: Vec<Token> = tok.tokenize(txt).map_err(io::Error::other)?;

    let overwrite_cnt: usize = buf_tokens.len().min(tokens.len());

    if buf_tokens.len() < tokens.len() {
        buf_tokens.reserve(tokens.len() - buf_tokens.len());
    }

    let dst_ow: &mut [String] = &mut buf_tokens[..overwrite_cnt];
    let src_ow: &[Token] = &tokens[..overwrite_cnt];

    for (dst, src) in dst_ow.iter_mut().zip(src_ow) {
        let sur: &str = &src.surface;
        dst.clear();
        dst.push_str(sur);
    }

    let src_new: &[Token] = &tokens[overwrite_cnt..];

    buf_tokens.extend(src_new.iter().map(|tok: &Token| {
        let sur: &str = &tok.surface;
        String::from(sur)
    }));

    Ok(tokens.len())
}

pub fn txt2tokens_s(input_size: u32) -> Result<usize, io::Error> {
    let gdat = INPUT_TEXT
        .try_read()
        .map_err(|_| io::Error::other("unable to read lock"))?;
    let adat: &[u8; 65536] = &gdat;
    let input_sz: usize = input_size as usize;
    let dat: &[u8] = &adat[..input_sz];
    let txt: &str = std::str::from_utf8(dat).unwrap_or_default();

    let otok: Option<&'static Tokenizer> = tokenizer();
    let tok: &'static Tokenizer = otok.ok_or(io::Error::other("no tokenizer available"))?;

    let mut mgdat = TOKENS
        .try_write()
        .map_err(|_| io::Error::other("unable to write lock"))?;
    let mtok: &mut Vec<String> = &mut mgdat;

    txt2tokens(txt, tok, mtok)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn txt2tokens_e(input_size: u32) -> i32 {
    txt2tokens_s(input_size)
        .ok()
        .and_then(|u| u.try_into().ok())
        .unwrap_or(-1)
}

pub fn txt2tokens2json2page_s(input_size: u32) -> Result<u64, io::Error> {
    let tok_cnt: usize = txt2tokens_s(input_size)?;
    let gtoks = TOKENS
        .try_read()
        .map_err(|_| io::Error::other("unable to write lock"))?;
    let toks: &[String] = &gtoks;
    let limited: &[String] = &toks[..tok_cnt];

    let mut gpage = OUTPUT_JSON_ARR
        .try_write()
        .map_err(|_| io::Error::other("unable to write lock"))?;
    let mpage: &mut [u8; 65536] = &mut gpage;
    tokens2json2page(limited, mpage)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn txt2tokens2json2page_e(input_size: u32) -> i32 {
    txt2tokens2json2page_s(input_size)
        .ok()
        .and_then(|u| u.try_into().ok())
        .unwrap_or(-1)
}

#[cfg(feature = "include-dummy-start")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() {}
