// darkstar-header-v1
// po co: pilnuje, zeby masowa zmiana nazwy nie zjadla zdan, ktore ta zmiane tlumacza
// nie wolno: wylaczac tego testu, zeby przepchnac rename
// autor: Claude Opus 5 / Cowork, na zlecenie Marcina
// powstal: 2026-09-21
//
// DLACZEGO TO ISTNIEJE
// Commit 2cd32c4 ("feat(darkstar): rename Sheriff perimeter to Warlock with
// alias") puscil masowa podmiane Sheriff -> Warlock po katalogu docs. Podmiana
// weszla takze w zdania, ktore te zmiane OPISYWALY, i zamienila je w zdania bez
// znaczenia. W repo zostalo dwanascie takich linii, miedzy innymi:
//
//     "Warlock replaces the earlier name Warlock."
//     "zmiana nazwy z Warlock Bridge na Warlock Bridge"
//     "utrwalenie zmiany Warlock -> Warlock;"
//     "Warlock (formerly Warlock)"
//     "## 9. Migracja Warlock do Warlock"
//
// Kod byl poprawny. Zepsula sie pamiec o tym, skad ten kod sie wzial.
//
// CZEGO TEN TEST NIE POTRAFI, POWIEDZIANE WPROST
// Lapie zdania SAMOODNOSZACE: takie, ktore mowia "X zastapil X". Z dwunastu
// zepsutych linii dziesiec ma taka postac i sa lapane. Dwie pozostale to samo
// podmienione slowo w zdaniu, ktore o zmianie nazwy nie mowi, na przyklad
// "typed Rust topology still uses Warlock identifiers" tam, gdzie chodzilo o
// stare identyfikatory Sheriff. Takiego bledu nie da sie wykryc z samego
// tekstu, bo zdanie jest poprawne skladniowo i sensowne. Test tego nie udaje.
//
// Pierwsza wersja tego testu byla ogolna: "ta sama nazwa po obu stronach
// dowolnego znacznika zmiany". Odpalona na repo dala osiem falszywych alarmow,
// miedzy innymi na `fn bind_addresses(primary: SocketAddr) -> Vec<SocketAddr>`
// i na wiersz tabeli "port 18080 na CBMS". Regula, ktora krzyczy na kod Rusta,
// jest gorsza niz brak reguly, bo uczy ludzi ja wylaczac. Dlatego ponizej jest
// jawna lista fraz opisujacych zmiane nazwy, a nie zgadywanie.

use std::fs;
use std::path::{Path, PathBuf};

/// Skad brac lewa nazwe pary.
#[derive(Clone, Copy)]
enum Lewa {
    /// Pierwsza nazwa wlasna w linii (dziala dla zdan i wierszy tabeli).
    PoczatekLinii,
    /// Nazwa wlasna stojaca najblizej PRZED prawym znacznikiem.
    NajblizejPrzed,
    /// Pierwsza nazwa wlasna po podanym lewym znaczniku.
    PoZnaczniku(&'static str),
}

/// Jawna lista fraz, ktorymi dokumentacja opisuje zmiane nazwy.
/// Kazda para to: skad wziac stara nazwe, i po czym stoi nowa.
/// Lista NIE jest wyczerpujaca. Nowe sformulowanie trzeba tu dopisac.
const WZORCE: &[(Lewa, &str)] = &[
    (Lewa::PoczatekLinii, "replaces the earlier project name "),
    (Lewa::PoczatekLinii, "replaces the earlier name "),
    (Lewa::NajblizejPrzed, "(formerly "),
    (Lewa::NajblizejPrzed, "renamed to "),
    (Lewa::PoZnaczniku("zmianą nazwy z "), " na "),
    (Lewa::PoZnaczniku("zmiany nazwy z "), " na "),
    (Lewa::PoZnaczniku("Migracja "), " do "),
    (Lewa::PoZnaczniku("migracja "), " do "),
    (Lewa::PoZnaczniku("migracji "), " do "),
    (Lewa::PoZnaczniku("zmiany "), " -> "),
    (Lewa::PoZnaczniku("zmiany "), " → "),
    (Lewa::PoZnaczniku("otrzymuje nazwę "), "wcześniejszą nazwę "),
    (Lewa::PoczatekLinii, "zastępująca "),
    (Lewa::PoZnaczniku("Zmiany nazwy "), " danych "),
];

fn korzen_repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("korzen repozytorium")
        .to_path_buf()
}

fn zbierz_markdown(katalog: &Path, zebrane: &mut Vec<PathBuf>) {
    let Ok(wpisy) = fs::read_dir(katalog) else {
        return;
    };
    for wpis in wpisy.flatten() {
        let sciezka = wpis.path();
        if sciezka.is_dir() {
            zbierz_markdown(&sciezka, zebrane);
        } else if sciezka.extension().and_then(|e| e.to_str()) == Some("md") {
            zebrane.push(sciezka);
        }
    }
}

/// Nazwa wlasna: token zaczynajacy sie wielka litera ASCII, min. cztery znaki.
fn jest_nazwa(token: &str) -> bool {
    token.chars().count() >= 4 && token.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

fn tokeny(fragment: &str) -> impl Iterator<Item = &str> {
    fragment.split(|c: char| !c.is_alphanumeric())
}

fn pierwsza_nazwa(fragment: &str) -> Option<&str> {
    tokeny(fragment).find(|t| jest_nazwa(t))
}

fn ostatnia_nazwa(fragment: &str) -> Option<&str> {
    tokeny(fragment).filter(|t| jest_nazwa(t)).last()
}

/// Zwraca opis naruszenia, jesli linia mowi "X zastapil X".
fn samoodnoszacy_rename(linia: &str) -> Option<String> {
    for (skad_lewa, prawy) in WZORCE {
        let Some(poz) = linia.find(prawy) else {
            continue;
        };
        let po_prawym = &linia[poz + prawy.len()..];
        let Some(nowa) = pierwsza_nazwa(po_prawym) else {
            continue;
        };

        let stara = match skad_lewa {
            Lewa::PoczatekLinii => pierwsza_nazwa(&linia[..poz]),
            Lewa::NajblizejPrzed => ostatnia_nazwa(&linia[..poz]),
            Lewa::PoZnaczniku(lewy) => linia[..poz]
                .find(lewy)
                .and_then(|p| pierwsza_nazwa(&linia[p + lewy.len()..poz])),
        };

        if let Some(stara) = stara
            && stara == nowa
        {
            return Some(format!(
                "zdanie mowi, ze {stara:?} zastapilo {nowa:?} (wzorzec {prawy:?})"
            ));
        }
    }
    None
}

#[test]
fn dokumentacja_nie_zawiera_samoodnoszacych_zmian_nazwy() {
    let korzen = korzen_repo();
    let mut pliki = Vec::new();
    zbierz_markdown(&korzen.join("docs"), &mut pliki);
    assert!(
        !pliki.is_empty(),
        "nie znaleziono zadnego pliku .md w docs/"
    );

    let mut naruszenia = Vec::new();
    for plik in &pliki {
        let Ok(tresc) = fs::read_to_string(plik) else {
            continue;
        };
        for (nr, linia) in tresc.lines().enumerate() {
            if let Some(powod) = samoodnoszacy_rename(linia) {
                naruszenia.push(format!(
                    "{}:{}: {}\n    {}",
                    plik.strip_prefix(&korzen).unwrap_or(plik).display(),
                    nr + 1,
                    powod,
                    linia.trim()
                ));
            }
        }
    }

    assert!(
        naruszenia.is_empty(),
        "znaleziono {} samoodnoszacych zmian nazwy w docs/ \
         (masowy rename zjadl zdania, ktore te zmiane tlumaczyly):\n{}",
        naruszenia.len(),
        naruszenia.join("\n")
    );
}

#[test]
fn lapie_prawdziwe_linie_zostawione_przez_2cd32c4() {
    // Doslowne linie z repo sprzed naprawy. Gdyby regula przestala je lapac,
    // test wyzej bylby ozdoba.
    for zla in [
        "Warlock replaces the earlier name Warlock. Existing Warlock identifiers remain",
        "Warlock replaces the earlier project name Warlock. At checkpoint fd8099e the",
        "REASON FOR CREATION: Ujednolicenie architektury referencyjnej z fazą Ghost Gate i zmianą nazwy z Warlock Bridge na Warlock Bridge.",
        "- utrwalenie zmiany Warlock -> Warlock;",
        "## 9. Migracja Warlock do Warlock",
        "This design covers ... It does not add Warlock (formerly Warlock), Kali, provider registry behavior",
        "Po dołączeniu kolejnej maszyny zewnętrzna warstwa otrzymuje nazwę Warlock, zastępując wcześniejszą nazwę Warlock:",
        "REASON FOR CREATION: Utrwalenie zatwierdzonej architektury natywnej bramy Darkstar, prywatnej sieci Headscale/Headplane oraz migracji Warlock do Warlock.",
        "| Warlock Bridge | PLANNED | Nowa nazwa kanoniczna zastępująca Warlock; wymaga migracji typów i przyszłego hosta. |",
        "7. Zmiany nazwy Warlock nie mogą zerwać odczytu starszych danych Warlock bez jawnej migracji.",
    ] {
        assert!(
            samoodnoszacy_rename(zla).is_some(),
            "regula przestala lapac prawdziwa zepsuta linie: {zla}"
        );
    }
}

#[test]
fn nie_krzyczy_na_poprawne_zdania_i_na_kod() {
    // Te same zdania po naprawie, plus linie, na ktorych pierwsza, zbyt ogolna
    // wersja tego testu dawala falszywy alarm. Test, ktory lapie wszystko, jest
    // tak samo bezuzyteczny jak test, ktory nie lapie niczego.
    for dobra in [
        "Warlock replaces the earlier name Sheriff. Existing Sheriff identifiers remain",
        "REASON FOR CREATION: Ujednolicenie architektury referencyjnej z fazą Ghost Gate i zmianą nazwy z Sheriff Bridge na Warlock Bridge.",
        "- utrwalenie zmiany Sheriff -> Warlock;",
        "## 9. Migracja Sheriff do Warlock",
        "It does not add Warlock (formerly Sheriff), Kali, provider registry behavior",
        "- migracja SheriffBridge -> WarlockBridge z aliasem danych legacy;",
        "| Warlock Bridge | PLANNED | Nowa nazwa kanoniczna zastępująca Sheriff; wymaga migracji typów i przyszłego hosta. |",
        // ponizej: falszywe alarmy pierwszej wersji
        "fn bind_addresses(primary: SocketAddr) -> Vec<SocketAddr> {",
        "| Serwer Darkstar | CBMS, docker `darkstar` | port 18080 na CBMS |",
        "rozmowę otwartą. Świat puka wtedy do Cloudflare, a nie do twojego domu, a Cloudflare",
        "wektor       -> BRAK W MOŚCIE              vector     -> BRAK W MOŚCIE",
        "Istniejacy nadzorca to `AIONS Conductor` w Harmonogramie -> `AUTOSTART AIONS SERVER FULL.bat`",
        "Plik `tokenizer.json` deklaruje **128**. Chroma jawnie nadpisuje to na **256**. Skoro Chroma",
    ] {
        assert!(
            samoodnoszacy_rename(dobra).is_none(),
            "falszywy alarm na poprawnej linii: {dobra}"
        );
    }
}
