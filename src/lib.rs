//! Host-independent conversation state machine. No clocks, IO, DOM, or network.
mod dialogue;
#[cfg(target_arch = "wasm32")]
mod wasm;

fn text(id: u16) -> &'static str {
    dialogue::LITERALS
        .binary_search_by_key(&id, |(key, _)| *key)
        .map(|i| dialogue::LITERALS[i].1)
        .expect("known original dialogue ID")
}

/// Turkish casing is significant: I → ı and İ → i.
pub fn turkish_lower(input: &str) -> String {
    input
        .chars()
        .flat_map(|c| match c {
            'I' => 'ı'.to_lowercase(),
            'İ' => 'i'.to_lowercase(),
            _ => c.to_lowercase(),
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    pub delay_ms: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    pub lines: Vec<Line>,
    pub prompt: &'static str,
    pub finished: bool,
}

impl Turn {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            prompt: "",
            finished: false,
        }
    }
    fn say(&mut self, s: impl Into<String>) {
        self.lines.push(Line {
            text: s.into(),
            delay_ms: 220,
        });
    }
    fn original(&mut self, id: u16) {
        self.say(text(id));
    }
    fn originals(&mut self, ids: &[u16]) {
        for &id in ids {
            self.original(id);
        }
    }
    /// Small dependency-free wire format shared with the WASM adapter.
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\"lines\":[");
        for (i, line) in self.lines.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str("{\"text\":");
            quote(&line.text, &mut s);
            s.push_str(&format!(",\"delay_ms\":{}}}", line.delay_ms));
        }
        s.push_str("],\"prompt\":");
        quote(self.prompt, &mut s);
        s.push_str(if self.finished {
            ",\"finished\":true}"
        } else {
            ",\"finished\":false}"
        });
        s
    }
}

fn quote(input: &str, out: &mut String) {
    out.push('"');
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Name,
    Age,
    Milk,
    Vote,
    Height,
    Weight,
    Eyes,
    Moon,
    Origin,
    Hold,
    Nickname,
    Mood,
    Trouble,
    Student,
    Occupation,
    School,
    Price(u16),
    Sport,
    Hometown,
    UserGuess,
    BotGuess,
    Done,
}

/// Deliberately finite, scripted chat, reconstructed from the DOS executable.
/// Randomness is reproducible across native and WASM targets.
pub struct Chat {
    stage: Stage,
    rng: u32,
    name: String,
    nickname: String,
    optional: u8,
    secret: i32,
    previous_guess: Option<i32>,
    user_attempts: u32,
    bot_attempts: u32,
    bot_guess: i32,
    low: i32,
    high: i32,
    cheats: u32,
}

impl Chat {
    pub fn new(seed: u32) -> Self {
        Self {
            stage: Stage::Name,
            rng: if seed == 0 { 0x4b415241 } else { seed },
            name: String::new(),
            nickname: String::new(),
            optional: 0,
            secret: 0,
            previous_guess: None,
            user_attempts: 0,
            bot_attempts: 0,
            bot_guess: 0,
            low: 1,
            high: 100,
            cheats: 0,
        }
    }
    fn random(&mut self, n: u32) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        // Multiply-high avoids modulo bias; no host RNG or floats involved.
        ((x as u64 * n as u64) >> 32) as u32
    }
    fn pick(&mut self, ids: &[u16], turn: &mut Turn) {
        turn.original(ids[self.random(ids.len() as u32) as usize]);
    }
    pub fn start(&self) -> Turn {
        let mut turn = Turn::new();
        turn.originals(&[0x630, 0x640, 0x65a]);
        turn.prompt = self.prompt();
        turn
    }
    pub fn finished(&self) -> bool {
        self.stage == Stage::Done
    }
    fn prompt(&self) -> &'static str {
        match self.stage {
            Stage::Name => "adın",
            Stage::Age => "yaş",
            Stage::Height => "boy (cm)",
            Stage::Weight => "kilo (kg)",
            Stage::Milk
            | Stage::Vote
            | Stage::Eyes
            | Stage::Moon
            | Stage::Hold
            | Stage::Nickname
            | Stage::Mood
            | Stage::Student => "e/h",
            Stage::Origin => "nerden?",
            Stage::Trouble => "anlat",
            Stage::Occupation => "işin",
            Stage::School => "okul",
            Stage::Price(_) => "kaç lira?",
            Stage::Sport => "kaç senelik?",
            Stage::Hometown => "memleket",
            Stage::UserGuess => "1–100",
            Stage::BotGuess => "y: yüksel / d: düş / Enter: buldun",
            Stage::Done => "",
        }
    }
    pub fn submit(&mut self, input: &str) -> Turn {
        let mut turn = Turn::new();
        if self.finished() {
            turn.finished = true;
            return turn;
        }
        let input: String = input
            .chars()
            .filter(|c| !c.is_control())
            .take(256)
            .collect();
        let raw = input.trim();
        let answer = turkish_lower(raw);
        if matches!(answer.as_str(), "/quit" | "/exit" | "/çık") {
            self.goodbye(&mut turn);
        } else {
            self.advance(raw, &answer, &mut turn);
        }
        turn.prompt = self.prompt();
        turn.finished = self.finished();
        turn
    }
    fn advance(&mut self, raw: &str, answer: &str, t: &mut Turn) {
        let yes = matches!(answer, "e" | "evet");
        match self.stage {
            Stage::Name => {
                if raw.is_empty() {
                    t.original(0x65a);
                    return;
                }
                self.name = raw.to_owned();
                self.nickname = raw.chars().take(2).collect::<String>() + "oş";
                if [
                    "am", "göt", "sik", "yar", "piç", "ebe", "bok", "pez", "oros", "çük",
                ]
                .iter()
                .any(|p| answer.starts_with(p))
                    || answer == "liboş"
                {
                    self.pick(&[0x6dc, 0x736, 0x766], t);
                }
                let len = raw.chars().count();
                if len > 7 {
                    t.original(0x79e);
                    self.pick(&[0x7b8, 0x7f2, 0x81e], t);
                    t.say(format!("{len}{}", text(0x858)));
                    t.say(format!("{}...", raw.chars().take(3).collect::<String>()));
                    t.original(0x8ac);
                }
                t.original(0x8e0);
                self.stage = Stage::Age;
            }
            Stage::Age => {
                let Some(n) = number(raw, t) else { return };
                if n > 98.0 {
                    t.originals(&[0xb46, 0x8e0]);
                } else if n < 5.0 {
                    t.original(0x8f8);
                    self.height(t);
                } else if n < 10.0 {
                    t.originals(&[0x932, 0x952]);
                    self.stage = Stage::Milk;
                } else if n < 18.0 {
                    t.original(0x9ac);
                    self.height(t);
                } else if n < 25.0 {
                    t.originals(&[0x9e4, 0x952]);
                    self.stage = Stage::Vote;
                } else {
                    t.original(if n < 40.0 {
                        0xa56
                    } else if n < 60.0 {
                        0xa9a
                    } else {
                        0xaf6
                    });
                    self.height(t);
                }
            }
            Stage::Milk => {
                t.original(if yes { 0x962 } else { 0x998 });
                self.height(t);
            }
            Stage::Vote => {
                t.original(if yes { 0x9fe } else { 0xa1c });
                self.height(t);
            }
            Stage::Height => {
                let Some(n) = number(raw, t) else { return };
                t.original(if n < 100.0 {
                    0xb7e
                } else if n < 150.0 {
                    0xbb0
                } else if n < 170.0 {
                    0xc06
                } else if n < 190.0 {
                    0xc4c
                } else if n < 210.0 {
                    0xc76
                } else {
                    0xc96
                });
                t.original(0xcca);
                self.stage = Stage::Weight;
            }
            Stage::Weight => {
                let Some(n) = number(raw, t) else { return };
                if n < 40.0 {
                    self.pick(&[0xd08, 0xd3c], t);
                } else if n < 60.0 {
                    t.original(0xd6e);
                } else if n < 80.0 {
                    t.original(0xda8);
                } else if n < 100.0 {
                    self.pick(&[0xdf6, 0xe40], t);
                } else {
                    self.pick(&[0xe6c, 0xe94], t);
                }
                self.laugh(t);
                self.next_optional(t);
            }
            Stage::Eyes => {
                t.original(if yes { 0xf62 } else { 0xf76 });
                self.next_optional(t);
            }
            Stage::Moon => {
                t.original(if yes { 0xfd6 } else { 0xff6 });
                self.next_optional(t);
            }
            Stage::Origin => {
                t.original(0x104a);
                self.next_optional(t);
            }
            Stage::Hold => {
                t.original(if yes { 0x1096 } else { 0x10aa });
                self.next_optional(t);
            }
            Stage::Nickname => {
                if yes {
                    t.original(0x112a);
                }
                t.say(format!("{}{}?", text(0x1158), self.nickname));
                t.original(0x1172);
                self.stage = Stage::Mood;
            }
            Stage::Mood => {
                let variant = self.random(3);
                if yes {
                    match variant {
                        0 => t.originals(&[0x1194, 0x11ca]),
                        1 => {
                            t.say(format!("{}{}?", text(0x11f2), self.name));
                            t.original(0x1226);
                        }
                        _ => {
                            t.say(format!("{}{}!", text(0x1252), self.name));
                            t.original(0x127a);
                        }
                    }
                    self.student(t);
                } else if matches!(answer, "h" | "hayır") && variant == 2 {
                    t.originals(&[0x1294, 0x12dc]);
                    self.stage = Stage::Trouble;
                } else {
                    t.original(if variant == 1 { 0x134a } else { 0x137c });
                    self.student(t);
                }
            }
            Stage::Trouble => {
                t.say(format!("{raw}??"));
                t.original(0x130a);
                self.student(t);
            }
            Stage::Student => {
                if yes {
                    self.pick(&[0x13ba, 0x140a], t);
                    self.after_occupation(t);
                } else {
                    t.originals(&[0x1454, 0x14a4]);
                    self.stage = Stage::Occupation;
                }
            }
            Stage::Occupation => self.occupation(answer, t),
            Stage::School => {
                t.say(format!("{raw}??.."));
                t.original(0x2190);
                self.after_occupation(t);
            }
            Stage::Price(id) => {
                t.say(format!("{raw} {}", text(id)));
                self.after_occupation(t);
            }
            Stage::Sport => {
                t.originals(&[0x22be, 0x22ea, 0x2320]);
                self.after_occupation(t);
            }
            Stage::Hometown => {
                match last_vowel(answer) {
                    'e' | 'i' => t.say(format!("nea!? {raw}den adam çıkmaz ki beah!!! hihöhöhö!!")),
                    'a' | 'ı' => t.say(format!("naaaber pis {raw}lı!")),
                    'ö' | 'ü' => t.say(format!("heheheh! {raw}den top çıkarmış diyolar!?!")),
                    _ => t.say(format!("madem {raw}lusun, buralara ne b*k yemeye geldin?! Ayrıca {raw}dan adam falan çıkmaz!")),
                }
                t.say(format!("neyse {}, kusura bakma...", self.name));
                t.say(format!("{},{}", self.name, text(0x16ac)));
                t.originals(&[0x16d0, 0x1706, 0x1712]);
                self.secret = self.random(100) as i32 + 1;
                self.stage = Stage::UserGuess;
            }
            Stage::UserGuess => self.user_guess(raw, t),
            Stage::BotGuess => self.bot_feedback(answer, t),
            Stage::Done => {}
        }
    }
    fn height(&mut self, t: &mut Turn) {
        t.original(0xb5e);
        self.stage = Stage::Height;
    }
    fn student(&mut self, t: &mut Turn) {
        t.originals(&[0x1394, 0x13a2]);
        self.stage = Stage::Student;
    }
    fn next_optional(&mut self, t: &mut Turn) {
        // DOS uses independent INT(RND * 2) gates, not a single selected question.
        while self.optional < 6 {
            let which = self.optional;
            self.optional += 1;
            if self.random(2) == 0 {
                continue;
            }
            match which {
                0 => {
                    t.originals(&[0xeca, 0xef4]);
                    self.laugh(t);
                }
                1 => {
                    t.say(format!("{}!", self.name));
                    t.originals(&[0xf1e, 0x952]);
                    self.stage = Stage::Eyes;
                    return;
                }
                2 => {
                    t.originals(&[0xfa2, 0xfae, 0x952]);
                    self.stage = Stage::Moon;
                    return;
                }
                3 => {
                    t.say(format!("{} {}", self.name, text(0x1034)));
                    self.stage = Stage::Origin;
                    return;
                }
                4 => {
                    t.originals(&[0x106c, 0x1080]);
                    self.stage = Stage::Hold;
                    return;
                }
                _ => {
                    t.say(format!(
                        "{}{}{}{}",
                        self.name,
                        text(0x1100),
                        self.nickname,
                        text(0x1112).trim_start_matches("oş")
                    ));
                    t.original(0x952);
                    self.stage = Stage::Nickname;
                    return;
                }
            }
        }
        t.say(format!("{}{}?", text(0x1158), self.nickname));
        t.original(0x1172);
        self.stage = Stage::Mood;
    }
    fn occupation(&mut self, a: &str, t: &mut Turn) {
        if a.starts_with("öğr") || a.starts_with("ogr") {
            t.original(0x216c);
            self.stage = Stage::School;
            return;
        }
        for (prefix, question, reply) in [("man", 0x21d8, 0x21f6), ("oto", 0x2232, 0x225e)] {
            if a.starts_with(prefix) {
                t.original(question);
                self.stage = Stage::Price(reply);
                return;
            }
        }
        if a.starts_with("spor") {
            t.original(0x22ae);
            self.stage = Stage::Sport;
            return;
        }
        let ids: &[u16] = if a.starts_with("serb") {
            &[0x235a]
        } else if a == "polis" {
            &[0x23aa, 0x23ee]
        } else if a.starts_with("emek") {
            &[0x241e]
        } else if a == "memur" {
            &[0x245a]
        } else if a == "işçi" || a == "isci" {
            &[0x24a4, 0x24f0]
        } else if a.starts_with("bors") {
            &[0x2526]
        } else if a.starts_with("mille") {
            &[0x2576, 0x25ae]
        } else if a.starts_with("pez") {
            &[0x2620]
        } else if a.starts_with("oros") {
            &[0x2664]
        } else {
            &[0x26aa]
        };
        t.originals(ids);
        self.after_occupation(t);
    }
    fn after_occupation(&mut self, t: &mut Turn) {
        t.original(0x14c6);
        self.joke(t);
        self.pick(&[0x14f4, 0x1526, 0x154e], t);
        t.say(format!("{}{}?", text(0x158a), self.name));
        self.stage = Stage::Hometown;
    }
    fn laugh(&mut self, t: &mut Turn) {
        self.pick(&[0x1d54, 0x1d76, 0x1d88, 0x1dc6, 0x1e0c], t);
    }
    fn joke(&mut self, t: &mut Turn) {
        // Five recovered jokes. The sixth, involving sexual abuse of a child,
        // stays in the archival extraction and is not performed by the tribute.
        let jokes: &[&[u16]] = &[
            &[0x1e42, 0x1e66, 0x1e92, 0x1eb0, 0x1ecc],
            &[0x1ee8, 0x1f0c, 0x1f26],
            &[0x1f54, 0x1f72, 0x1fa2],
            &[0x1fd2, 0x2002, 0x201c],
            &[0x209a, 0x20c6, 0x20e2],
        ];
        t.originals(jokes[self.random(jokes.len() as u32) as usize]);
        self.laugh(t);
    }
    fn user_guess(&mut self, raw: &str, t: &mut Turn) {
        let Some(n) = number(raw, t) else { return };
        if n.fract() != 0.0 {
            t.say("Tam sayı gir (1–100).");
            return;
        }
        let n = n as i32;
        self.user_attempts += 1;
        if self.previous_guess == Some(n) {
            t.original(0x172a);
        }
        self.previous_guess = Some(n);
        if !(1..=100).contains(&n) {
            t.original(0x174c);
            return;
        }
        let delta = n - self.secret;
        if delta != 0 {
            t.original(match delta {
                71.. => 0x1774,
                21..=70 => {
                    if self.random(2) == 0 {
                        0x178a
                    } else {
                        0x17a2
                    }
                }
                1..=20 => 0x17bc,
                -20..=-1 => 0x1812,
                -70..=-21 => {
                    if self.random(2) == 0 {
                        0x17ec
                    } else {
                        0x1806
                    }
                }
                _ => 0x17ce,
            });
            return;
        }
        let count = self.user_attempts;
        if count < 3 {
            t.say(format!("{count}{}", text(0x182e)));
        } else if count < 6 {
            t.say(format!("{count}{}", text(0x185e)));
        } else if count < 11 {
            t.say(format!("{count} {}", text(0x189c)));
        } else if count < 20 {
            t.say(format!("{}{count}{}", text(0x18b6), text(0x18cc)));
        } else if count < 30 {
            t.say(format!("{}{count}{}", text(0x18f2), text(0x191a)));
        } else {
            t.say(format!("{count} {}", text(0x1936)));
            t.originals(&[0x1956, 0x1972, 0x199c, 0x19ce]);
        }
        self.laugh(t);
        t.originals(&[0x19e2, 0x1a22, 0x1a72]);
        self.stage = Stage::BotGuess;
        self.next_bot_guess(t);
    }
    fn next_bot_guess(&mut self, t: &mut Turn) {
        self.bot_guess = self.low + self.random((self.high - self.low + 1) as u32) as i32;
        self.bot_attempts += 1;
        t.say(format!("{} ??", self.bot_guess));
    }
    fn bot_feedback(&mut self, answer: &str, t: &mut Turn) {
        if answer.is_empty() {
            t.say(format!("{}{}", self.bot_attempts, text(0x1bc2)));
            self.laugh(t);
            t.original(if self.bot_attempts == self.user_attempts {
                0x1bda
            } else if self.bot_attempts < self.user_attempts {
                0x1bf6
            } else {
                0x1c18
            });
            t.original(0x1c4c);
            self.joke(t);
            self.goodbye(t);
            return;
        }
        if !matches!(answer, "y" | "d") {
            t.say("'y', 'd' ya da bulduysam sadece Enter.");
            return;
        }
        let (low, high) = if answer == "y" {
            (self.bot_guess + 1, self.high)
        } else {
            (self.low, self.bot_guess - 1)
        };
        if low > high {
            self.cheats += 1;
            self.pick(&[0x1ad8, 0x1afc, 0x1b16, 0x1b44, 0x1b78], t);
            if self.cheats >= 4 {
                self.pick(&[0x210c, 0x212a, 0x2132], t);
                self.stage = Stage::Done;
            } else {
                t.say(format!("{} ??", self.bot_guess));
            }
            return;
        }
        self.low = low;
        self.high = high;
        self.next_bot_guess(t);
    }
    fn goodbye(&mut self, t: &mut Turn) {
        let (suffix, id) = match last_vowel(&turkish_lower(&self.name)) {
            'e' | 'i' => ("ciğim", 0x1c7a),
            'a' | 'ı' => ("cığım", 0x1cb0),
            'ö' | 'ü' => ("cüğüm", 0x1cde),
            _ => ("cuğum", 0x1d0c),
        };
        t.say(format!("neyse {}{suffix}...", self.name));
        t.originals(&[id, 0x1d40]);
        self.stage = Stage::Done;
    }
}

fn last_vowel(s: &str) -> char {
    s.chars()
        .rev()
        .find(|c| "aeıioöuü".contains(*c))
        .unwrap_or('a')
}
fn number(raw: &str, t: &mut Turn) -> Option<f64> {
    match raw.replace(',', ".").parse::<f64>() {
        Ok(n) if n.is_finite() && n.abs() <= 1_000_000.0 => Some(n),
        _ => {
            t.say("Sayı gir yavrucum... (örnek: 25)");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_descriptors_are_sorted_and_unique() {
        assert_eq!(dialogue::LITERALS.len(), 247);
        assert!(dialogue::LITERALS.windows(2).all(|p| p[0].0 < p[1].0));
        assert_eq!(text(0x630), "merhaba");
    }
    #[test]
    fn turkish_casing_and_json_escape() {
        assert_eq!(turkish_lower("IŞIK İŞÇİ"), "ışık işçi");
        let mut s = String::new();
        quote("\"\\\n\0İ", &mut s);
        assert_eq!(s, "\"\\\"\\\\\\n\\u0000İ\"");
    }
    #[test]
    fn age_boundaries_and_numeric_retry() {
        for (age, stage) in [
            (4, Stage::Height),
            (5, Stage::Milk),
            (9, Stage::Milk),
            (10, Stage::Height),
            (17, Stage::Height),
            (18, Stage::Vote),
            (24, Stage::Vote),
            (25, Stage::Height),
            (98, Stage::Height),
            (99, Stage::Age),
        ] {
            let mut c = Chat::new(1);
            c.submit("Ali");
            c.submit(&age.to_string());
            assert_eq!(c.stage, stage);
        }
        let mut c = Chat::new(1);
        c.submit("Ali");
        for s in ["", "NaN", "inf", "nope", "1e100"] {
            c.submit(s);
            assert_eq!(c.stage, Stage::Age);
        }
    }
    #[test]
    fn bot_converges_on_every_secret_without_repeating() {
        for seed in 1..=20 {
            for target in 1..=100 {
                let mut c = Chat::new(seed);
                c.stage = Stage::BotGuess;
                c.next_bot_guess(&mut Turn::new());
                let mut seen = Vec::new();
                while c.bot_guess != target {
                    assert!(!seen.contains(&c.bot_guess));
                    seen.push(c.bot_guess);
                    c.submit(if c.bot_guess < target { "y" } else { "d" });
                    assert_eq!(c.stage, Stage::BotGuess);
                    assert!(seen.len() < 100);
                }
                let t = c.submit("");
                assert!(t.finished);
            }
        }
    }
    #[test]
    fn all_seeded_sessions_reach_both_games_and_end() {
        for seed in 1..=100 {
            let mut c = Chat::new(seed);
            let mut turns = 0;
            while !c.finished() {
                let answer = match c.stage {
                    Stage::Name => "Işık",
                    Stage::Age => "25",
                    Stage::Height => "175",
                    Stage::Weight => "70",
                    Stage::Occupation => "polis",
                    Stage::Hometown => "Ankara",
                    Stage::UserGuess => {
                        let n = c.secret.to_string();
                        c.submit(&n);
                        turns += 1;
                        continue;
                    }
                    Stage::BotGuess => "",
                    _ => "e",
                };
                c.submit(answer);
                turns += 1;
                assert!(turns < 40);
            }
            assert_eq!(c.user_attempts, 1);
            assert!(c.bot_attempts > 0);
        }
    }
    #[test]
    fn contradictory_feedback_and_quit_terminate() {
        let mut c = Chat::new(1);
        c.stage = Stage::BotGuess;
        c.low = 1;
        c.high = 1;
        c.bot_guess = 1;
        for _ in 0..4 {
            c.submit("d");
        }
        assert!(c.finished());
        let mut c = Chat::new(2);
        assert!(c.submit("/quit").finished);
        assert!(c.submit("anything").lines.is_empty());
    }
}
