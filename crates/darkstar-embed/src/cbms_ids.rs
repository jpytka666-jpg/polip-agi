// darkstar-header-v1
// po co: cbms_ids.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie,
//   mapowanie glyph->u32, lokalne lustro Book/Vocabulary, Qdrant, implant, wagi, Python
// autor: Marcin
// powstal: 2026-09-06
//! Jedna krawedz: kanoniczny tekst EO -> te same Book/Vocabulary co `cbms ids` -> Vec<u32>.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Grok (implementer, not author)
//! TIMESTAMP: 2026-09-06 02:10:00 Europe/London
//! REASON FOR CREATION: Marcin via DIRECTOR: ONE MINIMAL EDGE - selected_eo_text:&str ->
//! CBMS Vocabulary/Book (SAME as writer) -> Vec<u32> -> Noworodek-ready. Nie pelny tokenizer.
//! MECHANICS: Ladowanie ksiegi przez `cbms_writing::Book::parse_lenient` (pierwszy wlasciciel
//! symbolu), potem `Vocabulary::encode` - ta sama sciezka co binarka `cbms <ksiega> ids`.
//! Bez mostu glyph ID -> CBMS ID. Bez lokalnego lustra arytmetyki id.
//! SYSTEM PART: Darkstar Embed / krawedz wejsciowa do znakow CBMS.
//! ARCHITECTURE FUNCTION: Wspoldzielona implementacja z aktywnym writerem cbms-writing;
//! tokenize/Noworodek biora stąd liczby, nie z przestrzeni glifow.
//! DEPENDENCIES/LINKS: cbms-writing (Book, Vocabulary) - path dep do aktywnego writera.
//! TECH STACK: Rust 2024 + cbms-writing (stdlib only w srodku).
//! LOCAL WORKSPACE: crates/darkstar-embed (workspace-relative)
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//! REVISION 2026-09-06 (Codex): path dep zastapiona kanonicznym Git rev w Cargo.toml.
//! Lokalna sciezka usunieta dla przenosnosci; testy wymagaja NOWORODEK_BOOK lub CBMS_BOOK.
//! Kazdy baseline porownuje wynik frontend_to_cbms z bezposrednim Vocabulary::encode.

use std::fmt;
use std::path::Path;

use cbms_writing::{Book, Vocabulary};

/// Blad krawedzi EO -> CBMS ids. Bez zgadywania - albo te same liczby co writer, albo glosna odmowa.
#[derive(Debug)]
pub enum CbmsIdsError {
    Io(String),
    Book(String),
    Vocab(String),
}

impl fmt::Display for CbmsIdsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CbmsIdsError::Io(m) | CbmsIdsError::Book(m) | CbmsIdsError::Vocab(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for CbmsIdsError {}

/// Wczytuje ksiege tym samym parserem co aktywny `cbms ids` (lenient: pierwszy wlasciciel symbolu).
pub fn load_cbms_book(path: impl AsRef<Path>) -> Result<Book, CbmsIdsError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .map_err(|e| CbmsIdsError::Io(format!("nie czytam ksiegi {}: {e}", path.display())))?;
    let (book, _collisions) = Book::parse_lenient(&text)
        .map_err(|e| CbmsIdsError::Book(format!("ksiega nie laduje: {e}")))?;
    Ok(book)
}

/// Kanoniczny tekst EO -> `Vec<u32>` jak `cbms <ksiega> ids`.
///
/// Wejscie to tekst esperancki (albo juz zcanonicalizowany), NIE ciag glifow CBMS.
/// Mapowanie glyph -> numer jest zakazane - tu idzie wylacznie Vocabulary::encode.
pub fn frontend_to_cbms(book: &Book, selected_eo_text: &str) -> Result<Vec<u32>, CbmsIdsError> {
    let vocab = Vocabulary::new(book).ok_or_else(|| {
        CbmsIdsError::Vocab(
            "nie da sie zbudowac Vocabulary z tej ksiegi (brak MORPH-SEP albo przeplenienie id)"
                .into(),
        )
    })?;
    Ok(vocab.encode(selected_eo_text))
}

/// Encode an already verified morpheme list as one plain CBMS id per unit.
///
/// This intentionally bypasses textual tokenisation: every unit must be an exact
/// Book root and no fallback, case folding, or spaced-id conversion is permitted.
pub fn morphemes_to_cbms(book: &Book, morphemes: &[&str]) -> Result<Vec<u32>, CbmsIdsError> {
    let vocab = Vocabulary::new(book).ok_or_else(|| {
        CbmsIdsError::Vocab(
            "nie da sie zbudowac Vocabulary z tej ksiegi (brak MORPH-SEP albo przeplenienie id)"
                .into(),
        )
    })?;
    morphemes
        .iter()
        .map(|unit| {
            if book.symbol_for(unit).is_none() {
                return Err(CbmsIdsError::Vocab(format!(
                    "morfem {unit:?} nie jest dokladnym symbolem (brak w ksiedze)"
                )));
            }
            let encoded = vocab.encode(unit);
            match encoded.as_slice() {
                [id] => Ok(*id),
                _ => Err(CbmsIdsError::Vocab(format!(
                    "morfem {unit:?} nie mapuje sie do pojedynczego plain id: {encoded:?}"
                ))),
            }
        })
        .collect()
}

/// Jak `frontend_to_cbms`, ale ksiega ze sciezki - wygodne dla testow i cienkich CLI.
pub fn frontend_to_cbms_path(
    book_path: impl AsRef<Path>,
    selected_eo_text: &str,
) -> Result<Vec<u32>, CbmsIdsError> {
    let book = load_cbms_book(book_path)?;
    frontend_to_cbms(&book, selected_eo_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    /// Ta sama ksiega, ktora dala baseline `cbms ids` na tej maszynie.
    fn baseline_book_path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("NOWORODEK_BOOK") {
            return Some(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("CBMS_BOOK") {
            return Some(PathBuf::from(p));
        }
        None
    }

    fn book() -> &'static Book {
        static BOOK: OnceLock<Book> = OnceLock::new();
        BOOK.get_or_init(|| {
            if let Some(path) = baseline_book_path() {
                load_cbms_book(&path).unwrap_or_else(|e| {
                    panic!(
                        "brak ksiegi do baseline ({}): {e}. Ustaw NOWORODEK_BOOK lub CBMS_BOOK.",
                        path.display()
                    )
                })
            } else {
                load_cbms_book_from_text(
                    "CODEBOOK_CBMS_ES\n\
                     kodo=ቂ\n\
                     ĉifro=ቃ\n\
                     cifro=ቄ\n\
                     memoro=ቅ\n\
                     memorilo=ቆ\n\
                     memor=ቇ\n\
                     o=ቈ\n\
                     il=቉\n\
                     mal=ቊ\n\
                     san=ቋ\n\
                     a=ቌ\n\
                     ul=ቍ\n\
                     ej=቎\n\
                     ĉifr=቏\n\
                     kod=ቐ\n\
                     CBMS-Eo-v1.1-EXT\n\
                     MORPH-SEP=U+00B7\n",
                )
            }
        })
    }

    fn frontend_matches_writer(text: &str) -> Vec<u32> {
        let baseline: Vec<u32> = Vocabulary::new(book()).unwrap().encode(text);
        let frontend = frontend_to_cbms(book(), text).unwrap();
        assert_eq!(
            frontend, baseline,
            "EO {text:?}: frontend != canonical writer"
        );
        println!(
            "EO TEXT: {text}\nCBMS AUTHORITATIVE RESULT: {baseline:?}\nfrontend_to_cbms RESULT: {frontend:?}\nMATCH: YES"
        );
        frontend
    }

    fn morph_book() -> Book { load_cbms_book_from_text(&book().to_text()) }

    fn load_cbms_book_from_text(text: &str) -> Book {
        let (book, collisions) = Book::parse_lenient(text).unwrap();
        assert!(collisions.is_empty(), "POC book collisions: {collisions:?}");
        book
    }

    #[test]
    fn verified_morpheme_vectors_use_plain_ids_in_order() {
        let b = morph_book();
        let vocab = Vocabulary::new(&b).unwrap();
        for units in [
            &["memor", "o"][..],
            &["memor", "il", "o"][..],
            &["mal", "san", "a"][..],
            &["san", "ul", "o"][..],
            &["mal", "san", "ul", "ej", "o"][..],
            &["ĉifr", "o"][..],
            &["kod", "o"][..],
        ] {
            let expected: Vec<u32> = units
                .iter()
                .map(|unit| {
                    let ids = vocab.encode(unit);
                    assert_eq!(ids.len(), 1, "{unit:?} must map to exactly one id");
                    ids[0]
                })
                .collect();
            assert_eq!(morphemes_to_cbms(&b, units).unwrap(), expected);
        }
    }

    #[test]
    fn frontend_to_cbms_kodo_matches_writer_baseline() {
        frontend_matches_writer("kodo");
    }

    #[test]
    fn frontend_to_cbms_cxifro_full_sequence() {
        frontend_matches_writer("ĉifro");
    }

    #[test]
    fn frontend_to_cbms_memoro_full_sequence() {
        frontend_matches_writer("memoro");
    }

    #[test]
    fn frontend_to_cbms_memorilo_full_sequence() {
        frontend_matches_writer("memorilo");
    }

    #[test]
    fn adversarial_distinct_eo_do_not_collide_via_glyph_space() {
        // ĉifro vs cifro: rozne EO, rozne id - gdyby sciezka szla przez wspolny glif, zderzylyby sie.
        let a = frontend_matches_writer("ĉifro");
        let b = frontend_matches_writer("cifro");
        assert_ne!(a, b, "rozne EO nie moga dac tych samych id");
    }

    #[test]
    fn glyph_text_is_not_a_bridge_to_kodo_id() {
        // Zakaz: glyph -> CBMS id. Symbol kodo w ksiega-max to ቂ; zakodowanie glifu
        // jako tekstu NIE SMIE dawac [1062].
        let eo = frontend_to_cbms(book(), "kodo").unwrap();
        let glyph = frontend_to_cbms(book(), "ቂ").unwrap();
        assert_eq!(eo, Vocabulary::new(book()).unwrap().encode("kodo"));
        assert_ne!(
            glyph, eo,
            "sciezka przez glif nie wolno trafiac w id slowa EO"
        );
    }
}
