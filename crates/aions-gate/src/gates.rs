//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: AIONS master protocol deliverables 3 and 4: the SCCE state machine and bounded recovery existed only as prose the model was asked to obey.
//! MECHANICS:           Loads aions-gates.conf, keeps the current task state in .aions/state, and advances exactly one state only when every gate declared for that transition returns PASS (exit 0). Exit 3, timeouts and spawn errors are UNKNOWN and block. FAIL may trigger a declared repair command, capped at MAX_REPAIR_CAP attempts, each followed by re-running the gate. Every gate run is appended to .aions/evidence.jsonl.
//! SYSTEM PART:         polip-agi / aions-gate / workspace state machine
//! ARCHITECTURE FUNC:   Enforcement plane: the model requests transitions, and this code decides them from command results. It has no path to COMPLETE other than passing gates.
//! DEPENDENCIES/LINKS:  crate::json; aions-gates.conf; sh; tests/gates.rs
//! TECH STACK:          Rust 2024, std only
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

use crate::json;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const STATES: [&str; 14] = [
    "PRE_FLIGHT",
    "SOURCE_OF_TRUTH_LOADED",
    "TARGET_VERIFIED",
    "DUPLICATE_SEARCHED",
    "PLAN_VALIDATED",
    "EXECUTION_AUTHORIZED",
    "MUTATION",
    "TEST",
    "REGRESSION_CHECK",
    "PROVENANCE_CHECK",
    "STATE_UPDATE",
    "LANDING",
    "POST_FLIGHT_VERIFY",
    "COMPLETE",
];

/// Hard ceiling on repair attempts, whatever the config says.
pub const MAX_REPAIR_CAP: u32 = 5;
pub const CONFIG_NAME: &str = "aions-gates.conf";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
    Unknown,
}

impl Outcome {
    pub fn label(self) -> &'static str {
        match self {
            Outcome::Pass => "PASS",
            Outcome::Fail => "FAIL",
            Outcome::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub transition: String,
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub gates: Vec<Gate>,
    /// Transitions explicitly declared gate-free, with the reason.
    pub open: Vec<(String, String)>,
    pub repairs: Vec<(String, String)>,
    pub max_repair: u32,
    pub timeout: Duration,
    pub excludes: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct ConfigError(pub String);

fn valid_transition(t: &str) -> bool {
    let Some((from, to)) = t.split_once("->") else {
        return false;
    };
    let (Some(a), Some(b)) = (index_of(from), index_of(to)) else {
        return false;
    };
    b == a + 1
}

pub fn index_of(state: &str) -> Option<usize> {
    STATES.iter().position(|s| *s == state)
}

impl Config {
    pub fn parse(text: &str) -> Result<Config, ConfigError> {
        let mut cfg = Config {
            max_repair: 3,
            timeout: Duration::from_secs(600),
            ..Default::default()
        };
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let err = |m: &str| ConfigError(format!("{CONFIG_NAME}:{}: {m}", n + 1));
            let (word, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
            let rest = rest.trim();
            match word {
                "gate" => {
                    let (lhs, cmd) = rest
                        .split_once('=')
                        .ok_or_else(|| err("gate needs '= command'"))?;
                    let mut parts = lhs.split_whitespace();
                    let (Some(t), Some(name), None) = (parts.next(), parts.next(), parts.next())
                    else {
                        return Err(err("expected: gate FROM->TO name = command"));
                    };
                    if !valid_transition(t) {
                        return Err(err(&format!(
                            "'{t}' is not a forward transition between adjacent states"
                        )));
                    }
                    if cfg.gates.iter().any(|g| g.name == name) {
                        return Err(err(&format!("duplicate gate name '{name}'")));
                    }
                    cfg.gates.push(Gate {
                        transition: t.into(),
                        name: name.into(),
                        command: cmd.trim().into(),
                    });
                }
                "open" => {
                    let (t, reason) = rest
                        .split_once(char::is_whitespace)
                        .ok_or_else(|| err("open needs a reason"))?;
                    if !valid_transition(t) {
                        return Err(err(&format!(
                            "'{t}' is not a forward transition between adjacent states"
                        )));
                    }
                    cfg.open.push((t.into(), reason.trim().into()));
                }
                "repair" => {
                    let (name, cmd) = rest
                        .split_once('=')
                        .ok_or_else(|| err("repair needs '= command'"))?;
                    cfg.repairs.push((name.trim().into(), cmd.trim().into()));
                }
                "max-repair" => {
                    let v: u32 = rest.parse().map_err(|_| err("max-repair needs a number"))?;
                    cfg.max_repair = v.min(MAX_REPAIR_CAP);
                }
                "timeout" => {
                    let v: u64 = rest.parse().map_err(|_| err("timeout needs seconds"))?;
                    cfg.timeout = Duration::from_secs(v.max(1));
                }
                "exclude" => {
                    let (prefix, reason) = rest
                        .split_once(char::is_whitespace)
                        .unwrap_or((rest, "unspecified"));
                    cfg.excludes.push((prefix.into(), reason.trim().into()));
                }
                other => return Err(err(&format!("unknown directive '{other}'"))),
            }
        }
        for (name, _) in &cfg.repairs {
            if !cfg.gates.iter().any(|g| &g.name == name) {
                return Err(ConfigError(format!("repair for unknown gate '{name}'")));
            }
        }
        for g in &cfg.gates {
            if cfg.open.iter().any(|(t, _)| *t == g.transition) {
                return Err(ConfigError(format!(
                    "{} is declared open but also has gate '{}'",
                    g.transition, g.name
                )));
            }
        }
        Ok(cfg)
    }
}

/// Kill the gate command and everything it spawned.
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Walk up from `start` to find the directory holding the config.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join(CONFIG_NAME).is_file())
        .map(Path::to_path_buf)
}

pub struct Workspace {
    pub root: PathBuf,
    pub config: Config,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskState {
    pub task: String,
    pub state: String,
}

#[derive(Debug)]
pub struct GateRun {
    pub gate: String,
    pub outcome: Outcome,
    pub exit: Option<i32>,
    pub attempt: u32,
    pub output_tail: String,
}

#[derive(Debug)]
pub struct AdvanceReport {
    pub from: String,
    pub to: String,
    pub runs: Vec<GateRun>,
    pub outcome: Outcome,
    pub reason: String,
}

impl Workspace {
    pub fn load(root: &Path) -> Result<Workspace, ConfigError> {
        let text = fs::read_to_string(root.join(CONFIG_NAME))
            .map_err(|e| ConfigError(format!("cannot read {CONFIG_NAME}: {e}")))?;
        Ok(Workspace {
            root: root.to_path_buf(),
            config: Config::parse(&text)?,
        })
    }

    fn dir(&self) -> PathBuf {
        self.root.join(".aions")
    }

    pub fn read_state(&self) -> Option<TaskState> {
        let text = fs::read_to_string(self.dir().join("state")).ok()?;
        let mut task = None;
        let mut state = None;
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("task=") {
                task = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("state=") {
                state = Some(v.to_string());
            }
        }
        let state = state.filter(|s| index_of(s).is_some())?;
        Some(TaskState { task: task?, state })
    }

    fn write_state(&self, st: &TaskState) -> std::io::Result<()> {
        fs::create_dir_all(self.dir())?;
        let tmp = self.dir().join("state.tmp");
        fs::write(
            &tmp,
            format!(
                "task={}\nstate={}\nupdated={}\n",
                st.task,
                st.state,
                json::now_utc()
            ),
        )?;
        fs::rename(tmp, self.dir().join("state"))
    }

    /// Start a task at PRE_FLIGHT. Idempotent for the same task; refuses to clobber another.
    pub fn init(&self, task: &str) -> Result<TaskState, String> {
        if task.trim().is_empty() || task.contains('\n') {
            return Err("task id must be a single non-empty line".into());
        }
        if let Some(cur) = self.read_state() {
            if cur.task == task {
                return Ok(cur);
            }
            if cur.state != "COMPLETE" {
                return Err(format!(
                    "task '{}' is still at {}; finish it or rewind explicitly",
                    cur.task, cur.state
                ));
            }
        }
        let st = TaskState {
            task: task.into(),
            state: STATES[0].into(),
        };
        self.write_state(&st).map_err(|e| e.to_string())?;
        self.evidence(&format!(
            "{{\"ts\":{},\"event\":\"init\",\"task\":{}}}",
            json::string(&json::now_utc()),
            json::string(task)
        ));
        Ok(st)
    }

    /// Move backwards (for repair or re-planning). Forward jumps are impossible here.
    pub fn rewind(&self, target: &str) -> Result<TaskState, String> {
        let cur = self.read_state().ok_or("no active task")?;
        let (Some(t), Some(c)) = (index_of(target), index_of(&cur.state)) else {
            return Err(format!("unknown state '{target}'"));
        };
        if t >= c {
            return Err(format!(
                "rewind must go backwards: {} -> {target} refused",
                cur.state
            ));
        }
        let st = TaskState {
            task: cur.task.clone(),
            state: target.into(),
        };
        self.write_state(&st).map_err(|e| e.to_string())?;
        self.evidence(&format!(
            "{{\"ts\":{},\"event\":\"rewind\",\"task\":{},\"from\":{},\"to\":{}}}",
            json::string(&json::now_utc()),
            json::string(&cur.task),
            json::string(&cur.state),
            json::string(target)
        ));
        Ok(st)
    }

    /// Append one JSON line of evidence. Failure to record evidence is reported, never hidden.
    fn evidence(&self, line: &str) {
        let res = fs::create_dir_all(self.dir()).and_then(|_| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.dir().join("evidence.jsonl"))
                .and_then(|mut f| writeln!(f, "{line}"))
        });
        if let Err(e) = res {
            eprintln!("aions-gate: WARNING evidence not recorded: {e}");
        }
    }

    fn run_command(&self, cmd: &str) -> (Outcome, Option<i32>, String) {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Own process group, so a timeout can kill grandchildren that still hold the pipes.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let child = command.spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(e) => return (Outcome::Unknown, None, format!("spawn failed: {e}")),
        };
        // Drain pipes on threads so a chatty command cannot deadlock on a full pipe.
        let mut out = child.stdout.take().expect("piped stdout");
        let mut err = child.stderr.take().expect("piped stderr");
        let t_out = std::thread::spawn(move || {
            let mut s = Vec::new();
            let _ = out.read_to_end(&mut s);
            s
        });
        let t_err = std::thread::spawn(move || {
            let mut s = Vec::new();
            let _ = err.read_to_end(&mut s);
            s
        });
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break Some(s),
                Ok(None) if started.elapsed() >= self.config.timeout => {
                    kill_tree(&mut child);
                    break None;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                Err(_) => break None,
            }
        };
        let mut combined = t_out.join().unwrap_or_default();
        combined.extend(t_err.join().unwrap_or_default());
        let text = String::from_utf8_lossy(&combined);
        let tail: String = {
            let chars: Vec<char> = text.chars().collect();
            chars[chars.len().saturating_sub(2000)..].iter().collect()
        };
        match status {
            None => (
                Outcome::Unknown,
                None,
                format!(
                    "{tail}\n[timeout or wait error after {:?}]",
                    self.config.timeout
                ),
            ),
            Some(s) => match s.code() {
                Some(0) => (Outcome::Pass, Some(0), tail),
                Some(3) => (Outcome::Unknown, Some(3), tail),
                Some(c) => (Outcome::Fail, Some(c), tail),
                None => (
                    Outcome::Unknown,
                    None,
                    format!("{tail}\n[terminated by signal]"),
                ),
            },
        }
    }

    fn record(&self, task: &str, transition: &str, run: &GateRun, kind: &str) {
        self.evidence(&format!(
            "{{\"ts\":{},\"event\":{},\"task\":{},\"transition\":{},\"gate\":{},\"attempt\":{},\"exit\":{},\"verdict\":{},\"output_tail\":{}}}",
            json::string(&json::now_utc()),
            json::string(kind),
            json::string(task),
            json::string(transition),
            json::string(&run.gate),
            run.attempt,
            run.exit.map_or("null".to_string(), |c| c.to_string()),
            json::string(run.outcome.label()),
            json::string(&run.output_tail)
        ));
    }

    /// Try to move the active task forward by exactly one state.
    pub fn advance(&self) -> Result<AdvanceReport, String> {
        let cur = self
            .read_state()
            .ok_or("no active task: run `aions-gate state init <task>`")?;
        let idx = index_of(&cur.state).ok_or("corrupt state file")?;
        if idx + 1 >= STATES.len() {
            return Err("task is already COMPLETE".into());
        }
        let next = STATES[idx + 1];
        let transition = format!("{}->{next}", cur.state);
        let gates: Vec<&Gate> = self
            .config
            .gates
            .iter()
            .filter(|g| g.transition == transition)
            .collect();
        let mut report = AdvanceReport {
            from: cur.state.clone(),
            to: next.into(),
            runs: Vec::new(),
            outcome: Outcome::Pass,
            reason: String::new(),
        };

        if gates.is_empty() {
            match self.config.open.iter().find(|(t, _)| *t == transition) {
                Some((_, reason)) => report.reason = format!("declared open: {reason}"),
                None => {
                    report.outcome = Outcome::Unknown;
                    report.reason = format!(
                        "no gate and no 'open' declaration for {transition}; UNKNOWN is not PASS"
                    );
                }
            }
        }

        for gate in gates {
            let mut attempt = 0;
            loop {
                let (outcome, exit, tail) = self.run_command(&gate.command);
                let run = GateRun {
                    gate: gate.name.clone(),
                    outcome,
                    exit,
                    attempt,
                    output_tail: tail,
                };
                self.record(&cur.task, &transition, &run, "gate");
                let repair = self.config.repairs.iter().find(|(n, _)| *n == gate.name);
                let can_repair = outcome == Outcome::Fail
                    && repair.is_some()
                    && attempt < self.config.max_repair;
                report.runs.push(run);
                if !can_repair {
                    if outcome != Outcome::Pass && report.outcome != Outcome::Fail {
                        report.outcome = outcome;
                    }
                    break;
                }
                attempt += 1;
                let (r_out, r_exit, r_tail) = self.run_command(&repair.expect("checked").1);
                let rrun = GateRun {
                    gate: format!("repair:{}", gate.name),
                    outcome: r_out,
                    exit: r_exit,
                    attempt,
                    output_tail: r_tail,
                };
                self.record(&cur.task, &transition, &rrun, "repair");
                report.runs.push(rrun);
            }
        }

        if report.outcome == Outcome::Pass {
            let st = TaskState {
                task: cur.task.clone(),
                state: next.into(),
            };
            self.write_state(&st).map_err(|e| e.to_string())?;
        } else if report.reason.is_empty() {
            report.reason = "at least one gate did not PASS; state unchanged".into();
        }
        self.evidence(&format!(
            "{{\"ts\":{},\"event\":\"advance\",\"task\":{},\"transition\":{},\"verdict\":{},\"reason\":{}}}",
            json::string(&json::now_utc()),
            json::string(&cur.task),
            json::string(&transition),
            json::string(report.outcome.label()),
            json::string(&report.reason)
        ));
        Ok(report)
    }
}
