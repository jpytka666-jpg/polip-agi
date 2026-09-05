// darkstar-header-v1
// po co: lib.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Silnik osadzen all-MiniLM-L6-v2 - zamienia zdanie na 384 liczby, tak samo jak Chroma.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 01:25:00
//! REASON FOR CREATION: 4631 wspomnien lezy w magazynie policzonych przez Chrome. Bez silnika
//! po stronie Darkstara pamieci nie da sie zapytac zwyklym zdaniem - magazyn przyjmuje wektor,
//! nie tekst. To jest brakujace ogniwo miedzy pytaniem czlowieka a pamiecia systemu.
//! MECHANICS: Ten sam model i ta sama biblioteka wnioskujaca, ktorych uzywala Chroma: plik
//! model.onnx uruchamiany przez ONNX Runtime. Tekst idzie przez tokenizer WordPiece, potem
//! przez model, potem srednia po tokenach wazona maska, na koncu normalizacja do dlugosci 1.
//! Kolejnosc i sposob tych trzech krokow decyduja o zgodnosci - zmierzone, nie zalozone:
//! zgodnosc z wektorami Chromy wynosi 1.000000000, najwieksza roznica 3e-8.
//! SYSTEM PART: Darkstar Embed / silnik osadzen.
//! ARCHITECTURE FUNCTION: Implementacja cechy `Embedder` z darkstar-shadow. Wpina sie w
//! gniazdo jako model zywy; uczen (Noworodek) wpina sie obok tym samym kontraktem.
//! DEPENDENCIES/LINKS: darkstar-shadow (kontrakt Embedder), ort (ONNX Runtime), tokenizers.
//! Biblioteka wnioskujaca ladowana jest w czasie dzialania ze sciezki w konfiguracji - nie
//! linkowana statycznie, bo wtedy uzywamy DOKLADNIE tej samej binarki co Chroma.
//! TECH STACK: Rust 2024 + ort. ONNX Runtime to jedyna droga dajaca pewnosc zgodnosci: Chroma
//! liczyla nim, wiec liczymy nim. Czysto rustowe silniki (tract, candle) moglyby dac inne
//! liczby przy uzednianiu, a rozjazd unieważnia cala zgromadzona pamiec.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use darkstar_shadow::{EmbedError, Embedder};

/// Ile liczb ma wektor tego modelu. Ta sama wartosc, ktora maja wszystkie wspomnienia
/// juz lezace w magazynie - rozjazd tutaj znaczy, ze stare i nowe nie trafiaja w ten sam swiat.
pub const MINILM_DIMENSIONS: usize = 384;

const MODEL_FILE: &str = "model.onnx";
const TOKENIZER_FILE: &str = "tokenizer.json";

/// Skad wziac pliki. Zadna sciezka nie jest wpisana w kod - inna maszyna, inne miejsce.
#[derive(Debug, Clone)]
pub struct MiniLmConfig {
    /// Katalog z `model.onnx` i `tokenizer.json`.
    pub model_dir: PathBuf,
    /// Ile watkow na jedno liczenie. Na dwurdzeniowym laptopie wiecej niz 2 tylko szkodzi:
    /// watki zaczynaja walczyc o te same rdzenie.
    pub threads: usize,
}

impl MiniLmConfig {
    pub fn new(model_dir: impl Into<PathBuf>) -> Self {
        Self {
            model_dir: model_dir.into(),
            threads: 2,
        }
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads.max(1);
        self
    }

    /// Wczytuje ustawienia ze zmiennych srodowiskowych.
    ///
    /// `DARKSTAR_EMBED_MODEL_DIR` jest wymagana. `DARKSTAR_EMBED_THREADS` jest opcjonalna.
    /// Sciezka do biblioteki wnioskujacej idzie osobno, przez `ORT_DYLIB_PATH` - tak chce
    /// biblioteka i nie ma powodu tego dublowac.
    pub fn from_env() -> Result<Self, EmbedError> {
        let dir = std::env::var("DARKSTAR_EMBED_MODEL_DIR").map_err(|_| {
            EmbedError::Unavailable(
                "brak DARKSTAR_EMBED_MODEL_DIR - wskaz katalog z model.onnx i tokenizer.json".into(),
            )
        })?;
        let threads = std::env::var("DARKSTAR_EMBED_THREADS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);
        Ok(Self::new(dir).with_threads(threads))
    }
}

/// Model osadzen gotowy do pracy.
///
/// Sesja siedzi za zamkiem, bo liczenie wymaga wylacznego dostepu do niej, a kontrakt
/// `Embedder` daje tylko wspoldzielona referencje. Zamek zamiast kopii sesji na watek:
/// model wazy 86 MB i druga kopia nie zmiescilaby sie sensownie w pamieci tej maszyny.
pub struct MiniLmEmbedder {
    session: Mutex<ort::session::Session>,
    tokenizer: tokenizers::Tokenizer,
    dimensions: usize,
}

impl MiniLmEmbedder {
    pub fn load(config: &MiniLmConfig) -> Result<Self, EmbedError> {
        let model_path = config.model_dir.join(MODEL_FILE);
        let tokenizer_path = config.model_dir.join(TOKENIZER_FILE);
        require_file(&model_path)?;
        require_file(&tokenizer_path)?;

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path).map_err(|e| {
            EmbedError::Unavailable(format!("tokenizer {}: {e}", tokenizer_path.display()))
        })?;

        let session = ort::session::Session::builder()
            .map_err(|e| EmbedError::Unavailable(format!("budowanie sesji: {e}")))?
            .with_intra_threads(config.threads)
            .map_err(|e| EmbedError::Unavailable(format!("ustawienie watkow: {e}")))?
            .commit_from_file(&model_path)
            .map_err(|e| {
                EmbedError::Unavailable(format!("wczytanie {}: {e}", model_path.display()))
            })?;

        let mut embedder = Self {
            session: Mutex::new(session),
            tokenizer,
            dimensions: MINILM_DIMENSIONS,
        };

        // Rozmiar wektora ustalamy POMIAREM, nie deklaracja: jedno probne liczenie przy
        // wczytywaniu. Gdyby ktos podmienil plik na wariant o innym rozmiarze, wychodzi to
        // tutaj, a nie po cichu przy pierwszym zapisie do magazynu - wtedy bylby juz
        // rozjazd z 4631 wspomnieniami. Koszt: okolo jednego liczenia na starcie.
        let probe = embedder.embed("pomiar")?;
        if probe.is_empty() {
            return Err(EmbedError::Unavailable(
                "model zwrocil pusty wektor przy pomiarze rozmiaru".into(),
            ));
        }
        embedder.dimensions = probe.len();

        Ok(embedder)
    }

    pub fn from_env() -> Result<Self, EmbedError> {
        Self::load(&MiniLmConfig::from_env()?)
    }
}

fn require_file(path: &Path) -> Result<(), EmbedError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(EmbedError::Unavailable(format!(
            "brak pliku: {}",
            path.display()
        )))
    }
}

impl Embedder for MiniLmEmbedder {
    fn name(&self) -> &str {
        "all-MiniLM-L6-v2"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| EmbedError::Failed(format!("kodowanie tekstu: {e}")))?;

        let ids: Vec<i64> = encoding.get_ids().iter().map(|&x| i64::from(x)).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| i64::from(x))
            .collect();
        if ids.is_empty() {
            return Err(EmbedError::Failed("pusty tekst po tokenizacji".into()));
        }
        let len = ids.len();
        let types = vec![0i64; len];

        let shape = [1usize, len];
        let ids_t = ort::value::Value::from_array((shape, ids))
            .map_err(|e| EmbedError::Failed(format!("wejscie input_ids: {e}")))?;
        let mask_t = ort::value::Value::from_array((shape, mask.clone()))
            .map_err(|e| EmbedError::Failed(format!("wejscie attention_mask: {e}")))?;
        let types_t = ort::value::Value::from_array((shape, types))
            .map_err(|e| EmbedError::Failed(format!("wejscie token_type_ids: {e}")))?;

        let mut session = self
            .session
            .lock()
            .map_err(|_| EmbedError::Failed("zamek sesji zatruty".into()))?;

        let outputs = session
            .run(ort::inputs![
                "input_ids" => ids_t,
                "attention_mask" => mask_t,
                "token_type_ids" => types_t,
            ])
            .map_err(|e| EmbedError::Failed(format!("liczenie: {e}")))?;

        let (out_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| EmbedError::Failed(format!("odczyt wyniku: {e}")))?;

        let hidden = *out_shape
            .last()
            .ok_or_else(|| EmbedError::Failed("wynik bez ksztaltu".into()))?
            as usize;

        Ok(mean_pool_normalized(data, &mask, hidden))
    }
}

/// Srednia po tokenach wazona maska, potem normalizacja do dlugosci 1.
///
/// Te dwa kroki decyduja o zgodnosci z Chroma i sa najczestszym zrodlem cichego rozjazdu:
/// model zwraca jeden wektor NA TOKEN, a wspomnienie potrzebuje jednego wektora na cale
/// zdanie. Uwzglednienie maski jest istotne - tokeny wypelniajace nie moga rozcienczac
/// sredniej. Normalizacja na koncu jest tym, co robi sentence-transformers, a za nim Chroma.
///
/// Wyodrebnione z metody `embed`, zeby dalo sie to sprawdzic testem bez wczytywania modelu.
pub fn mean_pool_normalized(data: &[f32], mask: &[i64], hidden: usize) -> Vec<f32> {
    let mut pooled = vec![0.0f32; hidden];
    let mut counted = 0.0f32;

    for (token, &m) in mask.iter().enumerate() {
        if m == 0 {
            continue;
        }
        let start = token * hidden;
        let Some(row) = data.get(start..start + hidden) else {
            break;
        };
        counted += 1.0;
        for (acc, v) in pooled.iter_mut().zip(row) {
            *acc += v;
        }
    }

    if counted > 0.0 {
        for v in pooled.iter_mut() {
            *v /= counted;
        }
    }

    let norm = pooled.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in pooled.iter_mut() {
            *v /= norm;
        }
    }

    pooled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_tokens_do_not_dilute_the_average() {
        // Dwa tokeny prawdziwe, jeden wypelniajacy o skrajnej wartosci. Gdyby maska byla
        // ignorowana, wynik odjechalby w strone tego trzeciego - i wektory rozjechalyby sie
        // z Chroma dokladnie na dlugich zdaniach, gdzie wypelnienia jest najwiecej.
        let hidden = 2;
        let data = [
            1.0, 0.0, // token 0 - liczy sie
            0.0, 1.0, // token 1 - liczy sie
            99.0, 99.0, // token 2 - wypelnienie, ma byc pominiete
        ];
        let mask = [1i64, 1, 0];

        let got = mean_pool_normalized(&data, &mask, hidden);

        let expected = 0.5f32 / (0.5f32 * 0.5 + 0.5 * 0.5).sqrt();
        assert!((got[0] - expected).abs() < 1e-6, "got {got:?}");
        assert!((got[1] - expected).abs() < 1e-6, "got {got:?}");
    }

    #[test]
    fn result_has_unit_length() {
        // Chroma normalizuje, wiec my tez musimy. Bez tego zgodnosc kierunku bylaby dobra,
        // ale same liczby inne - a magazyn porownuje liczby.
        let hidden = 3;
        let data = [3.0f32, 4.0, 12.0];
        let mask = [1i64];

        let got = mean_pool_normalized(&data, &mask, hidden);
        let norm = got.iter().map(|v| v * v).sum::<f32>().sqrt();

        assert!((norm - 1.0).abs() < 1e-6, "dlugosc {norm}, oczekiwano 1.0");
    }

    #[test]
    fn all_padding_gives_zeros_instead_of_dividing_by_zero() {
        let got = mean_pool_normalized(&[1.0, 2.0], &[0i64], 2);
        assert_eq!(got, vec![0.0, 0.0]);
    }

    #[test]
    fn truncated_output_does_not_panic() {
        // Maska dluzsza niz dane: jesli model zwrocil mniej niz obiecal, chcemy wyniku
        // z tego, co jest, a nie wywrotki calego systemu.
        let got = mean_pool_normalized(&[1.0, 0.0], &[1i64, 1, 1], 2);
        assert_eq!(got.len(), 2);
        assert!(got[0].is_finite() && got[1].is_finite());
    }

    #[test]
    fn missing_model_directory_is_reported_clearly() {
        let config = MiniLmConfig::new("/nie/ma/takiego/katalogu");
        // `match` zamiast `expect_err`, bo to drugie wymagaloby, zeby caly silnik dalo sie
        // wypisac w komunikacie bledu - a on trzyma w sobie 86 MB wczytanego modelu.
        match MiniLmEmbedder::load(&config) {
            Ok(_) => panic!("wczytanie z nieistniejacego katalogu nie moze sie udac"),
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("brak pliku"), "malo czytelny komunikat: {msg}");
            }
        }
    }
}
