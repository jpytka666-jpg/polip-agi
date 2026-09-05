// darkstar-header-v1
// po co: bak_pak_map_of_bak_files.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-06
//! BAK_PAK_MAP_OF_BAK_FILES — mapa kopii zapasowych, zeby nie blondzic we mgle.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-06 00:05:00
//! REASON FOR CREATION: Marcin kazal zostawic pliki .bak - "moga sie przydac jak za miesiac
//! sie okaze ze cos dzis zjebalismy" - i zbudowac do nich automat, ktory potrafi porownywac
//! wersje. Nazwe podal wprost: BAK_PAK_MAP_OF_BAK_FILES.
//! Powod jest mocny: historia projektu zapisuje tylko stany ZATWIERDZONE. Kopie .bak trzymaja
//! stany POMIEDZY nimi - piec kopii lathe.rs powstalo w ciagu dwoch minut, gdy ta sama regula
//! byla poprawiana. Gdy za miesiac cos przestanie dzialac, to jest jedyne miejsce, w ktorym
//! widac, co dokladnie sie wtedy dzialo. Bez narzedzia to jest kupa plikow z liczbami
//! w nazwie i nikt do niej nie zajrzy.
//! MECHANICS: Nazwa kopii ma postac `oryginal.<sekundy>.bak`, gdzie liczba to czas uniksowy.
//! Program grupuje kopie po oryginale, ustawia je w kolejnosci czasu, DOKLADA NA KONIEC PLIK
//! ZYWY jako wersje najnowsza i liczy roznice miedzy sasiednimi wersjami. Tryb `find` szuka
//! tekstu we WSZYSTKICH wersjach naraz i mowi, w ktorej sie pojawil i w ktorej zniknal - to
//! jest odpowiedz na pytanie "kiedy to weszlo", od ktorego zwykle zaczyna sie szukanie usterki.
//! SYSTEM PART: Darkstar / narzedzia robocze.
//! ARCHITECTURE FUNCTION: Zamienia smietnik w os czasu. Niczego nie kasuje i niczego nie
//! zapisuje - tylko czyta i pokazuje.
//! DEPENDENCIES/LINKS: zaden. Czysty odczyt plikow.
//! TECH STACK: Rust 2024, bez zaleznosci zewnetrznych.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! bak-pak-map-of-bak-files map     [katalog]              co w ogole jest
//! bak-pak-map-of-bak-files historia <plik> [katalog]      os czasu jednego pliku
//! bak-pak-map-of-bak-files roznica  <plik> <a> <b> [kat]  co sie zmienilo miedzy wersjami
//! bak-pak-map-of-bak-files szukaj   <tekst> [katalog]     kiedy to weszlo i kiedy znikelo
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Jedna kopia albo plik zywy.
#[derive(Debug, Clone)]
struct Version {
    path: PathBuf,
    /// Czas uniksowy z nazwy kopii. Dla pliku zywego `None` - i to jest wazne rozroznienie,
    /// bo plik zywy jest zawsze najnowszy, ale nie ma wlasnego znacznika.
    stamp: Option<u64>,
    lines: usize,
    bytes: u64,
}

impl Version {
    fn label(&self) -> String {
        match self.stamp {
            Some(t) => czas(t),
            None => "TERAZ (plik zywy)".to_string(),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("");
    let code = match mode {
        "map" => {
            let root = args.get(1).cloned().unwrap_or_else(|| ".".into());
            mapa(&root)
        }
        "historia" => match args.get(1) {
            Some(f) => historia(f, args.get(2).map(String::as_str).unwrap_or(".")),
            None => uzycie(),
        },
        "roznica" => match (args.get(1), args.get(2), args.get(3)) {
            (Some(f), Some(a), Some(b)) => roznica(
                f,
                a.parse().unwrap_or(0),
                b.parse().unwrap_or(0),
                args.get(4).map(String::as_str).unwrap_or("."),
            ),
            _ => uzycie(),
        },
        "szukaj" => match args.get(1) {
            Some(t) => szukaj(t, args.get(2).map(String::as_str).unwrap_or(".")),
            None => uzycie(),
        },
        "wypchnij" => wypchnij(
            args.get(1).map(String::as_str).unwrap_or("."),
            args.iter().any(|a| a == "--na-sucho"),
        ),
        _ => uzycie(),
    };
    std::process::exit(code);
}

fn uzycie() -> i32 {
    eprintln!("BAK_PAK_MAP_OF_BAK_FILES — mapa kopii zapasowych\n");
    eprintln!("  map                                co w ogole jest, pogrupowane");
    eprintln!("  historia <plik> [katalog]          os czasu jednego pliku");
    eprintln!("  roznica  <plik> <a> <b> [katalog]  co sie zmienilo miedzy wersja a i b");
    eprintln!("  szukaj   <tekst> [katalog]         w ktorej wersji tekst wszedl i znikl");
    eprintln!("  wypchnij [katalog] [--na-sucho]    odloz kopie na GitHub, osobna galaz");
    eprintln!("\nTryby czytajace niczego nie kasuja i nie zapisuja.");
    eprintln!("`wypchnij` NIE dotyka twojej biezacej pracy - patrz opis w pliku.");
    2
}

// ---------------------------------------------------------------------------
// wypchniecie na GitHub — osobna galaz, bez dotykania biezacej pracy
// ---------------------------------------------------------------------------

/// Nazwa galezi z kopiami. Bez wspolnej historii z projektem - kopie nigdy nie pojawia sie
/// w galezi roboczej, nie wchodza w drogi przy laczeniu zmian i nie zasmiecaja projektu.
const GALAZ: &str = "bak-pak";

/// Odklada kopie na GitHub.
///
/// Marcin: "TE KOPIE NIECH TWOJ AUTOMAT Z AUTOMATU WYPYCHA NA GH". Powod jest realny:
/// kopie leza w JEDNYM miejscu, wiec maja przetrwac miesiac na jednym dysku.
///
/// Dwie rzeczy, o ktore trzeba tu zadbac, i obie sa zrobione celowo:
///
/// Po pierwsze, to NIE MOZE dotknac biezacej pracy. Zwykle `git add` wpisuje sie do tego
/// samego miejsca, w ktorym siedzi to, co wlasnie przygotowujesz do zapisu - a ten program
/// ma chodzic sam z siebie, takze w polowie twojej edycji. Dlatego buduje wlasny, tymczasowy
/// spis plikow (`GIT_INDEX_FILE`) i pisze prosto do magazynu wersji. Galaz robocza, spis
/// biezacy i pliki na dysku zostaja nietkniete.
///
/// Po drugie, ma nie robic pustych zapisow. Jesli od ostatniego razu nic nie przybylo,
/// konczy bez zapisu - inaczej po tygodniu galaz mialaby setki identycznych wpisow i nie
/// dalo by sie w niej niczego znalezc.
fn wypchnij(root: &str, na_sucho: bool) -> i32 {
    let grupy = zbierz(root);
    let kopie: Vec<&Version> = grupy
        .values()
        .flatten()
        .filter(|v| v.stamp.is_some())
        .collect();
    if kopie.is_empty() {
        println!("Nie ma czego odkladac — zadnych kopii pod: {root}");
        return 0;
    }
    let bajtow: u64 = kopie.iter().map(|v| v.bytes).sum();
    println!("kopii do odlozenia: {}  ({} KB)", kopie.len(), bajtow / 1024);

    if na_sucho {
        println!("\n--na-sucho: nic nie zapisuje. Poszlyby te pliki:");
        for v in kopie.iter().take(30) {
            println!("  {}", v.path.display());
        }
        if kopie.len() > 30 {
            println!("  ... i jeszcze {}", kopie.len() - 30);
        }
        return 0;
    }

    let korzen = match git(root, &["rev-parse", "--show-toplevel"], None) {
        Ok(s) => s.trim().to_string(),
        Err(e) => {
            eprintln!("FAIL: to nie jest kopia robocza projektu: {e}");
            return 1;
        }
    };

    // Wlasny, tymczasowy spis plikow. To jest cala sztuczka: dzieki niemu `git add` nie
    // dotyka tego, co masz przygotowane do zapisu w swojej biezacej pracy.
    let spis = std::env::temp_dir().join(format!("bak-pak-spis-{}", std::process::id()));
    let _ = fs::remove_file(&spis);
    let env = Some(("GIT_INDEX_FILE", spis.to_string_lossy().to_string()));

    // Jesli galaz juz jest, zaczynamy od jej zawartosci - kopie maja sie GROMADZIC,
    // a nie zastepowac poprzednie.
    let rodzic = git(&korzen, &["rev-parse", "--verify", &format!("refs/heads/{GALAZ}")], None)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(r) = &rodzic
        && let Err(e) = git(&korzen, &["read-tree", r], env.clone())
    {
        eprintln!("FAIL: nie moge odczytac poprzedniej zawartosci galezi: {e}");
        let _ = fs::remove_file(&spis);
        return 1;
    }

    let mut dodanych = 0usize;
    for v in &kopie {
        let wzgledna = v
            .path
            .strip_prefix(&korzen)
            .unwrap_or(&v.path)
            .to_string_lossy()
            .replace('\\', "/");
        // `--force`, bo kopie moga byc wykluczone z projektu regula pomijania - a tutaj
        // wlasnie o nie chodzi.
        match git(&korzen, &["add", "--force", &wzgledna], env.clone()) {
            Ok(_) => dodanych += 1,
            Err(e) => eprintln!("  pomijam {wzgledna}: {e}"),
        }
    }
    if dodanych == 0 {
        eprintln!("FAIL: nie udalo sie dodac ani jednej kopii");
        let _ = fs::remove_file(&spis);
        return 1;
    }

    let drzewo = match git(&korzen, &["write-tree"], env.clone()) {
        Ok(s) => s.trim().to_string(),
        Err(e) => {
            eprintln!("FAIL: {e}");
            let _ = fs::remove_file(&spis);
            return 1;
        }
    };
    let _ = fs::remove_file(&spis);

    // Nic nie przybylo - konczymy bez zapisu.
    if let Some(r) = &rodzic
        && let Ok(poprzednie) = git(&korzen, &["rev-parse", &format!("{r}^{{tree}}")], None)
        && poprzednie.trim() == drzewo
    {
        println!("Bez zmian od ostatniego razu — nic nie zapisuje.");
        return 0;
    }

    let opis = format!(
        "bak-pak: {} kopii, {} KB\n\nOdlozone automatycznie. Ta galaz nie ma wspolnej historii\nz projektem - trzyma wylacznie kopie zapasowe plikow roboczych.",
        kopie.len(),
        bajtow / 1024
    );
    let mut arg = vec!["commit-tree", &drzewo, "-m", &opis];
    if let Some(r) = &rodzic {
        arg.push("-p");
        arg.push(r);
    }
    let zapis = match git(&korzen, &arg, None) {
        Ok(s) => s.trim().to_string(),
        Err(e) => {
            eprintln!("FAIL: {e}");
            return 1;
        }
    };
    if let Err(e) = git(&korzen, &["update-ref", &format!("refs/heads/{GALAZ}"), &zapis], None) {
        eprintln!("FAIL: {e}");
        return 1;
    }
    println!("zapisane lokalnie: {} na galezi {GALAZ}", &zapis[..8.min(zapis.len())]);

    match git(&korzen, &["push", "-q", "origin", &format!("{GALAZ}:{GALAZ}")], None) {
        Ok(_) => {
            println!("wyslane na GitHub: galaz {GALAZ}");
            0
        }
        Err(e) => {
            // Brak sieci nie jest utrata danych - kopie sa juz w magazynie wersji lokalnie
            // i pojda przy nastepnym uruchomieniu. Ale trzeba to powiedziec, a nie przemilczec.
            eprintln!("UWAGA: nie wyslane na GitHub ({e})");
            eprintln!("Kopie sa zapisane lokalnie na galezi {GALAZ} i pojda nastepnym razem.");
            0
        }
    }
}

/// Uruchamia polecenie zapisu wersji ZAWSZE we wskazanym katalogu, przez `-C`.
///
/// Bez tego program dzialalby w katalogu, z ktorego go wywolano - a ma chodzic z automatu
/// po kazdej turze, gdy katalogiem biezacym jest cos zupelnie innego. Wtedy nie znalazlby
/// ani projektu, ani kopii, i konczylby cicho, niczego nie odkladajac.
fn git(dir: &str, args: &[&str], env: Option<(&str, String)>) -> Result<String, String> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C").arg(dir).args(args);
    if let Some((k, v)) = env {
        cmd.env(k, v);
    }
    let out = cmd.output().map_err(|e| format!("git nie ruszyl: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

// ---------------------------------------------------------------------------
// zbieranie
// ---------------------------------------------------------------------------

/// Rozklada nazwe kopii na oryginal i czas. `spine.rs.1788631365.bak` -> `spine.rs`, 1788631365.
///
/// Zwraca `None` dla wszystkiego, co nie ma tego ksztaltu - w tym dla plikow konczacych sie
/// na `.bak` bez liczby. Zgadywanie tutaj konczyloby sie grupowaniem niezwiazanych plikow
/// pod jednym oryginalem, a wtedy os czasu klamie.
fn rozloz_nazwe(nazwa: &str) -> Option<(String, u64)> {
    let bez_bak = nazwa.strip_suffix(".bak")?;
    let (reszta, liczba) = bez_bak.rsplit_once('.')?;
    let stamp: u64 = liczba.parse().ok()?;
    if reszta.is_empty() {
        return None;
    }
    Some((reszta.to_string(), stamp))
}

/// Znajduje wszystkie kopie pod wskazanym katalogiem i grupuje je po oryginale.
///
/// Do kazdej grupy DOKLADA plik zywy, jesli istnieje - bez niego nie widac ostatniej zmiany,
/// czyli tej, ktora najczesciej okazuje sie ta szukana.
fn zbierz(root: &str) -> BTreeMap<PathBuf, Vec<Version>> {
    let mut grupy: BTreeMap<PathBuf, Vec<Version>> = BTreeMap::new();
    obejdz(Path::new(root), &mut |sciezka| {
        let Some(nazwa) = sciezka.file_name().and_then(|n| n.to_str()) else {
            return;
        };
        let Some((oryginal, stamp)) = rozloz_nazwe(nazwa) else {
            return;
        };
        let klucz = sciezka.with_file_name(oryginal);
        let (lines, bytes) = zmierz(&sciezka);
        grupy.entry(klucz).or_default().push(Version {
            path: sciezka,
            stamp: Some(stamp),
            lines,
            bytes,
        });
    });

    for (oryginal, wersje) in grupy.iter_mut() {
        wersje.sort_by_key(|v| v.stamp);
        if oryginal.exists() {
            let (lines, bytes) = zmierz(oryginal);
            wersje.push(Version {
                path: oryginal.clone(),
                stamp: None,
                lines,
                bytes,
            });
        }
    }
    grupy
}

/// Obchodzi katalogi, omijajac te, w ktorych kopii i tak nie ma, a ktore potrafia byc ogromne.
fn obejdz(dir: &Path, f: &mut impl FnMut(PathBuf)) {
    let Ok(wpisy) = fs::read_dir(dir) else { return };
    for wpis in wpisy.flatten() {
        let p = wpis.path();
        let nazwa = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if matches!(nazwa, "target" | ".git" | "node_modules") {
                continue;
            }
            obejdz(&p, f);
        } else {
            f(p);
        }
    }
}

fn zmierz(p: &Path) -> (usize, u64) {
    let bytes = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let lines = fs::read_to_string(p).map(|t| t.lines().count()).unwrap_or(0);
    (lines, bytes)
}

/// Czas uniksowy na czytelna date. Bez zaleznosci - wlasna arytmetyka kalendarza.
fn czas(sek: u64) -> String {
    let dni_razem = sek / 86_400;
    let reszta = sek % 86_400;
    let (g, m, s) = (reszta / 3600, (reszta % 3600) / 60, reszta % 60);

    // Od 1970-01-01. Lata przestepne wedlug reguly gregorianskiej.
    let mut rok = 1970u64;
    let mut d = dni_razem;
    loop {
        let dlugosc = if przestepny(rok) { 366 } else { 365 };
        if d < dlugosc {
            break;
        }
        d -= dlugosc;
        rok += 1;
    }
    let dlugosci = [
        31,
        if przestepny(rok) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut miesiac = 0usize;
    while miesiac < 12 && d >= dlugosci[miesiac] {
        d -= dlugosci[miesiac];
        miesiac += 1;
    }
    format!(
        "{rok:04}-{:02}-{:02} {g:02}:{m:02}:{s:02}",
        miesiac + 1,
        d + 1
    )
}

fn przestepny(r: u64) -> bool {
    (r.is_multiple_of(4) && !r.is_multiple_of(100)) || r.is_multiple_of(400)
}

// ---------------------------------------------------------------------------
// tryby
// ---------------------------------------------------------------------------

fn mapa(root: &str) -> i32 {
    let grupy = zbierz(root);
    if grupy.is_empty() {
        println!("Nie znalazlem zadnych kopii pod: {root}");
        return 0;
    }
    let mut kopii = 0usize;
    let mut bajtow = 0u64;
    println!("MAPA KOPII ZAPASOWYCH — {root}\n");
    for (oryginal, wersje) in &grupy {
        let ile_kopii = wersje.iter().filter(|v| v.stamp.is_some()).count();
        kopii += ile_kopii;
        bajtow += wersje.iter().filter(|v| v.stamp.is_some()).map(|v| v.bytes).sum::<u64>();
        let zywy = if wersje.iter().any(|v| v.stamp.is_none()) {
            ""
        } else {
            "   [ORYGINALU JUZ NIE MA]"
        };
        println!("{}{zywy}", oryginal.display());
        println!("  {ile_kopii} kopii, wersji do porownania: {}", wersje.len());
        if let (Some(p), Some(o)) = (wersje.first(), wersje.last()) {
            println!("  od {} do {}", p.label(), o.label());
        }
        println!();
    }
    println!("RAZEM: {} plikow, {kopii} kopii, {} KB", grupy.len(), bajtow / 1024);
    println!("\nDalej:  historia <plik>   |   roznica <plik> <a> <b>   |   szukaj <tekst>");
    0
}

fn historia(plik: &str, root: &str) -> i32 {
    let grupy = zbierz(root);
    let Some((oryginal, wersje)) = dopasuj(&grupy, plik) else {
        eprintln!("Nie znam pliku pasujacego do: {plik}");
        return 1;
    };
    println!("OS CZASU — {}\n", oryginal.display());
    println!("  nr  kiedy                  linii   bajtow   zmiana wobec poprzedniej");
    let mut poprzednia: Option<&Version> = None;
    for (i, v) in wersje.iter().enumerate() {
        let zmiana = match poprzednia {
            None => "  (najstarsza)".to_string(),
            Some(p) => {
                let (a, b) = (czytaj(&p.path), czytaj(&v.path));
                let d = roznica_linii(&a, &b);
                let usuniete = d.iter().filter(|c| matches!(c, Zmiana::Usunieta(_))).count();
                let dodane = d.iter().filter(|c| matches!(c, Zmiana::Dodana(_))).count();
                if usuniete == 0 && dodane == 0 {
                    "  bez zmian".to_string()
                } else {
                    format!("  -{usuniete} +{dodane}")
                }
            }
        };
        println!("  {:>2}  {:<22} {:>5}   {:>6}{zmiana}", i + 1, v.label(), v.lines, v.bytes);
        poprzednia = Some(v);
    }
    println!("\nAby zobaczyc tresc zmiany:  roznica {plik} <a> <b>");
    0
}

fn roznica(plik: &str, a: usize, b: usize, root: &str) -> i32 {
    let grupy = zbierz(root);
    let Some((oryginal, wersje)) = dopasuj(&grupy, plik) else {
        eprintln!("Nie znam pliku pasujacego do: {plik}");
        return 1;
    };
    if a == 0 || b == 0 || a > wersje.len() || b > wersje.len() {
        eprintln!("Numery wersji od 1 do {} — patrz `historia {plik}`", wersje.len());
        return 2;
    }
    let (va, vb) = (&wersje[a - 1], &wersje[b - 1]);
    println!("{}\n  wersja {a}: {}\n  wersja {b}: {}\n", oryginal.display(), va.label(), vb.label());
    let zmiany = roznica_linii(&czytaj(&va.path), &czytaj(&vb.path));
    if zmiany.is_empty() {
        println!("Te dwie wersje sa identyczne.");
        return 0;
    }
    let usuniete = zmiany.iter().filter(|c| matches!(c, Zmiana::Usunieta(_))).count();
    let dodane = zmiany.iter().filter(|c| matches!(c, Zmiana::Dodana(_))).count();
    println!("-{usuniete} linii z wersji {a}, +{dodane} linii w wersji {b}\n");
    for z in &zmiany {
        match z {
            Zmiana::Usunieta(l) => println!("  - {l}"),
            Zmiana::Dodana(l) => println!("  + {l}"),
        }
    }
    0
}

/// Odpowiada na pytanie "kiedy to weszlo" — od ktorego zwykle zaczyna sie szukanie usterki.
fn szukaj(tekst: &str, root: &str) -> i32 {
    let grupy = zbierz(root);
    let igla = tekst.to_lowercase();
    let mut trafien = 0usize;
    println!("SZUKAM: \"{tekst}\"\n");
    for (oryginal, wersje) in &grupy {
        let obecnosc: Vec<bool> = wersje
            .iter()
            .map(|v| czytaj(&v.path).to_lowercase().contains(&igla))
            .collect();
        if !obecnosc.iter().any(|b| *b) {
            continue;
        }
        trafien += 1;
        println!("{}", oryginal.display());
        // Pokazujemy MOMENTY ZMIANY, nie kazda wersje: interesuje nas, kiedy tekst wszedl
        // i kiedy zniknal, a nie to, ze trwal przez dziesiec kolejnych zapisow.
        let mut poprzednio = false;
        for (i, jest) in obecnosc.iter().enumerate() {
            if *jest && !poprzednio {
                println!("  + wszedl   w wersji {:>2}  ({})", i + 1, wersje[i].label());
            } else if !*jest && poprzednio {
                println!("  - zniknal  w wersji {:>2}  ({})", i + 1, wersje[i].label());
            }
            poprzednio = *jest;
        }
        if *obecnosc.last().unwrap_or(&false) {
            println!("  = jest nadal w najnowszej wersji");
        }
        println!();
    }
    if trafien == 0 {
        println!("Nie ma tego w zadnej wersji zadnego pliku.");
    }
    0
}

fn dopasuj<'a>(
    grupy: &'a BTreeMap<PathBuf, Vec<Version>>,
    wzorzec: &str,
) -> Option<(&'a PathBuf, &'a Vec<Version>)> {
    grupy
        .iter()
        .find(|(k, _)| k.to_string_lossy().ends_with(wzorzec))
        .or_else(|| {
            grupy
                .iter()
                .find(|(k, _)| k.to_string_lossy().contains(wzorzec))
        })
}

fn czytaj(p: &Path) -> String {
    fs::read_to_string(p).unwrap_or_default()
}

#[derive(Debug, PartialEq)]
enum Zmiana {
    Usunieta(String),
    Dodana(String),
}

/// Roznica linia po linii, przez najdluzszy wspolny podciag.
///
/// Ta sama metoda, co w `spine-loop`, i z tego samego powodu: porownanie po pozycjach
/// oglosiloby, ze wstawienie jednej linii na gorze zmienilo caly plik. Kod jest tu powtorzony,
/// a nie wyciagniety do wspolnego miejsca, bo wyciagniecie oznaczaloby ruszanie dzialajacego
/// programu tylko po to, zeby oszczedzic trzydziesci linii.
fn roznica_linii(stare: &str, nowe: &str) -> Vec<Zmiana> {
    let a: Vec<&str> = stare.lines().collect();
    let b: Vec<&str> = nowe.lines().collect();
    let (n, m) = (a.len(), b.len());
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            out.push(Zmiana::Usunieta(a[i].to_string()));
            i += 1;
        } else {
            out.push(Zmiana::Dodana(b[j].to_string()));
            j += 1;
        }
    }
    while i < n {
        out.push(Zmiana::Usunieta(a[i].to_string()));
        i += 1;
    }
    while j < m {
        out.push(Zmiana::Dodana(b[j].to_string()));
        j += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rozklada_nazwe_kopii() {
        assert_eq!(
            rozloz_nazwe("spine.rs.1788631365.bak"),
            Some(("spine.rs".to_string(), 1788631365))
        );
        assert_eq!(
            rozloz_nazwe("Cargo.toml.1788631383.bak"),
            Some(("Cargo.toml".to_string(), 1788631383))
        );
    }

    /// Plik bez liczby w nazwie nie jest nasza kopia. Wpuszczenie go zgrupowaloby
    /// niezwiazane pliki pod jednym oryginalem i os czasu klamalaby.
    #[test]
    fn odrzuca_co_nie_ma_znacznika() {
        assert_eq!(rozloz_nazwe("plik.bak"), None);
        assert_eq!(rozloz_nazwe("plik.rs"), None);
        assert_eq!(rozloz_nazwe("plik.abc.bak"), None);
        assert_eq!(rozloz_nazwe(".1788631365.bak"), None);
    }

    /// Czas jest w strefie zerowej (UTC), nie lokalnej - znacznik w nazwie kopii tez taki jest,
    /// wiec przeliczanie na czas lokalny wprowadzaloby przesuniecie zalezne od tego, gdzie
    /// akurat stoi maszyna. Sprawdzone recznie: 1788631365 to 20701 pelnych dni i 64965 sekund,
    /// czyli 18 godzin, 2 minuty, 45 sekund; 20701 dni od 1970-01-01 wypada 2026-09-05.
    #[test]
    fn czas_liczony_poprawnie() {
        assert_eq!(czas(0), "1970-01-01 00:00:00");
        assert_eq!(czas(1788631365), "2026-09-05 18:02:45");
    }

    #[test]
    fn rok_przestepny() {
        assert!(przestepny(2024));
        assert!(!przestepny(2100));
        assert!(przestepny(2000));
    }

    #[test]
    fn wstawienie_linii_to_jedna_zmiana() {
        let d = roznica_linii("a\nb\nc", "nowa\na\nb\nc");
        assert_eq!(d, vec![Zmiana::Dodana("nowa".into())]);
    }

    #[test]
    fn identyczne_daja_pusto() {
        assert!(roznica_linii("a\nb", "a\nb").is_empty());
    }
}
