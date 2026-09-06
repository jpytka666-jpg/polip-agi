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
            vocab.plain_id_for_symbol(unit).map_err(|e| {
                CbmsIdsError::Vocab(format!(
                    "morfem {unit:?} nie jest dokladnym symbolem: {e:?}"
                ))
            })
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
    fn baseline_book_path() -> PathBuf {
        if let Ok(p) = std::env::var("NOWORODEK_BOOK") {
            return PathBuf::from(p);
        }
        if let Ok(p) = std::env::var("CBMS_BOOK") {
            return PathBuf::from(p);
        }
        panic!("Ustaw NOWORODEK_BOOK lub CBMS_BOOK na ksiege uzyta do baseline.")
    }

    fn book() -> &'static Book {
        static BOOK: OnceLock<Book> = OnceLock::new();
        BOOK.get_or_init(|| {
            let path = baseline_book_path();
            load_cbms_book(&path).unwrap_or_else(|e| {
                panic!(
                    "brak ksiegi do baseline ({}): {e}. Ustaw NOWORODEK_BOOK lub CBMS_BOOK.",
                    path.display()
                )
            })
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

    fn morph_book() -> Book {
        let source = std::fs::read_to_string(baseline_book_path()).unwrap();
        let grown = format!(
            "{source}\nCBMS-Eo-v1.1-EXT\nil=U+E000\nmal=U+E001\nsan=U+E002\nul=U+E003\nĉifr=U+E004\n"
        );
        load_cbms_book_from_text(&grown)
    }

    fn load_cbms_book_from_text(text: &str) -> Book {
        let (book, collisions) = Book::parse_lenient(text).unwrap();
        assert!(collisions.is_empty(), "POC book collisions: {collisions:?}");
        book
    }

    #[test]
    fn verified_morpheme_vectors_use_plain_ids_in_order() {
        let b = morph_book();
        for (units, expected) in [
            (&["memor", "o"][..], &[97362, 1584][..]),
            (&["memor", "il", "o"][..], &[97362, 110354, 1584][..]),
            (&["mal", "san", "a"][..], &[110356, 110358, 1570][..]),
            (&["san", "ul", "o"][..], &[110358, 110360, 1584][..]),
            (
                &["mal", "san", "ul", "ej", "o"][..],
                &[110356, 110358, 110360, 46084, 1584][..],
            ),
            (&["ĉifr", "o"][..], &[110362, 1584][..]),
            (&["kod", "o"][..], &[1816, 1584][..]),
        ] {
            assert_eq!(morphemes_to_cbms(&b, units).unwrap(), expected);
        }
    }

    #[test]
    fn frontend_to_cbms_kodo_matches_writer_baseline() {
        // Baseline: cbms.exe <ksiega-max> ids  -> 1062
        assert_eq!(frontend_matches_writer("kodo"), vec![1062]);
    }

    #[test]
    fn frontend_to_cbms_cxifro_full_sequence() {
        // Baseline: ĉifro -> 420,94280,1584
        assert_eq!(frontend_matches_writer("ĉifro"), vec![420, 94280, 1584]);
    }

    #[test]
    fn frontend_to_cbms_memoro_full_sequence() {
        assert_eq!(frontend_matches_writer("memoro"), vec![97362, 1186]);
    }

    #[test]
    fn frontend_to_cbms_memorilo_full_sequence() {
        assert_eq!(frontend_matches_writer("memorilo"), vec![660, 7312, 1584]);
    }

    #[test]
    fn adversarial_distinct_eo_do_not_collide_via_glyph_space() {
        // ĉifro vs cifro: rozne EO, rozne id - gdyby sciezka szla przez wspolny glif, zderzylyby sie.
        let a = frontend_matches_writer("ĉifro");
        let b = frontend_matches_writer("cifro");
        assert_ne!(a, b, "rozne EO nie moga dac tych samych id");
        assert_eq!(a, vec![420, 94280, 1584]);
        assert_eq!(b, vec![6214, 94280, 1584]);
    }

    #[test]
    fn glyph_text_is_not_a_bridge_to_kodo_id() {
        // Zakaz: glyph -> CBMS id. Symbol kodo w ksiega-max to ቂ; zakodowanie glifu
        // jako tekstu NIE SMIE dawac [1062].
        let eo = frontend_to_cbms(book(), "kodo").unwrap();
        let glyph = frontend_to_cbms(book(), "ቂ").unwrap();
        assert_eq!(eo, vec![1062]);
        assert_ne!(
            glyph, eo,
            "sciezka przez glif nie wolno trafiac w id slowa EO"
        );
    }
}
