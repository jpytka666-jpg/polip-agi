// darkstar-header-v1
// po co: main.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Wpisuje wiedze wprost w wagi Noworodka, zamiast czekac, az sam sie jej nauczy.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 12:45:00
//! REASON FOR CREATION: Trzy przebiegi treningu skonczyly sie tym samym zdaniem: model
//! jest GORSZY niz zwykla tablica czestosci. Najlepszy z nich - duzy model po 53 minutach -
//! wciaz przegrywal o 2.262 bitu. Skoro tablica czestosci bije wytrenowany model, nie ma
//! powodu uczyc go tego przez godziny: mozna mu to wpisac i pozwolic uczyc sie tego,
//! czego tablica NIE umie - kontekstu.
//! MECHANICS: Czyta plik wag w formacie NWRD, przelicza wiersze glowicy wyjsciowej
//! (`model.lm_head.weight`) proporcjonalnie do log-czestosci symbolu w korpusie, zapisuje
//! NOWY plik. Zrodlo nigdy nie jest nadpisywane. Po zmianie liczy i wypisuje dokladnie to,
//! co `editor.rs` Noworodka nazywa TensorDiff: ile liczb sie ruszylo, suma zmian, najwieksza
//! pojedyncza zmiana - zeby dalo sie powiedziec, co sie stalo, a nie tylko ze cos sie stalo.
//! SYSTEM PART: Darkstar / zasiew wag.
//! ARCHITECTURE FUNCTION: Narzedzie do wpisywania wiedzy, ktora juz mamy, do modelu, ktory
//! jeszcze jej nie ma. Format NWRD obslugiwany wlasnym kodem, bo Noworodek zyje w innym
//! repozytorium, na innej galezi, i trwa tam praca - nie wolno go dotykac.
//! DEPENDENCIES/LINKS: format NWRD z noworodek/src/bin/train-cbms.rs (magic, wersja u32,
//! liczba tensorow u32, dalej per tensor: dlugosc nazwy u32, nazwa, liczba wartosci u64,
//! wartosci f32 little-endian).
//! TECH STACK: Rust 2024, zero zaleznosci. Format jest prosty i czytanie go wprost jest
//! tansze niz wciaganie calej biblioteki Noworodka z innego repozytorium.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! darkstar-seed <wagi-wejsciowe.nwrd> <log-czestosci.txt> <wagi-wyjsciowe.nwrd> [sila]
//! ```
//! `sila` domyslnie 0.5. Zero znaczy brak zmiany - przydatne do sprawdzenia, ze samo
//! przepisanie pliku niczego nie psuje.

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

const MAGIC: &[u8; 4] = b"NWRD";
const TARGET: &str = "model.lm_head.weight";
/// Nazwa rubryki wyjsciowej. Musi zgadzac sie co do znaku z tym, czego szuka model -
/// literowka nie da bledu, tylko ciche zignorowanie calej wpisanej wiedzy.
const BIAS_NAME: &str = "model.lm_head.bias";
/// Tablica wektorow wejsciowych - po jednym na symbol.
const EMBED_NAME: &str = "model.embeddings.token.weight";
/// Tablica pozycji - po jednym wektorze na miejsce w zdaniu.
const POSITION_NAME: &str = "model.embeddings.position.weight";
/// Warstwa o dlugosci rownej wymiarowi ukrytemu - stad go odczytujemy, zamiast zgadywac.
const NORM_NAME: &str = "model.final_norm.weight";
/// Ile miejsc obejmuje tablica pozycji. Musi zgadzac sie z MAX_POSITIONS w modelu.
const MAX_POSITIONS: usize = 64;

/// Najwiekszy dopuszczalny mnoznik wiersza (i najmniejszy, jako jego odwrotnosc).
///
/// ZMIERZONE, dlaczego to musi tu byc: bez ograniczenia przy sile 0.5 mnozniki siegnely
/// 239.9x, a trening na tak zmienionych wagach zwrocil `odlozone: inf` - liczby wybuchly
/// i model przestal cokolwiek liczyc.
///
/// Przyczyna lezy w ksztalcie danych, nie w sile: z 53746 symbolow slownika w korpusie
/// wystepuje tylko 5312. Pozostale 48434 maja te sama, bardzo niska log-czestosc, wiec
/// zaniżaja srednia i scieskaja odchylenie. Symbole obecne wychodza wtedy daleko poza
/// skale, a `exp` zamienia to w mnozniki rzedu setek.
const MAX_SCALE: f32 = 2.0;

struct Tensor {
    name: String,
    values: Vec<f32>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!(
            "uzycie: darkstar-seed <wagi.nwrd> <log-czestosci.txt> <wynik.nwrd> [sila=0.5]"
        );
        std::process::exit(2);
    }
    let strength: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.5);

    let mut tensors = match read_nwrd(&args[0]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac wag: {e}");
            std::process::exit(1);
        }
    };
    println!("wczytano {} tensorow z {}", tensors.len(), args[0]);

    let log_freq = match read_log_frequencies(&args[1]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac czestosci: {e}");
            std::process::exit(1);
        }
    };
    println!("wczytano {} log-czestosci", log_freq.len());

    // Tryb wlasciwy: tabelka wchodzi jako osobna rubryka, dodawana do kazdego wyniku.
    // Skalowanie wierszy (ponizej) bylo namiastka z czasow, gdy model tej rubryki nie mial.
    if args.iter().any(|a| a == "--bias") {
        seed_as_bias(&mut tensors, &log_freq);

        // Pary wchodza razem z rubryka, bo to dwie warstwy tej samej wiedzy: rubryka mowi,
        // co jest czeste, a pary - co po czym idzie. Osobno kazda z nich jest polowa odpowiedzi.
        if let Some(pos) = args.iter().position(|a| a == "--pairs") {
            match args.get(pos + 1) {
                Some(corpus_path) => match read_corpus(corpus_path) {
                    Ok(corpus) => {
                        println!("\nkorpus              : {} symboli", corpus.len());
                        let window = args.iter().position(|a| a == "--window").and_then(|p| args.get(p + 1)).and_then(|v| v.parse().ok()).filter(|w: &usize| *w > 0).unwrap_or(1);
                        if let Err(e) = seed_pairs(&mut tensors, &corpus, log_freq.len(), strength, window) {
                            eprintln!("FAIL: pary: {e}");
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("FAIL: nie moge odczytac korpusu {corpus_path}: {e}");
                        std::process::exit(1);
                    }
                },
                None => {
                    eprintln!("FAIL: --pairs wymaga sciezki do korpusu .u16");
                    std::process::exit(2);
                }
            }
        }

        // Zmysl kolejnosci. Tablica pozycji nie istnieje w wagach sprzed tej zmiany, wiec
        // trzeba ja zalozyc; wymiar bierzemy z warstwy, ktora go okresla jednoznacznie.
        if args.iter().any(|a| a == "--positions") {
            match tensors
                .iter()
                .find(|t| t.name == NORM_NAME)
                .map(|t| t.values.len())
                .filter(|len| *len > 0)
            {
                Some(hidden) => {
                    if !tensors.iter().any(|t| t.name == POSITION_NAME) {
                        tensors.push(Tensor {
                            name: POSITION_NAME.to_string(),
                            values: vec![0.0; MAX_POSITIONS * hidden],
                        });
                        println!("\n(tablica pozycji zalozona - wagi jej nie mialy)");
                    }
                    if !seed_positions(&mut tensors, hidden, strength) {
                        eprintln!("FAIL: nie moge wpisac tablicy pozycji");
                        std::process::exit(1);
                    }
                }
                None => {
                    eprintln!("FAIL: brak {NORM_NAME} - nie ustale wymiaru bez zgadywania");
                    std::process::exit(1);
                }
            }
        }

        if let Err(e) = write_nwrd(&args[2], &tensors) {
            eprintln!("FAIL: nie moge zapisac: {e}");
            std::process::exit(1);
        }
        println!("\nzapisano: {}", args[2]);
        println!("zrodlo NIETKNIETE: {}", args[0]);
        return;
    }

    let Some(head) = tensors.iter_mut().find(|t| t.name == TARGET) else {
        eprintln!("FAIL: brak tensora {TARGET} w pliku wag");
        std::process::exit(1);
    };

    // Wymiar bierzemy z danych, nie z zalozenia: liczba wartosci podzielona przez slownik.
    if head.values.len() % log_freq.len() != 0 {
        eprintln!(
            "FAIL: {} liczb nie dzieli sie przez {} symboli - inny slownik niz korpus",
            head.values.len(),
            log_freq.len()
        );
        std::process::exit(1);
    }
    let hidden = head.values.len() / log_freq.len();
    println!("glowica wyjsciowa: {} symboli x {hidden} liczb", log_freq.len());

    let before = head.values.clone();
    let scales = row_scales(&log_freq, strength);
    apply_row_scales(&mut head.values, &scales, hidden);

    report(&before, &head.values, &scales, strength);

    if let Err(e) = write_nwrd(&args[2], &tensors) {
        eprintln!("FAIL: nie moge zapisac: {e}");
        std::process::exit(1);
    }
    println!("\nzapisano: {}", args[2]);
    println!("zrodlo NIETKNIETE: {}", args[0]);
}

/// Mnoznik dla kazdego symbolu: czestszy symbol dostaje glosniejszy wiersz w glowicy.
///
/// Log-czestosci sa najpierw sprowadzane do wspolnej skali (odjeta srednia, podzielone
/// przez odchylenie), zeby `sila` znaczyla to samo niezaleznie od korpusu. Bez tego ta
/// sama wartosc dawalaby zupelnie inny efekt na innym materiale.
fn row_scales(log_freq: &[f32], strength: f32) -> Vec<f32> {
    let n = log_freq.len() as f32;
    let mean = log_freq.iter().sum::<f32>() / n;
    let variance = log_freq.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    let sd = variance.sqrt().max(1e-6);
    log_freq
        .iter()
        .map(|v| ((strength * (v - mean) / sd).exp()).clamp(1.0 / MAX_SCALE, MAX_SCALE))
        .collect()
}

/// Wpisuje tabelke czestosci jako osobna rubryke wyjsciowa (`model.lm_head.bias`).
///
/// To jest droga wlasciwa, w odroznieniu od skalowania wierszy. Model liczy wynik jako
/// `logit[i] = <ukryte, glowica[i]> + rubryka[i]`, a `softmax` zamienia to na rozklad. Jesli
/// w rubryce siedzi `log p(i)`, model bez zadnej nauki startuje od rozkladu czestosci -
/// dokladnie tego, do czego piec przebiegow treningu nie doszlo.
///
/// Odejmujemy srednia, bo `softmax` i tak ignoruje stala dodana do wszystkich pozycji -
/// bez tego rubryka niosla by wielka stala, ktora niczego nie zmienia, za to psuje skale
/// gradientow reszty sieci.
fn seed_as_bias(tensors: &mut Vec<Tensor>, log_freq: &[f32]) {
    let mean = log_freq.iter().sum::<f32>() / log_freq.len() as f32;
    let values: Vec<f32> = log_freq.iter().map(|v| v - mean).collect();

    let min = values.iter().copied().fold(f32::MAX, f32::min);
    let max = values.iter().copied().fold(f32::MIN, f32::max);
    println!("\nrubryka wyjsciowa   : {} liczb", values.len());
    println!("zakres od/do        : {min:.4} .. {max:.4}");
    println!("srednia po odjeciu  : {:.6}", values.iter().sum::<f32>() / values.len() as f32);

    // Podmiana, nie dopisanie: powtorne uruchomienie na tym samym pliku ma dawac ten sam
    // wynik, a nie dwie rubryki, z ktorych model zobaczylby tylko jedna.
    if let Some(existing) = tensors.iter_mut().find(|t| t.name == BIAS_NAME) {
        println!("(rubryka juz byla - podmieniam)");
        existing.values = values;
    } else {
        tensors.push(Tensor {
            name: BIAS_NAME.to_string(),
            values,
        });
        println!("(rubryka dodana jako nowy tensor)");
    }
}

/// Wpisuje w wagi wiedze o tym, CO PO CZYM NASTEPUJE.
///
/// Czestosc to jedna liczba na symbol i miesci sie w rubryce wyjsciowej. Para to zwiazek
/// miedzy dwoma symbolami i osobnego miejsca na nia nie ma - ale jest lepsze. Model
/// przewiduje B po A wtedy, gdy wektor WEJSCIOWY symbolu A pasuje do wektora WYJSCIOWEGO
/// symbolu B. Wystarczy wiec dobrac wektory tak, zeby pasowaly dokladnie tam, gdzie pary
/// wystepuja w korpusie.
///
/// Robimy to rzutem losowym: kazdy symbol dostaje staly, losowy kierunek, a wektor symbolu
/// A powstaje jako suma kierunkow wszystkich jego nastepnikow. Dwa symbole o podobnych
/// nastepnikach dostaja wtedy podobne wektory - a o to wlasnie chodzi. Lemat
/// Johnsona-Lindenstraussa mowi, ze taki rzut zachowuje odleglosci z dokladnoscia zalezna
/// od liczby wymiarow, wiec 128 wymiarow niesie sensowne przyblizenie zwiazkow miedzy
/// tysiacami symboli.
///
/// Kierunki sa liczone z numeru symbolu, nie losowane i pamietane: ten sam symbol zawsze
/// dostaje ten sam kierunek, wiec caly zasiew jest powtarzalny co do bitu.
fn seed_pairs(tensors: &mut [Tensor], corpus: &[usize], vocab: usize, strength: f32, window: usize) -> Result<(), String> {
    let hidden = tensors
        .iter()
        .find(|t| t.name == EMBED_NAME)
        .map(|t| t.values.len() / vocab)
        .ok_or_else(|| format!("brak tensora {EMBED_NAME}"))?;

    // Wektor wejsciowy symbolu = suma kierunkow jego NASTEPNIKOW.
    // Wektor wyjsciowy symbolu = suma kierunkow jego POPRZEDNIKOW.
    // Dzieki temu iloczyn <wejscie[A], wyjscie[B]> rosnie dokladnie dla par, ktore
    // naprawde wystepuja - a to jest liczba, ktora model porownuje przy przewidywaniu.
    let mut input = vec![0.0f32; vocab * hidden];
    let mut output = vec![0.0f32; vocab * hidden];
    let mut pairs = 0usize;

    // Okno: ile symboli w przod bierzemy pod uwage, nie tylko bezposredniego nastepnika.
    // Wklad maleje jak 1/odleglosc - sasiad tuz obok mowi o symbolu wiecej niz ten piec
    // miejsc dalej, ale ten dalszy nadal cos mowi. Wybor 1/d zamiast czegos ostrzejszego
    // jest celowy: przy zbyt szybkim spadku okno przestaje sie roznic od samych par.
    for start in 0..corpus.len() {
        let a = corpus[start];
        if a >= vocab {
            continue;
        }
        for distance in 1..=window {
            let Some(&b) = corpus.get(start + distance) else {
                break;
            };
            if b >= vocab {
                continue;
            }
            let weight = 1.0 / distance as f32;
            for dim in 0..hidden {
                input[a * hidden + dim] += weight * direction(b, dim);
                output[b * hidden + dim] += weight * direction(a, dim);
            }
            pairs += 1;
        }
    }
    println!("okno                : {window} symboli w przod");
    println!("polaczen zapisanych : {pairs}");

    let touched_in = normalize_rows(&mut input, hidden, strength);
    let touched_out = normalize_rows(&mut output, hidden, strength);
    println!("symboli z historia  : wejscie {touched_in}, wyjscie {touched_out} z {vocab}");

    // Symbole nieobecne w korpusie maja same zera - dla nich zostawiamy dotychczasowe wagi.
    // Podmiana ich na zera skasowalaby jedyne, co model o nich wie.
    write_rows(tensors, EMBED_NAME, &input, hidden);
    write_rows(tensors, TARGET, &output, hidden);
    Ok(())
}

/// Wpisuje tablice pozycji - zmysl kolejnosci, ktorego model nie mial.
///
/// ZNALEZIONE PRZY PRZEGLADANIU KODU: uwaga w tym modelu porownuje wylacznie wektory
/// symboli (`dot(q[i], k[j])`) i nigdzie nie wchodzi informacja o tym, GDZIE stoi i oraz j.
/// Maska przyczynowa nie pozwala patrzec w przod, ale nie odroznia symbolu tuz obok od
/// oddalonego o piec miejsc. Model czytal wiec WOREK symboli, nie zdanie - a powtarzany
/// werdykt trenera, ze "nie uzywa kontekstu", byl doslownie prawdziwy: nie mial czym.
///
/// Widac to bylo w pomiarach, tylko nie umialem tego wtedy nazwac: poszerzenie okna
/// zasiewu z 1 do 5 pomoglo (+0.652 -> +0.869), ale z 5 do 12 juz ZASZKODZILO (+0.813).
/// Dokladnie tak zachowuje sie model, ktory nie umie reprezentowac odleglosci - dalsi
/// sasiedzi tylko rozmywaja kierunek symbolu.
///
/// Wpisujemy klasyczne kodowanie sinusoidalne z pierwszej pracy o transformerach: dla
/// pozycji `p` i wymiaru `i` wartosc to `sin(p / 10000^(2i/d))` na parzystych i `cos(...)`
/// na nieparzystych. Nie wymaga nauki, bo NIE JEST wiedza o danych - jest wiedza o tym, jak
/// zapisac odleglosc tak, zeby dala sie odczytac przez iloczyn skalarny. Kazda para
/// sasiadujacych wymiarow obraca sie z inna predkoscia, wiec caly wektor koduje pozycje
/// jednoznacznie, a roznice pozycji daja sie z niego wyliczyc liniowo.
/// `symbol_len` to dlugosc wektora symbolu - pozycja jest do niej skalowana.
///
/// SKALA JEST TU NAJWAZNIEJSZA I ZMIERZONA NA WLASNYM BLEDZIE. Pierwsza wersja wpisywala
/// czyste sinusoidy, czyli wektory o dlugosci `sqrt(hidden/2)` = 8 przy 128 wymiarach.
/// Wektory symboli mialy dlugosc 0.5, wiec pozycja byla **16 razy silniejsza od tresci**:
/// model widzial "to jest trzecie slowo" zamiast "to jest slowo pamiec". Wynik spadl
/// z 4.7913 do 5.3906, a przewaga nad tablica czestosci z 0.869 do 0.004 - czyli dodanie
/// zmyslu kolejnosci skasowalo prawie caly zysk z dwoch poprzednich zasiewow.
///
/// Pozycja ma byc PRZYPRAWA, nie danim glownym: dodatkiem, ktory rozroznia zdania o tych
/// samych slowach, a nie sygnalem zagluszajacym slowa.
const POSITION_STRENGTH: f32 = 0.25;

fn seed_positions(tensors: &mut [Tensor], hidden: usize, symbol_len: f32) -> bool {
    let Some(table) = tensors.iter_mut().find(|t| t.name == POSITION_NAME) else {
        return false;
    };
    if hidden == 0 || table.values.len() % hidden != 0 {
        return false;
    }
    let positions = table.values.len() / hidden;

    // Czysta sinusoida daje wektor o dlugosci sqrt(hidden/2) - kazdy wymiar ma wartosc
    // rzedu jednosci. Sprowadzamy go do ulamka dlugosci wektora symbolu.
    let raw_len = (hidden as f32 / 2.0).sqrt();
    let scale = (symbol_len * POSITION_STRENGTH) / raw_len.max(f32::EPSILON);

    for pos in 0..positions {
        for dim in 0..hidden {
            // Pary wymiarow (2i, 2i+1) dziela ten sam okres; predkosc maleje wykladniczo
            // z numerem pary, wiec pierwsze wymiary rozrozniaja sasiadow, a dalsze - odlegle
            // czesci zdania.
            let pair = (dim / 2) * 2;
            let angle = pos as f32 / 10000f32.powf(pair as f32 / hidden as f32);
            let value = if dim % 2 == 0 { angle.sin() } else { angle.cos() };
            table.values[pos * hidden + dim] = value * scale;
        }
    }

    let got_len = table.values[hidden..2 * hidden]
        .iter()
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt();
    println!("\ntablica pozycji     : {positions} miejsc x {hidden} liczb");
    println!("kodowanie           : sinusoidalne, wpisane wprost - bez nauki");
    println!(
        "sila wobec symbolu  : {got_len:.4} wobec {symbol_len:.4} = {:.0}%",
        100.0 * got_len / symbol_len.max(f32::EPSILON)
    );
    true
}

/// Staly, powtarzalny kierunek dla symbolu w danym wymiarze: +1 albo -1.
///
/// Liczony z numeru symbolu przez mieszanie bitow (splitmix64), a nie losowany i zapisywany:
/// tablica 53746 x 128 kierunkow zajelaby tyle, co same wagi, a i tak jest w pelni okreslona
/// przez numer.
fn direction(symbol: usize, dim: usize) -> f32 {
    let mut h = (symbol as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (dim as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    if h & 1 == 0 { 1.0 } else { -1.0 }
}

/// Sprowadza kazdy niepusty wiersz do zadanej dlugosci. Zwraca liczbe takich wierszy.
///
/// Bez tego symbol wystepujacy tysiac razy mialby wektor tysiac razy dluzszy niz symbol
/// wystepujacy raz - i zdominowalby kazde porownanie niezaleznie od tresci. Liczy sie
/// KIERUNEK, w ktorym symbol wskazuje, nie to, jak czesto wystapil; czestosc siedzi juz
/// w rubryce wyjsciowej.
fn normalize_rows(values: &mut [f32], hidden: usize, target_len: f32) -> usize {
    let mut touched = 0;
    for row in values.chunks_mut(hidden) {
        let norm = row.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm <= f32::EPSILON {
            continue;
        }
        let factor = target_len / norm;
        for v in row.iter_mut() {
            *v *= factor;
        }
        touched += 1;
    }
    touched
}

/// Wpisuje policzone wiersze do tensora, pomijajac wiersze puste.
fn write_rows(tensors: &mut [Tensor], name: &str, rows: &[f32], hidden: usize) {
    let Some(tensor) = tensors.iter_mut().find(|t| t.name == name) else {
        return;
    };
    for (row_index, row) in rows.chunks(hidden).enumerate() {
        if row.iter().all(|v| *v == 0.0) {
            continue;
        }
        let start = row_index * hidden;
        let Some(slot) = tensor.values.get_mut(start..start + hidden) else {
            break;
        };
        slot.copy_from_slice(row);
    }
}

fn read_corpus(path: &str) -> std::io::Result<Vec<usize>> {
    let bytes = std::fs::read(path)?;
    Ok(bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]) as usize)
        .collect())
}

fn apply_row_scales(values: &mut [f32], scales: &[f32], hidden: usize) {
    for (row, scale) in scales.iter().enumerate() {
        let start = row * hidden;
        for v in &mut values[start..start + hidden] {
            *v *= scale;
        }
    }
}

/// To samo, co liczy `editor.rs` Noworodka w `TensorDiff` - zeby dalo sie porownac.
fn report(before: &[f32], after: &[f32], scales: &[f32], strength: f32) {
    let changed = before.iter().zip(after).filter(|(a, b)| a != b).count();
    let l1: f64 = before
        .iter()
        .zip(after)
        .map(|(a, b)| f64::from(b - a).abs())
        .sum();
    let l2: f64 = before
        .iter()
        .zip(after)
        .map(|(a, b)| f64::from(b - a).powi(2))
        .sum::<f64>()
        .sqrt();
    let max_abs = before
        .iter()
        .zip(after)
        .map(|(a, b)| (b - a).abs())
        .fold(0.0f32, f32::max);

    let min_scale = scales.iter().copied().fold(f32::MAX, f32::min);
    let max_scale = scales.iter().copied().fold(0.0f32, f32::max);

    println!("\nsila                : {strength}");
    println!("mnoznik od/do       : {min_scale:.4} .. {max_scale:.4}");
    println!("zmienionych liczb   : {changed} z {}", before.len());
    println!("suma zmian (l1)     : {l1:.3}");
    println!("dlugosc zmiany (l2) : {l2:.3}");
    println!("najwieksza zmiana   : {max_abs:.6}");
}

fn read_nwrd(path: &str) -> std::io::Result<Vec<Tensor>> {
    let mut r = BufReader::new(File::open(path)?);

    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("zly znacznik pliku: {magic:?}, oczekiwano NWRD"),
        ));
    }
    let _version = read_u32(&mut r)?;
    let count = read_u32(&mut r)?;

    let mut tensors = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name_len = read_u32(&mut r)? as usize;
        let mut name_bytes = vec![0u8; name_len];
        r.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).into_owned();

        let value_count = read_u64(&mut r)? as usize;
        let mut values = Vec::with_capacity(value_count);
        let mut buf = [0u8; 4];
        for _ in 0..value_count {
            r.read_exact(&mut buf)?;
            values.push(f32::from_le_bytes(buf));
        }
        tensors.push(Tensor { name, values });
    }
    Ok(tensors)
}

fn write_nwrd(path: &str, tensors: &[Tensor]) -> std::io::Result<()> {
    let mut w = BufWriter::new(File::create(path)?);
    w.write_all(MAGIC)?;
    w.write_all(&1u32.to_le_bytes())?;
    w.write_all(&(tensors.len() as u32).to_le_bytes())?;
    for t in tensors {
        let name = t.name.as_bytes();
        w.write_all(&(name.len() as u32).to_le_bytes())?;
        w.write_all(name)?;
        w.write_all(&(t.values.len() as u64).to_le_bytes())?;
        for v in &t.values {
            w.write_all(&v.to_le_bytes())?;
        }
    }
    w.flush()
}

fn read_log_frequencies(path: &str) -> std::io::Result<Vec<f32>> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line.parse::<f32>() {
            Ok(v) if v.is_finite() => out.push(v),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("wiersz {} nie jest liczba: {line}", i + 1),
                ));
            }
        }
    }
    Ok(out)
}

fn read_u32(r: &mut impl Read) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64(r: &mut impl Read) -> std::io::Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequent_symbols_get_a_louder_row() {
        // Sedno narzedzia: symbol czestszy ma dostac wiekszy mnoznik. Gdyby kolejnosc
        // sie odwrocila, model uczylby sie odwrotnosci tego, co chcemy mu wpisac.
        let log_freq = [-1.0f32, -5.0, -3.0];
        let scales = row_scales(&log_freq, 0.5);
        assert!(scales[0] > scales[2], "czestszy musi byc glosniejszy");
        assert!(scales[2] > scales[1], "kolejnosc musi byc zachowana");
    }

    #[test]
    fn zero_strength_changes_nothing() {
        // Sprawdzian, ze samo przepisanie pliku niczego nie psuje - bez tego nie dalo by
        // sie odroznic bledu zapisu od skutku zmiany.
        let scales = row_scales(&[-1.0, -5.0, -3.0], 0.0);
        for s in scales {
            assert!((s - 1.0).abs() < 1e-6, "mnoznik przy sile 0 musi byc 1.0, jest {s}");
        }
    }

    #[test]
    fn no_multiplier_can_blow_the_weights_up() {
        // Ten test istnieje z powodu prawdziwej awarii: bez ograniczenia mnozniki doszly
        // do 239.9x i trening zwrocil `odlozone: inf`. Ksztalt danych jest tu skrajny -
        // wiekszosc slownika nie wystepuje w korpusie - wiec zabezpieczenie musi trzymac
        // niezaleznie od zadanej sily.
        let mut log_freq = vec![-14.1f32; 48434]; // nieobecne w korpusie
        log_freq.extend(std::iter::repeat_n(-3.0f32, 5312)); // obecne

        for strength in [0.1, 0.5, 2.0, 10.0] {
            let scales = row_scales(&log_freq, strength);
            let max = scales.iter().copied().fold(0.0f32, f32::max);
            let min = scales.iter().copied().fold(f32::MAX, f32::min);
            assert!(max <= MAX_SCALE, "sila {strength}: mnoznik {max} ponad limit");
            assert!(min >= 1.0 / MAX_SCALE, "sila {strength}: mnoznik {min} ponizej limitu");
            assert!(scales.iter().all(|s| s.is_finite()), "sila {strength}: mnoznik nieskonczony");
        }
    }

    #[test]
    fn scaling_touches_whole_rows_and_only_them() {
        let mut values = vec![1.0f32; 6]; // 3 symbole x 2 liczby
        apply_row_scales(&mut values, &[2.0, 1.0, 0.5], 2);
        assert_eq!(values, vec![2.0, 2.0, 1.0, 1.0, 0.5, 0.5]);
    }

    #[test]
    fn nwrd_round_trips_through_disk() {
        // Format zapisu jest kontraktem z Noworodkiem. Plik, ktorego on nie wczyta,
        // jest bezwartosciowy, a bledu nie zobaczylibysmy az do uruchomienia treningu.
        let dir = std::env::temp_dir().join(format!("darkstar-seed-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        let path = dir.join("t.nwrd");
        let p = path.to_string_lossy().to_string();

        let original = vec![
            Tensor { name: "a.weight".into(), values: vec![1.0, -2.5, 3.25] },
            Tensor { name: "b.weight".into(), values: vec![0.0; 4] },
        ];
        write_nwrd(&p, &original).expect("zapis");
        let back = read_nwrd(&p).expect("odczyt");

        assert_eq!(back.len(), 2);
        assert_eq!(back[0].name, "a.weight");
        assert_eq!(back[0].values, vec![1.0, -2.5, 3.25]);
        assert_eq!(back[1].values.len(), 4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn wrong_magic_is_refused() {
        let dir = std::env::temp_dir().join(format!("darkstar-seed-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog");
        let path = dir.join("bad.nwrd");
        std::fs::write(&path, b"XXXX\x01\x00\x00\x00").expect("zapis");
        // `match` zamiast `expect_err`, bo to drugie wymagaloby wypisania calej zawartosci
        // wag w komunikacie bledu - a to dziesiatki megabajtow liczb.
        match read_nwrd(&path.to_string_lossy()) {
            Ok(_) => panic!("plik z zlym znacznikiem nie moze zostac przyjety"),
            Err(e) => assert!(e.to_string().contains("NWRD"), "komunikat: {e}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
