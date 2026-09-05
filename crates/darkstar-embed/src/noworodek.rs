// darkstar-header-v1
// po co: noworodek.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Noworodek jako UCZEN w gniezdzie trybu cienia.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 14:30:00
//! REASON FOR CREATION: Gniazdo ucznia (darkstar-shadow) stoi od 2026-09-04 i czeka. Noworodek
//! spelnia jego warunek dopiero teraz: ma wagi, przekroczyl prog czestosci i umie zwinac
//! znaki w opis calego zdania. To jest ta implementacja.
//! MECHANICS: Tekst idzie do `cbms ids`, ktore zwraca ciag znakow CBMS; kazdy znak ma wiersz
//! w tablicy osadzen wczytanej z pliku wag NWRD; wiersze sa usredniane i normalizowane do
//! dlugosci 1 - tak samo jak konczy all-MiniLM-L6-v2, zeby oba silniki dawaly wektory tej
//! samej natury.
//! SYSTEM PART: Darkstar Embed / silnik ucznia.
//! ARCHITECTURE FUNCTION: Druga implementacja cechy `Embedder`. Wpina sie tym samym
//! kontraktem co model zywy, ale wylacznie jako uczen - gniazdo pilnuje, zeby jego glos
//! nigdy nie trafil do odpowiedzi.
//! DEPENDENCIES/LINKS: darkstar-shadow (kontrakt), plik wag NWRD, ksiega CBMS, narzedzie
//! `cbms` z galezi feature/cbms-writing.
//! TECH STACK: Rust 2024, zero nowych zaleznosci.
//!
//! DLACZEGO TOKENIZACJA IDZIE PRZEZ OSOBNY PROGRAM, A NIE JEST TU PRZEPISANA:
//! numeracja znakow w CBMS nie wynika wprost z kolejnosci wpisow w ksiedze - sprawdzone
//! i obalone pomiarem: znak o numerze 1684 nie odpowiada wpisowi 842, jak sugerowalaby
//! regula "plain na 2i, spaced na 2i+1". Przepisanie tej logiki tutaj byloby czwarta jej
//! kopia w systemie, a kazda kopia to kolejne miejsce, w ktorym moze sie rozjechac.
//! Kosztem jest uruchomienie procesu na kazde zdanie - ZMIERZONE i wypisywane, bo gniazdo
//! czeka na ucznia synchronicznie, wiec wolny uczen spowalnia caly system.
//!
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use darkstar_shadow::{EmbedError, Embedder};

const MAGIC: &[u8; 4] = b"NWRD";
const EMBED_TENSOR: &str = "model.embeddings.token.weight";
const NORM_TENSOR: &str = "model.final_norm.weight";

/// Skad wziac czesci ucznia. Zadna sciezka nie jest wpisana w kod.
#[derive(Debug, Clone)]
pub struct NoworodekConfig {
    /// Plik wag w formacie NWRD.
    pub weights: PathBuf,
    /// Ksiega kodow CBMS - ta sama, ktora kodowala korpus.
    pub book: PathBuf,
    /// Program `cbms` zamieniajacy tekst na znaki.
    pub cbms_tool: PathBuf,
    /// Katalog na pliki przejsciowe tokenizacji.
    pub work_dir: PathBuf,
}

impl NoworodekConfig {
    /// Wczytuje ustawienia ze zmiennych srodowiskowych.
    ///
    /// Wszystkie sa wymagane: uczen bez ktorejkolwiek czesci nie jest wolniejszy, tylko
    /// bezuzyteczny, wiec lepiej odmowic wczytania niz wpiac cos, co zawsze zwraca blad.
    pub fn from_env() -> Result<Self, EmbedError> {
        let get = |key: &str| -> Result<PathBuf, EmbedError> {
            std::env::var(key)
                .map(PathBuf::from)
                .map_err(|_| EmbedError::Unavailable(format!("brak {key}")))
        };
        Ok(Self {
            weights: get("NOWORODEK_WEIGHTS")?,
            book: get("NOWORODEK_BOOK")?,
            cbms_tool: get("NOWORODEK_CBMS")?,
            work_dir: std::env::var("NOWORODEK_WORK_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir()),
        })
    }
}

/// Uczen: zamienia zdanie na wektor, uzywajac wpisanej mu wiedzy o znakach.
pub struct NoworodekEmbedder {
    config: NoworodekConfig,
    /// Tablica osadzen: `vocab` wierszy po `hidden` liczb.
    table: Vec<f32>,
    hidden: usize,
    vocab: usize,
}

impl NoworodekEmbedder {
    pub fn load(config: NoworodekConfig) -> Result<Self, EmbedError> {
        for (what, path) in [
            ("wagi", &config.weights),
            ("ksiega", &config.book),
            ("program cbms", &config.cbms_tool),
        ] {
            if !path.is_file() {
                return Err(EmbedError::Unavailable(format!(
                    "{what}: brak pliku {}",
                    path.display()
                )));
            }
        }

        let tensors = read_nwrd(&config.weights)
            .map_err(|e| EmbedError::Unavailable(format!("wagi: {e}")))?;

        // Wymiar odczytujemy z warstwy, ktora ma go dokladnie tyle, ile wynosi. Zgadywanie
        // z rozmiaru tablicy jest zawodne: 53746 x 128 dzieli sie takze przez 256, a zly
        // wymiar czyni KAZDY wektor bezsensownym, przy czym liczby nadal wygladaja jak liczby.
        let hidden = tensors
            .iter()
            .find(|t| t.name == NORM_TENSOR)
            .map(|t| t.values.len())
            .filter(|len| *len > 0)
            .ok_or_else(|| {
                EmbedError::Unavailable(format!("brak {NORM_TENSOR} - nie ustale wymiaru"))
            })?;

        let table = tensors
            .into_iter()
            .find(|t| t.name == EMBED_TENSOR)
            .map(|t| t.values)
            .ok_or_else(|| EmbedError::Unavailable(format!("brak {EMBED_TENSOR}")))?;

        if table.len() % hidden != 0 {
            return Err(EmbedError::Unavailable(format!(
                "tablica {} liczb nie dzieli sie przez wymiar {hidden}",
                table.len()
            )));
        }
        let vocab = table.len() / hidden;

        Ok(Self {
            config,
            table,
            hidden,
            vocab,
        })
    }

    pub fn from_env() -> Result<Self, EmbedError> {
        Self::load(NoworodekConfig::from_env()?)
    }

    /// Ile znakow slownika ma niezerowy opis - czyli o ilu uczen cokolwiek wie.
    pub fn known_symbols(&self) -> usize {
        self.table
            .chunks(self.hidden)
            .filter(|row| row.iter().any(|v| *v != 0.0))
            .count()
    }

    /// Tekst -> znaki CBMS, przez zewnetrzny program.
    ///
    /// Pliki przejsciowe maja w nazwie numer procesu i licznik, bo gniazdo moze wolac ucznia
    /// z wielu watkow naraz, a dwa watki piszace do tego samego pliku daja wynik, ktory nie
    /// odpowiada zadnemu z pytan.
    fn to_symbols(&self, text: &str) -> Result<Vec<usize>, EmbedError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let tag = format!(
            "{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );

        let in_path = self.config.work_dir.join(format!("nwd-{tag}.txt"));
        let out_path = self.config.work_dir.join(format!("nwd-{tag}.u16"));

        let write = |path: &Path| -> std::io::Result<()> {
            let mut f = File::create(path)?;
            f.write_all(text.as_bytes())?;
            f.write_all(b"\n")
        };
        write(&in_path).map_err(|e| EmbedError::Failed(format!("zapis pytania: {e}")))?;

        let status = Command::new(&self.config.cbms_tool)
            .arg(&self.config.book)
            .arg("ids")
            .arg(&in_path)
            .arg(&out_path)
            .output();

        // Sprzatamy zawsze - takze gdy program zawiodl. Inaczej katalog roboczy zapelnia sie
        // odpadami po kazdym nieudanym pytaniu.
        let _ = std::fs::remove_file(&in_path);

        let result = match status {
            Ok(out) if out.status.success() => std::fs::read(&out_path)
                .map(|bytes| {
                    bytes
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]) as usize)
                        .collect::<Vec<_>>()
                })
                .map_err(|e| EmbedError::Failed(format!("odczyt znakow: {e}"))),
            Ok(out) => Err(EmbedError::Failed(format!(
                "cbms odmowil: {}",
                String::from_utf8_lossy(&out.stderr).chars().take(200).collect::<String>()
            ))),
            Err(e) => Err(EmbedError::Unavailable(format!("nie moge uruchomic cbms: {e}"))),
        };

        let _ = std::fs::remove_file(&out_path);
        result
    }
}

impl Embedder for NoworodekEmbedder {
    fn name(&self) -> &str {
        "noworodek-cbms"
    }

    fn dimensions(&self) -> usize {
        self.hidden
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let symbols = self.to_symbols(text)?;
        let (vector, used) = fold_symbols(&self.table, &symbols, self.hidden, self.vocab);
        if used == 0 {
            return Err(EmbedError::Failed(
                "zaden znak zdania nie ma opisu - uczen nie zna tych slow".into(),
            ));
        }
        Ok(vector)
    }
}

/// Srednia opisow znakow, znormalizowana do dlugosci 1. Zwraca takze, ile znakow wzieto.
///
/// Znaki nieznane i te o samych zerach sa pomijane, nie usredniane: zero jest wektorem bez
/// kierunku i wciagniete do sredniej przesunelaby wynik ku poczatkowi ukladu - jedno nieznane
/// slowo zmienialoby opis calego zdania.
pub fn fold_symbols(
    table: &[f32],
    symbols: &[usize],
    hidden: usize,
    vocab: usize,
) -> (Vec<f32>, usize) {
    let mut sum = vec![0.0f32; hidden];
    let mut used = 0usize;

    for &id in symbols {
        if id >= vocab {
            continue;
        }
        let Some(row) = table.get(id * hidden..(id + 1) * hidden) else {
            continue;
        };
        if row.iter().all(|v| *v == 0.0) {
            continue;
        }
        for (acc, v) in sum.iter_mut().zip(row) {
            *acc += v;
        }
        used += 1;
    }

    if used > 0 {
        let n = used as f32;
        for v in sum.iter_mut() {
            *v /= n;
        }
        let norm = sum.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in sum.iter_mut() {
                *v /= norm;
            }
        }
    }
    (sum, used)
}

struct RawTensor {
    name: String,
    values: Vec<f32>,
}

fn read_nwrd(path: &Path) -> std::io::Result<Vec<RawTensor>> {
    let mut r = BufReader::new(File::open(path)?);
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "zly znacznik pliku, oczekiwano NWRD",
        ));
    }
    let mut u32_buf = [0u8; 4];
    let mut u64_buf = [0u8; 8];
    r.read_exact(&mut u32_buf)?; // wersja
    r.read_exact(&mut u32_buf)?;
    let count = u32::from_le_bytes(u32_buf);

    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        r.read_exact(&mut u32_buf)?;
        let name_len = u32::from_le_bytes(u32_buf) as usize;
        let mut name_bytes = vec![0u8; name_len];
        r.read_exact(&mut name_bytes)?;
        r.read_exact(&mut u64_buf)?;
        let value_count = u64::from_le_bytes(u64_buf) as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            r.read_exact(&mut u32_buf)?;
            values.push(f32::from_le_bytes(u32_buf));
        }
        out.push(RawTensor {
            name: String::from_utf8_lossy(&name_bytes).into_owned(),
            values,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_symbols_do_not_change_the_description() {
        let table = vec![1.0, 0.0, 0.0, 1.0];
        let (with_unknown, used_a) = fold_symbols(&table, &[0, 9999], 2, 2);
        let (without, used_b) = fold_symbols(&table, &[0], 2, 2);
        assert_eq!((used_a, used_b), (1, 1));
        assert_eq!(with_unknown, without);
    }

    #[test]
    fn symbols_with_no_description_are_skipped() {
        let table = vec![1.0, 0.0, 0.0, 0.0];
        let (_, used) = fold_symbols(&table, &[1], 2, 2);
        assert_eq!(used, 0, "wiersz z samych zer nie jest opisem");
    }

    #[test]
    fn description_has_unit_length() {
        let table = vec![3.0, 4.0, 6.0, 8.0];
        let (v, _) = fold_symbols(&table, &[0, 1], 2, 2);
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "dlugosc {norm}");
    }

    #[test]
    fn missing_parts_are_reported_clearly() {
        let config = NoworodekConfig {
            weights: PathBuf::from("/nie/ma/wag.nwrd"),
            book: PathBuf::from("/nie/ma/ksiegi.txt"),
            cbms_tool: PathBuf::from("/nie/ma/cbms"),
            work_dir: std::env::temp_dir(),
        };
        match NoworodekEmbedder::load(config) {
            Ok(_) => panic!("uczen bez czesci nie moze sie wczytac"),
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("wagi"), "komunikat ma mowic, czego brakuje: {msg}");
            }
        }
    }
}
