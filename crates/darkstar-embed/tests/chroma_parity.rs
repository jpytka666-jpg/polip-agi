// darkstar-header-v1
// po co: chroma_parity.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Egzamin wstepny silnika: czy daje TE SAME liczby co Chroma.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 01:30:00
//! REASON FOR CREATION: 4631 wspomnien policzyla Chroma. Silnik, ktory daje inne liczby,
//! jest bezwartosciowy - choc by byl najszybszy - bo stare wspomnienia i nowe pytania
//! przestaja trafiac w ten sam swiat. To jedyny test, ktory to rozstrzyga.
//! MECHANICS: Czyta plik wzorcowy: pary tekst + wektor policzony przez Chrome. Dla kazdej
//! pary liczy wektor wlasnym silnikiem i porownuje. Prog 0.9999 zgodnosci kierunku - ponizej
//! tego roznica przestaje byc szumem zaokraglen.
//! SYSTEM PART: Darkstar Embed / test zgodnosci.
//! ARCHITECTURE FUNCTION: Bramka jakosci. Kazda zmiana sposobu liczenia - inny tokenizer,
//! inna kolejnosc usredniania, inna wersja biblioteki - musi przez nia przejsc.
//! DEPENDENCIES/LINKS: darkstar-embed. Wymaga trzech zmiennych srodowiskowych; bez nich test
//! sie POMIJA zamiast falszywie przechodzic - patrz nizej.
//! TECH STACK: Rust 2024, zwykly test integracyjny.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uruchomienie:
//! ```text
//! ORT_DYLIB_PATH=<sciezka do biblioteki onnxruntime>
//! DARKSTAR_EMBED_MODEL_DIR=<katalog z model.onnx i tokenizer.json>
//! DARKSTAR_EMBED_REFERENCE=<plik wzorcowy z Chromy>
//! cargo test -p darkstar-embed --test chroma_parity -- --nocapture
//! ```
//!
//! Plik wzorcowy powstaje przez odczyt z Chromy i zawiera prawdziwa tresc wspomnien,
//! wiec NIE trafia do repozytorium. Ksztalt:
//! `{"documents": ["...", ...], "embeddings": [[384 liczby], ...]}`

use darkstar_embed::MiniLmEmbedder;
use darkstar_shadow::{Embedder, cosine_similarity};

/// Ponizej tej zgodnosci roznica przestaje byc szumem zaokraglen i staje sie inna odpowiedzia.
const THRESHOLD: f32 = 0.9999;

/// Ustaw te zmienna tam, gdzie pominiecie testu zgodnosci ma byc bledem, a nie wyborem -
/// czyli wszedzie, gdzie ktos moglby wziac zielone swiatlo za dowod zgodnosci.
const REQUIRE: &str = "DARKSTAR_EMBED_REQUIRE_PARITY";

/// Mowi, czy brakuje warunkow do uruchomienia testu - i pilnuje, zeby pominiecie nie udawalo
/// sukcesu tam, gdzie ma go nie udawac.
///
/// Bez tego test na maszynie bez modelu konczy sie slowem PASSED, co czyta sie jak
/// "zgodnosc potwierdzona", a znaczy "nic nie sprawdzono". To jest najgorszy rodzaj testu:
/// taki, ktory daje pewnosc dokladnie tam, gdzie jej nie ma.
fn missing(vars: &[&str]) -> bool {
    let absent: Vec<&str> = vars
        .iter()
        .copied()
        .filter(|v| std::env::var(v).is_err())
        .collect();
    if absent.is_empty() {
        return false;
    }
    if std::env::var(REQUIRE).is_ok() {
        panic!(
            "{REQUIRE} jest ustawione, a brakuje: {}. Test zgodnosci MUSI sie tu wykonac.",
            absent.join(", ")
        );
    }
    eprintln!(
        "POMINIETO: brak {} (ustaw {REQUIRE}, zeby brak byl bledem)",
        absent.join(", ")
    );
    true
}

#[test]
fn vectors_match_what_chroma_computed() {
    if missing(&["DARKSTAR_EMBED_REFERENCE", "DARKSTAR_EMBED_MODEL_DIR"]) {
        return;
    }
    let reference_path = std::env::var("DARKSTAR_EMBED_REFERENCE").expect("sprawdzone wyzej");

    let raw = std::fs::read_to_string(&reference_path)
        .unwrap_or_else(|e| panic!("nie moge odczytac wzorca {reference_path}: {e}"));
    let reference: serde_json::Value =
        serde_json::from_str(&raw).expect("wzorzec nie jest poprawnym JSON-em");

    let documents = reference["documents"]
        .as_array()
        .expect("wzorzec bez pola documents");
    let embeddings = reference["embeddings"]
        .as_array()
        .expect("wzorzec bez pola embeddings");
    assert!(!documents.is_empty(), "wzorzec jest pusty");
    assert_eq!(
        documents.len(),
        embeddings.len(),
        "wzorzec niespojny: {} tekstow wobec {} wektorow",
        documents.len(),
        embeddings.len()
    );

    let embedder = MiniLmEmbedder::from_env().expect("wczytanie modelu");
    assert_eq!(embedder.dimensions(), 384, "model o innym rozmiarze wektora");

    let mut checked = 0usize;
    let mut worst = f32::MAX;

    for (i, doc) in documents.iter().enumerate() {
        let text = doc.as_str().unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        let expected: Vec<f32> = embeddings[i]
            .as_array()
            .expect("wektor wzorcowy nie jest lista")
            .iter()
            .map(|v| v.as_f64().unwrap_or_default() as f32)
            .collect();

        let got = embedder.embed(text).expect("liczenie osadzenia");
        assert_eq!(
            got.len(),
            expected.len(),
            "wpis {i}: policzono {} wymiarow, wzorzec ma {}",
            got.len(),
            expected.len()
        );

        let similarity = cosine_similarity(&got, &expected).expect("porownanie");
        let max_diff = got
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);

        // Tresci wspomnien nie wypisujemy - tylko numer, dlugosc i zmierzone liczby.
        eprintln!(
            "wpis {i}: {} znakow | zgodnosc {similarity:.9} | najwieksza roznica {max_diff:.9}",
            text.len()
        );

        assert!(
            similarity >= THRESHOLD,
            "wpis {i}: zgodnosc {similarity:.9} ponizej progu {THRESHOLD} - \
             tak wpiety silnik rozjechalby sie z pamiecia policzona przez Chrome"
        );

        worst = worst.min(similarity);
        checked += 1;
    }

    assert!(checked > 0, "wzorzec nie zawieral ani jednego tekstu do sprawdzenia");
    eprintln!("ZDANE: {checked} wpisow, najgorsza zgodnosc {worst:.9}");
}

/// Co silnik robi z tekstem pustym i z samymi bialymi znakami.
///
/// Przeglad zglosil to jako mozliwy rozjazd: gdyby Chroma zwracala dla pustego tekstu wektor,
/// a my blad, wyniki rozjechalyby sie na wpisach bez tresci. Rozstrzygamy pomiarem zamiast
/// rozwazaniem - tokenizer z `add_special_tokens = true` dokłada [CLS] i [SEP], wiec
/// pytanie brzmi, czy ciag tokenow w ogole bywa pusty.
#[test]
fn empty_and_blank_text_behave_predictably() {
    if missing(&["DARKSTAR_EMBED_MODEL_DIR"]) {
        return;
    }

    let embedder = MiniLmEmbedder::from_env().expect("wczytanie modelu");

    for (label, text) in [("pusty", ""), ("spacja", " "), ("biale znaki", "\t\n  ")] {
        match embedder.embed(text) {
            Ok(v) => {
                eprintln!("{label}: wektor {} wymiarow", v.len());
                assert_eq!(v.len(), 384, "{label}: zly rozmiar wektora");
                assert!(
                    v.iter().all(|x| x.is_finite()),
                    "{label}: wektor zawiera wartosci nieliczbowe"
                );
            }
            Err(e) => {
                // Blad tez jest dopuszczalnym zachowaniem - ale musimy WIEDZIEC, ktore
                // z dwoch zachodzi, bo od tego zalezy, czy pusty wpis da sie odnalezc.
                eprintln!("{label}: blad -> {e}");
            }
        }
    }
}

/// Silnik wpiety w gniazdo trybu cienia: czy pasuje i czy nadal daje ten sam wynik.
///
/// To jest sprawdzian polaczenia dwoch czesci, ktore powstaly osobno. Kazda z nich dziala
/// we wlasnych testach; ten mowi, czy dzialaja RAZEM - i czy obecnosc ucznia niczego nie
/// zmienia w odpowiedzi, ktora dostaje system.
#[test]
fn the_engine_fits_the_shadow_socket() {
    if missing(&["DARKSTAR_EMBED_MODEL_DIR"]) {
        return;
    }

    use darkstar_shadow::ShadowedEmbedder;

    // JEDEN model, dwa liczenia. Wczytanie dwoch kopii zajmowaloby 2 x 86 MB naraz - a ten
    // sam plik ostrzega, ze druga kopia nie miesci sie sensownie w pamieci tej maszyny.
    // Najpierw wynik bezposrednio, potem ten sam silnik przeniesiony do gniazda.
    let direct = MiniLmEmbedder::from_env().expect("wczytanie silnika");
    let expected = direct.embed("pamiec systemu").expect("liczenie bezposrednie");

    let socketed = ShadowedEmbedder::new(Box::new(direct));

    assert_eq!(socketed.live_name(), "all-MiniLM-L6-v2");
    assert_eq!(socketed.dimensions(), 384);
    assert_eq!(socketed.shadow_name(), None, "uczen jeszcze nie wpiety");

    let through_socket = socketed.embed("pamiec systemu").expect("liczenie przez gniazdo");

    assert_eq!(
        through_socket, expected,
        "gniazdo nie moze zmieniac wyniku - ma go tylko przepuszczac"
    );
    eprintln!(
        "ZDANE: silnik pasuje do gniazda, {} wymiarow, wynik identyczny",
        through_socket.len()
    );
}
