// darkstar-header-v1
// po co: eo_morph.rs
// nie wolno: CBMS, ksiega, wagi, sense selection, hotspot, Python
// autor: Marcin
// powstal: 2026-09-06
//! Deterministyczny POC rozbioru slow esperanckich na podstawie danych Hunspell.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5 Codex
//! TIMESTAMP: 2026-09-06  Europe/London
//! REASON FOR CREATION: Sprawdzenie krawedzi kanoniczne Esperanto -> morfemy przed
//! przyszlym, osobnym wpieciem do CBMS. Ten modul nie zna CBMS i nie zmienia produkcyjnego
//! tokenizera.
//! SYSTEM PART: Darkstar / eksperymentalna morfologia Esperanto.
//! ARCHITECTURE FUNCTION: Wczytuje stem-y z eo_morf.dic oraz dodatki PFX/SFX z eo_morf.aff;
//! zwraca wszystkie znalezione, deterministycznie uporzadkowane segmentacje.
//! MECHANICS: Hunspell UTF-8, jeden znak flagi; PFX/SFX sa uzywane jako katalog morfemow.
//! To jest POC segmentacji, nie pelny silnik Hunspell i nie rozstrzyga wieloznacznosci.
//! DEPENDENCIES / LINKS: zewnetrzne artefakty Esperantilo: eo_morf.dic i eo_morf.aff.
//! SAFETY / DO-NOT: Brak sciezek w kodzie; nie kodowac wyniku do CBMS i nie zmieniac ksiegi.
//! REVISIONS: 2026-09-06 — pierwsza wersja proof-of-concept.
//! REVISIONS: 2026-09-06 — odwrotne ADD/STRIP, warunki i continuation flags.
//! REVISIONS: 2026-09-06 — NEEDAFFIX i ograniczona semantyka compound flags.
//! ==========================================

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Prefix,
    Root,
    Suffix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Morpheme {
    pub text: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub surface: String,
    pub morphemes: Vec<Morpheme>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum RejectionReason {
    AddMismatch,
    ConditionMismatch,
    RootNotFound,
    RootFlagMissing,
    ContinuationFlagMissing,
    CrossProductForbidden,
    DepthLimit,
    DuplicatePath,
    SpecialFlagUnsupported,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleProvenance {
    pub id: usize,
    pub kind: Kind,
    pub flag: char,
    pub add: String,
    pub strip: String,
    pub condition: String,
    pub continuation: Vec<char>,
    pub cross_product: bool,
    pub input_surface: String,
    pub reconstructed_stem: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectionEvent {
    pub surface: String,
    pub reason: RejectionReason,
    pub rule_id: Option<usize>,
    pub intermediate_stem: String,
}

#[derive(Debug, Clone)]
pub struct AcceptedProvenance {
    pub analysis: Analysis,
    pub root_flags: Vec<char>,
    pub rules: Vec<RuleProvenance>,
}

#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub accepted: Vec<AcceptedProvenance>,
    pub rejections: Vec<RejectionEvent>,
    pub had_candidate_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardCase {
    pub root: String,
    pub surface: String,
    pub rule_ids: Vec<usize>,
    pub morphemes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileCounters {
    pub reverse_step_calls: usize,
    pub unique_reverse_states: usize,
    pub repeated_reverse_states: usize,
    pub max_repeat_count: usize,
    pub total_reverse_candidates: usize,
    pub forward_chain_calls: usize,
    pub total_forward_candidates: usize,
    seen_reverse_states: HashMap<(String, Vec<usize>), usize>,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidDictionary(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::InvalidDictionary(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone)]
pub struct Analyzer {
    roots: Vec<String>,
    root_flags: HashMap<String, Vec<Vec<char>>>,
    root_index: HashMap<String, Vec<usize>>,
    prefix_by_add: HashMap<String, Vec<usize>>,
    suffix_by_add: HashMap<String, Vec<usize>>,
    prefix_lengths: Vec<usize>,
    suffix_lengths: Vec<usize>,
    rules_by_flag: HashMap<char, Vec<usize>>,
    prefix_rule_ids: Vec<usize>,
    suffix_rule_ids: Vec<usize>,
    profile: RefCell<ProfileCounters>,
    rules: Vec<Rule>,
    need_affix: Option<char>,
    only_in_compound: Option<char>,
    compound_flag: Option<char>,
    compound_permit_flag: Option<char>,
    compound_begin: Option<char>,
    compound_forbid_flag: Option<char>,
}

#[derive(Debug, Clone)]
struct Rule {
    id: usize,
    kind: Kind,
    flag: char,
    strip: String,
    add: String,
    continuation: Vec<char>,
    condition: String,
    cross_product: bool,
}

impl Analyzer {
    fn root_has_flag(&self, root: &str, flag: char) -> bool {
        self.root_index
            .get(root)
            .is_some_and(|entries| !entries.is_empty())
            && self
                .root_flags
                .get(root)
                .is_some_and(|variants| variants.iter().any(|flags| flags.contains(&flag)))
    }

    fn root_flags_for_provenance(&self, root: &str) -> Vec<char> {
        self.root_flags
            .get(root)
            .and_then(|variants| variants.first().cloned())
            .unwrap_or_default()
    }

    pub fn from_files(dic: impl AsRef<Path>, aff: impl AsRef<Path>) -> Result<Self, Error> {
        let dic_text = fs::read_to_string(dic)?;
        let aff_text = fs::read_to_string(aff)?;
        Self::from_text(&dic_text, &aff_text)
    }

    pub fn from_text(dic: &str, aff: &str) -> Result<Self, Error> {
        let mut roots = HashSet::new();
        let mut root_flags: HashMap<String, Vec<Vec<char>>> = HashMap::new();
        let mut root_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut entry_index = 0usize;
        for (line_no, line) in dic.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line_no == 0 {
                continue;
            }
            let (stem, flags) = line
                .split_once('/')
                .map_or((line, ""), |(stem, flags)| (stem, flags));
            let stem = stem.trim();
            if stem.chars().count() >= 3 {
                let root = stem.to_lowercase();
                roots.insert(root.clone());
                root_index
                    .entry(root.clone())
                    .or_default()
                    .push(entry_index);
                root_flags
                    .entry(root)
                    .or_default()
                    .push(flags.chars().collect());
                entry_index += 1;
            }
        }
        if roots.is_empty() {
            return Err(Error::InvalidDictionary(
                "eo_morf.dic nie zawiera stem-ow".into(),
            ));
        }
        let mut rules = Vec::new();
        let mut active: Option<(Kind, char, bool)> = None;
        let mut need_affix = None;
        let mut only_in_compound = None;
        let mut compound_flag = None;
        let mut compound_permit_flag = None;
        let mut compound_begin = None;
        let mut compound_forbid_flag = None;
        for line in aff.lines() {
            let mut fields = line.split_whitespace();
            let Some(kind) = fields.next() else { continue };
            match kind {
                "NEEDAFFIX" => need_affix = fields.next().and_then(|x| x.chars().next()),
                "ONLYINCOMPOUND" => only_in_compound = fields.next().and_then(|x| x.chars().next()),
                "COMPOUNDFLAG" => compound_flag = fields.next().and_then(|x| x.chars().next()),
                "COMPOUNDPERMITFLAG" => {
                    compound_permit_flag = fields.next().and_then(|x| x.chars().next())
                }
                "COMPOUNDBEGIN" => compound_begin = fields.next().and_then(|x| x.chars().next()),
                "COMPOUNDFORBIDFLAG" => {
                    compound_forbid_flag = fields.next().and_then(|x| x.chars().next())
                }
                _ => {}
            }
            if matches!(
                kind,
                "NEEDAFFIX"
                    | "ONLYINCOMPOUND"
                    | "COMPOUNDFLAG"
                    | "COMPOUNDPERMITFLAG"
                    | "COMPOUNDBEGIN"
                    | "COMPOUNDFORBIDFLAG"
            ) {
                continue;
            }
            if kind != "PFX" && kind != "SFX" {
                continue;
            }
            let Some(flag_text) = fields.next() else {
                continue;
            };
            let Some(strip_or_y) = fields.next() else {
                continue;
            };
            if strip_or_y == "Y" {
                let cross = strip_or_y == "Y";
                active = Some((
                    if kind == "PFX" {
                        Kind::Prefix
                    } else {
                        Kind::Suffix
                    },
                    flag_text.chars().next().unwrap_or('\0'),
                    cross,
                ));
                continue;
            }
            let Some((ref rule_kind, rule_flag, cross_product)) = active else {
                continue;
            };
            let Some(add) = fields.next() else {
                continue;
            };
            let (add, continuation) = add.split_once('/').map_or((add, ""), |(a, c)| (a, c));
            let condition = fields.next().unwrap_or(".").to_string();
            let strip = if strip_or_y == "0" {
                String::new()
            } else {
                strip_or_y.to_lowercase()
            };
            let add = match add {
                "0" | "-" => String::new(),
                value => value.to_lowercase(),
            };
            let id = rules.len();
            rules.push(Rule {
                id,
                kind: rule_kind.clone(),
                flag: rule_flag,
                strip,
                add,
                continuation: continuation.chars().collect(),
                condition,
                cross_product,
            });
        }
        let mut prefix_by_add: HashMap<String, Vec<usize>> = HashMap::new();
        let mut suffix_by_add: HashMap<String, Vec<usize>> = HashMap::new();
        let mut rules_by_flag: HashMap<char, Vec<usize>> = HashMap::new();
        let mut prefix_rule_ids = Vec::new();
        let mut suffix_rule_ids = Vec::new();
        for rule in &rules {
            rules_by_flag.entry(rule.flag).or_default().push(rule.id);
            if rule.add.is_empty() {
                continue;
            }
            let index = match rule.kind {
                Kind::Prefix => {
                    prefix_rule_ids.push(rule.id);
                    &mut prefix_by_add
                }
                Kind::Suffix => {
                    suffix_rule_ids.push(rule.id);
                    &mut suffix_by_add
                }
                Kind::Root => continue,
            };
            index.entry(rule.add.clone()).or_default().push(rule.id);
        }
        let mut prefix_lengths: Vec<_> = prefix_by_add.keys().map(String::len).collect();
        prefix_lengths.sort_unstable();
        prefix_lengths.dedup();
        let mut suffix_lengths: Vec<_> = suffix_by_add.keys().map(String::len).collect();
        suffix_lengths.sort_unstable();
        let mut roots: Vec<_> = roots.into_iter().collect();
        roots.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        Ok(Self {
            roots,
            root_flags,
            root_index,
            prefix_by_add,
            suffix_by_add,
            prefix_lengths,
            suffix_lengths,
            rules_by_flag,
            prefix_rule_ids,
            suffix_rule_ids,
            profile: RefCell::new(ProfileCounters::default()),
            rules,
            need_affix,
            only_in_compound,
            compound_flag,
            compound_permit_flag,
            compound_begin,
            compound_forbid_flag,
        })
    }

    pub fn analyze(&self, surface: &str) -> Vec<Analysis> {
        let surface = surface.trim().to_lowercase();
        if surface.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        self.reverse_step(&surface, Vec::new(), &mut out, 0);
        self.compound_step(&surface, &mut out);
        out.sort_by(|a, b| {
            let a_key: Vec<&str> = a.morphemes.iter().map(|m| m.text.as_str()).collect();
            let b_key: Vec<&str> = b.morphemes.iter().map(|m| m.text.as_str()).collect();
            a_key.cmp(&b_key)
        });
        out.dedup();
        out
    }

    fn compound_step(&self, surface: &str, out: &mut Vec<Analysis>) {
        let (Some(compound_flag), Some(compound_begin)) = (self.compound_flag, self.compound_begin)
        else {
            return;
        };
        if surface.chars().count() < 2 {
            return;
        }
        let boundaries: Vec<usize> = surface
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(surface.len()))
            .collect();
        let mut paths: Vec<Vec<String>> = Vec::new();
        self.compound_parts(
            surface,
            &boundaries,
            0,
            Vec::new(),
            &mut paths,
            compound_flag,
            compound_begin,
        );
        for parts in paths.into_iter().take(128) {
            if parts.len() < 2 {
                continue;
            }
            let mut morphemes = Vec::new();
            for part in parts {
                morphemes.push(Morpheme {
                    text: part,
                    kind: Kind::Root,
                });
            }
            out.push(Analysis {
                surface: surface.to_string(),
                morphemes,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compound_parts(
        &self,
        surface: &str,
        boundaries: &[usize],
        start: usize,
        parts: Vec<String>,
        out: &mut Vec<Vec<String>>,
        compound_flag: char,
        compound_begin: char,
    ) {
        if parts.len() >= 8 {
            return;
        }
        for end_index in (start + 1)..boundaries.len() {
            let end = boundaries[end_index];
            let part = &surface[boundaries[start]..end];
            let Some(variants) = self.root_flags.get(part) else {
                continue;
            };
            if self
                .compound_forbid_flag
                .is_some_and(|flag| variants.iter().any(|flags| flags.contains(&flag)))
            {
                continue;
            }
            let first_ok = parts.is_empty()
                && (self.root_has_flag(part, compound_begin)
                    || self.root_has_flag(part, compound_flag));
            let later_ok = !parts.is_empty()
                && (self.root_has_flag(part, compound_flag)
                    || self
                        .compound_permit_flag
                        .is_some_and(|f| self.root_has_flag(part, f)));
            if !first_ok && !later_ok {
                continue;
            }
            let mut next = parts.clone();
            next.push(part.to_string());
            if end == surface.len() {
                out.push(next);
            } else {
                self.compound_parts(
                    surface,
                    boundaries,
                    end_index,
                    next,
                    out,
                    compound_flag,
                    compound_begin,
                );
            }
        }
    }

    pub fn diagnose(&self, surface: &str) -> Diagnostics {
        let surface = surface.trim().to_lowercase();
        let mut diagnostics = Diagnostics::default();
        if surface.is_empty() {
            return diagnostics;
        }
        self.diagnose_step(&surface, Vec::new(), Vec::new(), &mut diagnostics, 0);
        diagnostics
    }

    pub fn generate_forward_cases(&self, max_depth: usize, max_cases: usize) -> Vec<ForwardCase> {
        let mut cases = Vec::new();
        for root in &self.roots {
            for (index, rule) in self.rules.iter().enumerate() {
                if !self.root_has_flag(root, rule.flag) {
                    continue;
                }
                if let Some(surface) = apply_rule(root, rule) {
                    cases.push(ForwardCase {
                        root: root.clone(),
                        surface: surface.clone(),
                        rule_ids: vec![index],
                        morphemes: self.ordered_morphemes(root, &[index]),
                    });
                    self.forward_chain(
                        &root.clone(),
                        &surface,
                        vec![index],
                        max_depth,
                        max_cases,
                        &mut cases,
                    );
                    if cases.len() >= max_cases {
                        return cases;
                    }
                }
            }
        }
        cases
    }

    /// One deterministic legal single-affix example for each applicable rule.
    pub fn generate_single_forward_cases(&self) -> Vec<ForwardCase> {
        let mut cases = Vec::new();
        for (index, rule) in self.rules.iter().enumerate() {
            let Some(root) = self
                .roots
                .iter()
                .find(|root| self.root_has_flag(root, rule.flag))
            else {
                continue;
            };
            if let Some(surface) = apply_rule(root, rule) {
                cases.push(ForwardCase {
                    root: root.clone(),
                    surface,
                    rule_ids: vec![index],
                    morphemes: self.ordered_morphemes(root, &[index]),
                });
            }
        }
        cases
    }

    pub fn generate_continuation_cases(
        &self,
        max_depth: usize,
        max_cases: usize,
    ) -> Vec<ForwardCase> {
        let seeds = self.generate_single_forward_cases();
        let mut cases = Vec::new();
        for seed in seeds {
            self.forward_chain(
                &seed.root,
                &seed.surface,
                seed.rule_ids,
                max_depth,
                max_cases,
                &mut cases,
            );
            if cases.len() >= max_cases {
                break;
            }
        }
        cases
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn reset_profile(&self) {
        *self.profile.borrow_mut() = ProfileCounters::default();
    }

    pub fn profile(&self) -> ProfileCounters {
        self.profile.borrow().clone()
    }

    pub fn applicable_rule_count(&self) -> usize {
        self.rules
            .iter()
            .filter(|rule| {
                self.root_flags
                    .values()
                    .any(|variants| variants.iter().any(|flags| flags.contains(&rule.flag)))
            })
            .count()
    }

    pub fn need_affix_root_count(&self) -> usize {
        self.root_flags
            .values()
            .filter(|variants| {
                self.need_affix
                    .is_some_and(|flag| variants.iter().any(|flags| flags.contains(&flag)))
            })
            .count()
    }

    fn ordered_morphemes(&self, root: &str, ids: &[usize]) -> Vec<String> {
        let mut out = Vec::new();
        for &i in ids {
            if self.rules[i].kind == Kind::Prefix {
                out.push(self.rules[i].add.clone());
            }
        }
        out.push(root.to_string());
        for &i in ids {
            if self.rules[i].kind == Kind::Suffix {
                out.push(self.rules[i].add.clone());
            }
        }
        out
    }

    fn forward_chain(
        &self,
        root: &String,
        surface: &str,
        chain: Vec<usize>,
        max_depth: usize,
        max_cases: usize,
        out: &mut Vec<ForwardCase>,
    ) {
        if chain.len() >= max_depth || out.len() >= max_cases {
            return;
        }
        self.profile.borrow_mut().forward_chain_calls += 1;
        let previous = &self.rules[*chain.last().unwrap()];
        let mut candidates = Vec::new();
        for flag in &previous.continuation {
            if let Some(ids) = self.rules_by_flag.get(flag) {
                candidates.extend(ids.iter().copied());
            }
        }
        if previous.cross_product {
            if previous.kind == Kind::Prefix {
                candidates.extend(self.suffix_rule_ids.iter().copied());
            } else {
                candidates.extend(self.prefix_rule_ids.iter().copied());
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        self.profile.borrow_mut().total_forward_candidates += candidates.len();
        for index in candidates {
            if chain.contains(&index) {
                continue;
            }
            let rule = &self.rules[index];
            let continuation_ok = previous.continuation.contains(&rule.flag);
            let cross_ok = previous.kind != rule.kind && previous.cross_product;
            if !continuation_ok && !cross_ok {
                continue;
            }
            let Some(next_surface) = apply_rule(surface, rule) else {
                continue;
            };
            let mut next_chain = chain.clone();
            next_chain.push(index);
            let morphemes = self.ordered_morphemes(root, &next_chain);
            out.push(ForwardCase {
                root: root.clone(),
                surface: next_surface.clone(),
                rule_ids: next_chain.clone(),
                morphemes,
            });
            self.forward_chain(root, &next_surface, next_chain, max_depth, max_cases, out);
            if out.len() >= max_cases {
                return;
            }
        }
    }

    fn diagnose_step(
        &self,
        surface: &str,
        reverse_rules: Vec<usize>,
        provenance: Vec<RuleProvenance>,
        diagnostics: &mut Diagnostics,
        depth: usize,
    ) {
        if depth > 8 {
            diagnostics.rejections.push(RejectionEvent {
                surface: surface.to_string(),
                reason: RejectionReason::DepthLimit,
                rule_id: None,
                intermediate_stem: surface.to_string(),
            });
            return;
        }
        let mut found_root = false;
        for root in &self.roots {
            if surface != root {
                continue;
            }
            found_root = true;
            diagnostics.had_candidate_path = true;
            let mut rules = reverse_rules.clone();
            rules.reverse();
            let mut prefixes = Vec::new();
            let mut suffixes = Vec::new();
            for &rule_index in &rules {
                let rule = &self.rules[rule_index];
                if rule.add.is_empty() {
                    continue;
                }
                let morpheme = Morpheme {
                    text: rule.add.clone(),
                    kind: rule.kind.clone(),
                };
                if rule.kind == Kind::Prefix {
                    prefixes.push(morpheme);
                } else {
                    suffixes.push(morpheme);
                }
            }
            let mut morphemes = prefixes;
            morphemes.push(Morpheme {
                text: root.clone(),
                kind: Kind::Root,
            });
            morphemes.extend(suffixes);
            let analysis = Analysis {
                surface: surface.to_string(),
                morphemes,
            };
            if let Some(reason) = self.validation_reason(&analysis, &rules) {
                diagnostics.rejections.push(RejectionEvent {
                    surface: surface.to_string(),
                    reason,
                    rule_id: rules.first().copied().map(|i| self.rules[i].id),
                    intermediate_stem: surface.to_string(),
                });
            } else {
                diagnostics.accepted.push(AcceptedProvenance {
                    analysis,
                    root_flags: self.root_flags_for_provenance(root),
                    rules: provenance.iter().cloned().rev().collect(),
                });
            }
        }
        if !found_root && reverse_rules.is_empty() {
            diagnostics.rejections.push(RejectionEvent {
                surface: surface.to_string(),
                reason: RejectionReason::RootNotFound,
                rule_id: None,
                intermediate_stem: surface.to_string(),
            });
        }
        for (index, rule) in self.rules.iter().enumerate() {
            if rule.add.is_empty() || reverse_rules.contains(&index) {
                continue;
            }
            let Some(stem) = reverse_rule(surface, rule) else {
                continue;
            };
            diagnostics.had_candidate_path = true;
            if !condition_matches(&rule.condition, &stem, &rule.kind) {
                diagnostics.rejections.push(RejectionEvent {
                    surface: surface.to_string(),
                    reason: RejectionReason::ConditionMismatch,
                    rule_id: Some(rule.id),
                    intermediate_stem: stem,
                });
                continue;
            }
            let mut next_rules = reverse_rules.clone();
            next_rules.push(index);
            let mut next_provenance = provenance.clone();
            next_provenance.push(RuleProvenance {
                id: rule.id,
                kind: rule.kind.clone(),
                flag: rule.flag,
                add: rule.add.clone(),
                strip: rule.strip.clone(),
                condition: rule.condition.clone(),
                continuation: rule.continuation.clone(),
                cross_product: rule.cross_product,
                input_surface: surface.to_string(),
                reconstructed_stem: stem.clone(),
            });
            self.diagnose_step(&stem, next_rules, next_provenance, diagnostics, depth + 1);
        }
    }

    fn applicable(&self, analysis: &Analysis, rules: &[usize]) -> bool {
        self.validation_reason(analysis, rules).is_none()
    }

    fn validation_reason(&self, analysis: &Analysis, rules: &[usize]) -> Option<RejectionReason> {
        let Some(root) = analysis
            .morphemes
            .iter()
            .find(|m| m.kind == Kind::Root)
            .map(|m| m.text.as_str())
        else {
            return Some(RejectionReason::RootNotFound);
        };
        if self
            .only_in_compound
            .is_some_and(|flag| self.root_has_flag(root, flag))
        {
            return Some(RejectionReason::SpecialFlagUnsupported);
        }
        if rules.is_empty() {
            return if self
                .need_affix
                .is_some_and(|flag| self.root_has_flag(root, flag))
            {
                Some(RejectionReason::RootFlagMissing)
            } else {
                None
            };
        }
        let first = &self.rules[rules[0]];
        if !self.root_has_flag(root, first.flag) {
            return Some(RejectionReason::RootFlagMissing);
        }
        for pair in rules.windows(2) {
            let previous = &self.rules[pair[0]];
            let next = &self.rules[pair[1]];
            let continuation_ok = previous.continuation.contains(&next.flag);
            let cross_product_ok = previous.kind != next.kind && previous.cross_product;
            if !continuation_ok && previous.kind != next.kind && !previous.cross_product {
                return Some(RejectionReason::CrossProductForbidden);
            }
            if !continuation_ok && !cross_product_ok {
                return Some(RejectionReason::ContinuationFlagMissing);
            }
        }
        None
    }

    fn reverse_step(
        &self,
        surface: &str,
        reverse_rules: Vec<usize>,
        out: &mut Vec<Analysis>,
        depth: usize,
    ) {
        if depth > 8 || out.len() >= 512 {
            return;
        }
        {
            let mut profile = self.profile.borrow_mut();
            profile.reverse_step_calls += 1;
            let key = (surface.to_string(), reverse_rules.clone());
            let count = {
                let count = profile.seen_reverse_states.entry(key).or_default();
                *count += 1;
                *count
            };
            if count == 1 {
                profile.unique_reverse_states += 1;
            } else {
                profile.repeated_reverse_states += 1;
                profile.max_repeat_count = profile.max_repeat_count.max(count);
            }
        }
        if self.root_flags.contains_key(surface) {
            let root = surface;
            let mut rules = reverse_rules.clone();

            // A NEEDAFFIX root may be licensed by a legal zero-surface rule.
            // This transition carries no textual morpheme, so it must not add
            // an empty Morpheme; it only records the rule in provenance and
            // lets normal applicability checks satisfy NEEDAFFIX.
            if rules.is_empty()
                && self
                    .need_affix
                    .is_some_and(|flag| self.root_has_flag(root, flag))
            {
                for rule in &self.rules {
                    if !rule.add.is_empty()
                        || !rule.strip.is_empty()
                        || !self.root_has_flag(root, rule.flag)
                        || !condition_matches(&rule.condition, root, &rule.kind)
                    {
                        continue;
                    }
                    let zero_surface_rules = vec![rule.id];
                    let analysis = Analysis {
                        surface: surface.to_string(),
                        morphemes: vec![Morpheme {
                            text: root.to_string(),
                            kind: Kind::Root,
                        }],
                    };
                    if self.applicable(&analysis, &zero_surface_rules) {
                        out.push(analysis);
                    }
                }
            }

            rules.reverse();
            let mut prefixes = Vec::new();
            let mut suffixes = Vec::new();
            for &rule_index in &rules {
                let rule = &self.rules[rule_index];
                if !rule.add.is_empty() {
                    let morpheme = Morpheme {
                        text: rule.add.clone(),
                        kind: rule.kind.clone(),
                    };
                    if rule.kind == Kind::Prefix {
                        prefixes.push(morpheme);
                    } else {
                        suffixes.push(morpheme);
                    }
                }
            }
            let mut morphemes = prefixes;
            morphemes.push(Morpheme {
                text: root.to_string(),
                kind: Kind::Root,
            });
            morphemes.extend(suffixes);
            let analysis = Analysis {
                surface: surface.to_string(),
                morphemes,
            };
            if self.applicable(&analysis, &rules) {
                out.push(analysis);
            }
        }
        for index in self.candidate_rule_ids(surface) {
            let rule = &self.rules[index];
            let Some(stem) = reverse_rule(surface, rule) else {
                continue;
            };
            if !condition_matches(&rule.condition, &stem, &rule.kind) {
                continue;
            }
            if reverse_rules.contains(&index) {
                continue;
            }
            let mut next = reverse_rules.clone();
            next.push(index);
            self.reverse_step(&stem, next, out, depth + 1);
        }
    }

    fn candidate_rule_ids(&self, surface: &str) -> Vec<usize> {
        let mut ids = Vec::new();
        for length in &self.prefix_lengths {
            let rules = (*length <= surface.len() && surface.is_char_boundary(*length))
                .then(|| self.prefix_by_add.get(&surface[..*length]))
                .flatten();
            if let Some(rules) = rules {
                ids.extend(rules.iter().copied());
            }
        }
        for length in &self.suffix_lengths {
            if *length <= surface.len() {
                let start = surface.len() - *length;
                let rules = surface
                    .is_char_boundary(start)
                    .then(|| self.suffix_by_add.get(&surface[start..]))
                    .flatten();
                if let Some(rules) = rules {
                    ids.extend(rules.iter().copied());
                }
            }
        }
        ids.sort_unstable();
        ids.dedup();
        self.profile.borrow_mut().total_reverse_candidates += ids.len();
        ids
    }
}

fn reverse_rule(surface: &str, rule: &Rule) -> Option<String> {
    match rule.kind {
        Kind::Prefix => {
            if !surface.starts_with(&rule.add) {
                return None;
            }
            let remainder = &surface[rule.add.len()..];
            Some(format!("{}{}", rule.strip, remainder))
        }
        Kind::Suffix => {
            if !surface.ends_with(&rule.add) {
                return None;
            }
            let remainder = &surface[..surface.len() - rule.add.len()];
            Some(format!("{}{}", remainder, rule.strip))
        }
        Kind::Root => None,
    }
}

fn apply_rule(stem: &str, rule: &Rule) -> Option<String> {
    if !condition_matches(&rule.condition, stem, &rule.kind) {
        return None;
    }
    match rule.kind {
        Kind::Prefix => {
            if !rule.strip.is_empty() && !stem.starts_with(&rule.strip) {
                return None;
            }
            Some(format!("{}{}", rule.add, &stem[rule.strip.len()..]))
        }
        Kind::Suffix => {
            if !rule.strip.is_empty() && !stem.ends_with(&rule.strip) {
                return None;
            }
            let end = stem.len() - rule.strip.len();
            Some(format!("{}{}", &stem[..end], rule.add))
        }
        Kind::Root => None,
    }
}

fn condition_matches(condition: &str, stem: &str, kind: &Kind) -> bool {
    if condition == "." || condition.is_empty() {
        return true;
    }
    let chars: Vec<char> = match kind {
        Kind::Prefix => stem.chars().collect(),
        Kind::Suffix => stem.chars().rev().collect(),
        Kind::Root => return false,
    };
    let pattern: Vec<char> = condition.chars().collect();
    if pattern.len() > chars.len() {
        return false;
    }
    pattern
        .iter()
        .zip(chars.iter())
        .all(|(expected, actual)| *expected == '.' || *expected == *actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIC: &str = "3\nmemor/XN\nsan/XmAN\nkod/XN\nĉifr/XN\n";
    const AFF: &str =
        "PFX m Y 1\nPFX m 0 mal .\nSFX N Y 2\nSFX N 0 o .\nSFX N 0 a .\nSFX A Y 1\nSFX A 0 ul .\n";

    #[test]
    fn parses_stems_and_affixes() {
        let a = Analyzer::from_text(DIC, AFF).unwrap();
        let forms = a.analyze("malsana");
        assert!(forms.iter().any(|x| x
            .morphemes
            .iter()
            .map(|m| m.text.as_str())
            .collect::<Vec<_>>()
            == ["mal", "san", "a"]));
        assert!(a.analyze("unknown").is_empty());
    }

    #[test]
    fn reverses_strip_and_checks_literal_condition() {
        let dic = "2\nbar/X\nfoo/P\n";
        let aff = "SFX X Y 1\nSFX X r o .\nPFX P Y 1\nPFX P 0 pre a\n";
        let analyzer = Analyzer::from_text(dic, aff).unwrap();
        assert!(analyzer.analyze("bao").iter().any(|a| a
            .morphemes
            .iter()
            .map(|m| m.text.as_str())
            .eq(["bar", "o"])));
        assert!(analyzer.analyze("prefoo").is_empty());
        assert!(Analyzer::from_text("2\nfoo/P\napple/P\n", aff)
            .unwrap()
            .analyze("preapple")
            .iter()
            .any(|a| a
                .morphemes
                .iter()
                .map(|m| m.text.as_str())
                .eq(["pre", "apple"])));
    }

    #[test]
    fn applies_real_compound_flag_contract() {
        let dic = "5\nbare/X\nstart/52\ninner/24\nblocked/27\nend/2\n";
        let aff = "NEEDAFFIX X\nONLYINCOMPOUND 4\nCOMPOUNDFLAG 2\nCOMPOUNDPERMITFLAG 3\nCOMPOUNDBEGIN 5\nCOMPOUNDFORBIDFLAG 7\n";
        let analyzer = Analyzer::from_text(dic, aff).unwrap();
        assert!(analyzer.analyze("bare").is_empty());
        assert!(analyzer.analyze("inner").is_empty());
        assert!(analyzer.analyze("startinner").iter().any(|a| {
            a.morphemes
                .iter()
                .map(|m| m.text.as_str())
                .eq(["start", "inner"])
        }));
        assert!(analyzer.analyze("startblocked").is_empty());
    }

    #[test]
    fn zero_surface_affix_satisfies_needaffix_without_empty_morpheme() {
        let analyzer =
            Analyzer::from_text("1\nbare/X\n", "NEEDAFFIX X\nSFX X Y 1\nSFX X 0 0 .\n").unwrap();

        let analyses = analyzer.analyze("bare");
        assert!(analyses.iter().any(|analysis| {
            analysis
                .morphemes
                .iter()
                .map(|morpheme| morpheme.text.as_str())
                .eq(["bare"])
        }));
        assert!(analyses
            .iter()
            .flat_map(|analysis| analysis.morphemes.iter())
            .all(|morpheme| !morpheme.text.is_empty()));

        let bare_only = Analyzer::from_text("1\nbare/X\n", "NEEDAFFIX X\n").unwrap();
        assert!(bare_only.analyze("bare").is_empty());
    }

    #[test]
    fn artifact_examples_if_configured() {
        let Ok(dic) = std::env::var("EO_MORF_DIC") else {
            return;
        };
        let Ok(aff) = std::env::var("EO_MORF_AFF") else {
            return;
        };
        let analyzer = Analyzer::from_files(dic, aff).unwrap();
        for (word, expected) in [
            ("memoro", &["memor", "o"][..]),
            ("memorilo", &["memor", "il", "o"][..]),
            ("malsana", &["mal", "san", "a"][..]),
            ("sanulo", &["san", "ul", "o"][..]),
            ("malsanulejo", &["mal", "san", "ul", "ej", "o"][..]),
            ("kodo", &["kod", "o"][..]),
            ("ĉifro", &["ĉifr", "o"][..]),
        ] {
            let analyses = analyzer.analyze(word);
            println!("{word} -> {analyses:?}");
            assert!(
                analyses.iter().any(|a| a
                    .morphemes
                    .iter()
                    .map(|m| m.text.as_str())
                    .eq(expected.iter().copied())),
                "brak oczekiwanej analizy dla {word}"
            );
        }
    }
}
