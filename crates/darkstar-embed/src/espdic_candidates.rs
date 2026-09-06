// darkstar-header-v1
// THIS IS VERY IMPORTANT!!!
// AUTHOR: M. SZUL
// AI MODEL: Codex; dokladny identyfikator modelu niedostepny
// TIMESTAMP: 2026-09-06
// REASON FOR CREATION: wspoldzielone istniejace generowanie kandydatow dla espdic i tokenize.
// SYSTEM PART: Darkstar / kandydaci EN -> EO.
// ARCHITECTURE FUNCTION: jedno zrodlo logiki ESPDIC, bez wyboru znaczenia.
// MECHANICS: invert przeniesione bez zmiany algorytmu z bin/espdic.rs.
// DEPENDENCIES / LINKS: bin/espdic.rs (Claude Sonnet 5, 2026-09-05);
// ESPDIC (Paul Denisowski, CC-BY), format rdzen : definicje.
// SAFETY / DO-NOT: zachowac wszystkich kandydatow i metadane; bez sieci,
// MiniLM, nowych ID, parsera CBMS, stalych sciezek/portow.
// REVISIONS: 2026-09-06 Codex: ekstrakcja istniejacych Candidate, Stats, invert.

use std::collections::HashMap;

/// Jedno wystapienie: ktory rdzen, na ktorej pozycji definicji, przy jak dlugiej liscie.
///
/// Pozycja i dlugosc sa zapisywane, bo przyszly stopien wyboru bedzie ich potrzebowal.
/// NIE sa tu uzywane do wybierania - probowalem i zmierzylem, ze na tych dwoch liczbach
/// wybor wychodzi gorzej niz bez nich (`error` spadalo z `eraro` na `prierara`).
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub root: String,
    pub position: usize,
    pub of_total: usize,
    /// Cala angielska definicja rdzenia, taka jak w slowniku. To jest jedyny opis znaczenia,
    /// jakim dysponujemy - `kodo : code` i `ĉifro : cipher, code` roznia sie wlasnie tym,
    /// a nie samym rdzeniem.
    pub gloss: String,
}

#[derive(Default)]
pub struct Stats {
    pub roots: usize,
    pub single: usize,
    pub phrases: usize,
    pub empty: usize,
}

/// Odwraca slownik, ZACHOWUJAC wszystkie znaczenia.
///
/// Rdzen wymieniajacy dane haslo trafia na liste zawsze - nawet jesli haslo stoi u niego
/// na ostatnim miejscu wsrod dziesieciu. Odrzucenie takiego wpisu byloby cichym wyborem,
/// a wybor nalezy do osobnego stopnia, ktorego jeszcze nie ma.
pub fn invert(text: &str) -> (HashMap<String, Vec<Candidate>>, Stats) {
    let mut index: HashMap<String, Vec<Candidate>> = HashMap::new();
    let mut stats = Stats::default();
    let mut single_keys = std::collections::HashSet::new();
    let mut phrase_keys = std::collections::HashSet::new();

    for line in text.lines() {
        // Naglowek i komentarze zaczynaja sie krzyzykiem; BOM na poczatku pliku tez tu wpada.
        let line = line.trim_start_matches('\u{feff}');
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((root, defs)) = line.split_once(" : ") else {
            continue;
        };
        let root = root.trim();
        if root.is_empty() {
            continue;
        }
        stats.roots += 1;

        let parts: Vec<&str> = defs.split(',').map(str::trim).collect();
        let total = parts.len();
        for (i, def) in parts.iter().enumerate() {
            // Nawias na POCZATKU znaczy, ze cala pozycja jest objasnieniem, a nie
            // tlumaczeniem: "(adjective ending)" nie jest angielskim haslem. Sprawdzamy to
            // PRZED zdejmowaniem nawiasow - odwrotna kolejnosc zdejmuje nawiasy, po czym
            // pyta o nawias, ktorego juz nie ma, i objasnienia wchodza do slownika jako hasla.
            let def = def.trim();
            if def.starts_with('(') {
                stats.empty += 1;
                continue;
            }
            // Nawias na koncu doprecyzowuje haslo - "memory (computer)". Haslem jest to,
            // co przed nawiasem; doprecyzowanie zostawiamy, bo nie jest slowem do szukania.
            let key = match def.split_once('(') {
                Some((head, _)) => head.trim(),
                None => def,
            }
            .trim_end_matches('.')
            .trim();
            if key.is_empty() {
                stats.empty += 1;
                continue;
            }
            let key = key.to_lowercase();
            if key.contains(' ') {
                phrase_keys.insert(key.clone());
            } else {
                single_keys.insert(key.clone());
            }
            index.entry(key).or_default().push(Candidate {
                root: root.to_string(),
                position: i,
                of_total: total,
                gloss: defs.trim().to_string(),
            });
        }
    }
    stats.single = single_keys.len();
    stats.phrases = phrase_keys.len();
    (index, stats)
}
