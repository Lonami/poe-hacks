use super::{ActionResult, MemoryChecker, PlayerStats, PostCondition, PreCondition};
use crate::utils;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

// Avoid spamming actions by default,
// or the server may send "too many actions" on accident.
const DEFAULT_ACTION_DELAY: Duration = Duration::from_millis(500);

const DEFAULT_ACTION_WINDUP: Duration = Duration::ZERO;

#[derive(Debug, PartialEq)]
struct Action {
    pre: Vec<PreCondition>,
    post: PostCondition,
    last_trigger: Instant,
    delay: Duration,
    windup_start: Option<Instant>,
    windup_time: Duration,
    silent: bool,
    _source: String,
}

pub struct ActionSet {
    pub checker: MemoryChecker,
    actions: Vec<Action>,
    inhibit_key_presses: bool,
    created: Instant,
}

enum TriggerResult {
    Success(ActionResult),
    Failed { reason: &'static str },
    Queued,
    Delayed,
}

impl Action {
    fn from_line(line: &str) -> Result<Option<Action>, String> {
        if line.starts_with("//") || line.chars().all(|c| c.is_whitespace()) {
            return Ok(None);
        }

        let mut pre: Vec<PreCondition> = Vec::new();
        let mut post: Option<PostCondition> = None;
        let mut delay = DEFAULT_ACTION_DELAY;
        let mut after = DEFAULT_ACTION_WINDUP;
        let mut silent = false;

        enum State {
            WaitKeyword,

            WaitPreKind,
            WaitLifeValue,
            WaitEsValue,
            WaitManaValue,
            WaitKeyValue,

            WaitPostKind,
            WaitPostValue,
            WaitPostRemaining,

            WaitDelayValue,
            WaitAfterValue,
        }

        let mut state = State::WaitKeyword;
        let line = line.to_lowercase();
        for word in line.split_whitespace() {
            use State::*;
            state = match &state {
                WaitKeyword => match word {
                    "on" => WaitPreKind,
                    "do" => WaitPostKind,
                    "every" => WaitDelayValue,
                    "after" => WaitAfterValue,
                    "silent" => {
                        silent = true;
                        WaitKeyword
                    }
                    _ => return Err(format!("found unexpected keyword '{}'", word)),
                },

                WaitPreKind => match word {
                    "life" => WaitLifeValue,
                    "es" => WaitEsValue,
                    "mana" => WaitManaValue,
                    "flask" | "key" | "skill" => WaitKeyValue,
                    _ => return Err(format!("found unknown condition '{}'", word)),
                },
                WaitLifeValue => {
                    let threshold = utils::parse_value(word)?;
                    pre.push(PreCondition::LifeBelow { threshold });
                    WaitKeyword
                }
                WaitEsValue => {
                    let threshold = utils::parse_value(word)?;
                    pre.push(PreCondition::EnergyBelow { threshold });
                    WaitKeyword
                }
                WaitManaValue => {
                    let threshold = utils::parse_value(word)?;
                    pre.push(PreCondition::ManaBelow { threshold });
                    WaitKeyword
                }
                WaitKeyValue => {
                    pre.push(PreCondition::KeyPress {
                        vk: utils::parse_vk(word)?,
                    });
                    WaitKeyword
                }

                WaitPostKind => match word {
                    "disconnect" => {
                        post = Some(PostCondition::Disconnect);
                        WaitKeyword
                    }
                    "flask" | "key" | "skill" => WaitPostValue,
                    "type" => {
                        post = Some(PostCondition::Type {
                            string: String::new(),
                        });
                        WaitPostRemaining
                    }
                    "price" => {
                        post = Some(PostCondition::ShowPrice);
                        WaitKeyword
                    }
                    "invite" => {
                        post = Some(PostCondition::InviteLast);
                        WaitKeyword
                    }
                    "destroy" => {
                        post = Some(PostCondition::Destroy);
                        WaitKeyword
                    }
                    "downscale" => {
                        post = Some(PostCondition::Downscaling { enable: true });
                        WaitKeyword
                    }
                    "upscale" => {
                        post = Some(PostCondition::Downscaling { enable: false });
                        WaitKeyword
                    }
                    "disable" => {
                        post = Some(PostCondition::SetKeySuppression { suppress: true });
                        WaitKeyword
                    }
                    "enable" => {
                        post = Some(PostCondition::SetKeySuppression { suppress: false });
                        WaitKeyword
                    }
                    _ => return Err(format!("found unknown action '{}'", word)),
                },
                WaitPostValue => {
                    post = Some(PostCondition::PressKey {
                        vk: utils::parse_vk(word)?,
                    });
                    WaitKeyword
                }
                WaitPostRemaining => {
                    match post {
                        Some(PostCondition::Type { ref mut string }) => {
                            if !string.is_empty() {
                                string.push(' ');
                            }
                            string.push_str(word);
                        }
                        _ => return Err(format!("cannot parse remaining unless action is typing")),
                    }
                    WaitPostRemaining
                }

                WaitDelayValue | WaitAfterValue => {
                    let (number, factor) = if word.ends_with("ms") {
                        (&word[..word.len() - 2], 1)
                    } else if word.ends_with("s") {
                        (&word[..word.len() - 1], 1000)
                    } else {
                        return Err(format!("found unknown duration '{}' without ms", word));
                    };

                    let duration = Duration::from_millis(match number.parse::<u64>() {
                        Ok(value) => value * factor,
                        Err(_) => return Err(format!("found unknown duration value '{}'", word)),
                    });

                    match &state {
                        WaitDelayValue => delay = duration,
                        WaitAfterValue => after = duration,
                        _ => unreachable!(),
                    }

                    WaitKeyword
                }
            }
        }

        if pre.is_empty() {
            return Err("it has no trigger condition".into());
        }
        let post = match post {
            Some(post) => post,
            None => return Err("it has no action to perform".into()),
        };

        Ok(Some(Action {
            pre,
            post,
            delay,
            windup_time: after,
            last_trigger: Instant::now() - delay,
            windup_start: None,
            silent,
            _source: line,
        }))
    }

    fn check(&self, stats: &PlayerStats) -> bool {
        self.windup_start.is_some()
            || (self.pre.iter().all(|p| p.is_valid(stats))
                && self.last_trigger.elapsed() > self.delay)
    }

    fn try_trigger(&mut self) -> Result<ActionResult, &'static str> {
        self.last_trigger = Instant::now();
        self.post.act()
    }

    fn trigger(&mut self) -> TriggerResult {
        if self.windup_time > Duration::ZERO {
            let now = Instant::now();
            if let Some(start) = self.windup_start {
                if now < start + self.windup_time {
                    return TriggerResult::Delayed;
                } else {
                    self.windup_start = None;
                }
            } else {
                self.windup_start = Some(now);
                return TriggerResult::Queued;
            }
        }

        match self.try_trigger() {
            Ok(result) => TriggerResult::Success(result),
            Err(reason) => TriggerResult::Failed { reason },
        }
    }
}

impl ActionSet {
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        checker: MemoryChecker,
    ) -> Result<Self, &'static str> {
        let actions: Vec<Action> = match File::open(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Err("poe key file not found");
            }
            Err(_) => {
                return Err("failed to open poe key file, lack of permissions?");
            }
            Ok(file) => BufReader::new(file)
                .lines()
                .map(|line| line.expect("failed to read file"))
                .flat_map(|line| match Action::from_line(&line) {
                    Ok(action) => action,
                    Err(message) => {
                        eprintln!("warning: skipping '{}' because {}", line, message);
                        None
                    }
                })
                .collect(),
        };

        Ok(ActionSet {
            checker,
            actions,
            inhibit_key_presses: false,
            created: Instant::now(),
        })
    }

    pub fn check_all(&mut self) {
        // won't suffer from TOCTOU (all methods rely on information cached during refresh)
        if let Some(stats) = self.checker.player_stats() {
            let actions = &mut self.actions;
            let inhibit_key_presses = &mut self.inhibit_key_presses;
            let skip_key_presses = *inhibit_key_presses;
            let created = &self.created;
            actions
                .iter_mut()
                .filter(|a| !(skip_key_presses && matches!(a.post, PostCondition::PressKey { .. })))
                .filter(|a| a.check(stats))
                .for_each(|a| match a.trigger() {
                    TriggerResult::Success(action_result) => {
                        if !a.silent {
                            eprintln!(
                                "[{:?}; {}] note: ran successfully: {}",
                                created.elapsed(),
                                stats,
                                a
                            );
                        }
                        match action_result {
                            ActionResult::SetKeySuppression { suppress } => {
                                *inhibit_key_presses = suppress;
                            }
                            ActionResult::None => {}
                        }
                    }
                    TriggerResult::Failed { reason } => {
                        eprintln!(
                            "[{:?}; {}] warning: run failed: {}: {}",
                            created.elapsed(),
                            stats,
                            a,
                            reason
                        );
                    }
                    TriggerResult::Queued => {
                        if !a.silent {
                            eprintln!(
                                "[{:?}; {}] note: queued action: {}",
                                created.elapsed(),
                                stats,
                                a
                            );
                        }
                    }
                    TriggerResult::Delayed => {}
                });
        }
    }
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} actions for:", self.actions.len(),)?;
        for action in self.actions.iter() {
            write!(f, "\n- {}", action)?;
        }
        Ok(())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for p in self.pre.iter() {
            write!(f, "on {} ", p)?;
        }
        if self.delay != DEFAULT_ACTION_DELAY {
            write!(f, "every {}ms ", self.delay.as_millis())?;
        }
        if self.windup_time != DEFAULT_ACTION_WINDUP {
            write!(f, "after {}ms ", self.windup_time.as_millis())?;
        }
        if self.silent {
            write!(f, "silent ")?;
        }
        write!(f, "do {}", self.post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Value;

    fn action(line: &str) -> Action {
        Action::from_line(line).unwrap().unwrap()
    }

    #[test]
    fn empty_action() {
        assert_eq!(Action::from_line("\t  \n"), Ok(None));
    }

    #[test]
    fn comment_action() {
        assert_eq!(Action::from_line("// on key 0x1 do disconnect"), Ok(None));
    }

    #[test]
    fn pre_no_post() {
        assert!(Action::from_line("on key 0x1").is_err());
    }

    #[test]
    fn post_no_pre() {
        assert!(Action::from_line("do disconnect").is_err());
    }

    #[test]
    fn life_percent() {
        assert_eq!(
            action("on life 50% do disconnect").pre,
            vec![PreCondition::LifeBelow {
                threshold: Value::Percent(0.5)
            }]
        );
    }

    #[test]
    fn life_flat() {
        assert_eq!(
            action("on life 1000 do disconnect").pre,
            vec![PreCondition::LifeBelow {
                threshold: Value::Flat(1000)
            }]
        );
    }

    #[test]
    fn es_percent() {
        assert_eq!(
            action("on es 50% do disconnect").pre,
            vec![PreCondition::EnergyBelow {
                threshold: Value::Percent(0.5)
            }]
        );
    }

    #[test]
    fn es_flat() {
        assert_eq!(
            action("on es 1000 do disconnect").pre,
            vec![PreCondition::EnergyBelow {
                threshold: Value::Flat(1000)
            }]
        );
    }

    #[test]
    fn mana_percent() {
        assert_eq!(
            action("on mana 50% do disconnect").pre,
            vec![PreCondition::ManaBelow {
                threshold: Value::Percent(0.5)
            }]
        );
    }

    #[test]
    fn mana_flat() {
        assert_eq!(
            action("on mana 1000 do disconnect").pre,
            vec![PreCondition::ManaBelow {
                threshold: Value::Flat(1000)
            }]
        );
    }

    #[test]
    fn after_delay() {
        assert_eq!(
            action("on life 1000 do disconnect every 1000ms after 140ms").windup_time,
            Duration::from_millis(140)
        );
    }

    #[test]
    fn key() {
        assert_eq!(
            action("on key z do disconnect").pre,
            vec![PreCondition::KeyPress { vk: 0x5A }]
        );
        assert_eq!(
            action("on key Z do disconnect").pre,
            vec![PreCondition::KeyPress { vk: 0x5A }]
        );
        assert_eq!(
            action("on key 6 do disconnect").pre,
            vec![PreCondition::KeyPress { vk: 0x36 }]
        );
        assert_eq!(
            action("on key F11 do disconnect").pre,
            vec![PreCondition::KeyPress { vk: 0x7A }]
        );
        assert_eq!(
            action("on key 0x2 do disconnect").pre,
            vec![PreCondition::KeyPress { vk: 0x02 }]
        );
    }

    #[test]
    fn key_synonyms() {
        assert_eq!(
            action("on key Z do disconnect").pre,
            action("on flask Z do disconnect").pre
        );
        assert_eq!(
            action("on key Z do disconnect").pre,
            action("on skill Z do disconnect").pre
        );
    }

    #[test]
    fn parse_self_display() {
        fn parse_self(line: &str) {
            let parsed = action(line);
            let reparsed = action(&parsed.to_string());
            assert_eq!(parsed.pre, reparsed.pre);
            assert_eq!(parsed.post, reparsed.post);
            assert_eq!(parsed.delay, reparsed.delay);
            assert_eq!(parsed.windup_time, reparsed.windup_time);
            assert_eq!(parsed.silent, reparsed.silent);
        }

        parse_self("on life 50% do disconnect");
        parse_self("do disconnect on life 1000");
        parse_self("every 200ms on es 50% do disconnect");
        parse_self("on es 1000 every 200ms do disconnect");
        parse_self("on mana 50% do disconnect every 200ms");
        parse_self("do disconnect every 200ms on mana 1000");
        parse_self("every 200ms on key z do type test after 50ms");
        parse_self("do destroy on key Z every 200ms");
        parse_self("on key A do disable silent");
        parse_self("on key B do enable");
    }

    #[test]
    fn display() {
        assert_eq!(
            action("on key Z do disconnect every 2s").to_string(),
            "on key 0x5A every 2000ms do disconnect"
        );

        assert_eq!(
            action("on key A every 200ms do disconnect after 10ms on key Z every 300ms after 30ms do type test")
                .to_string(),
            "on key 0x41 on key 0x5A every 300ms after 30ms do type test"
        );
    }
}
