use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::mem;
use std::rc::Rc;

use chrono::{Local, NaiveDate};
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::{window, Window};

use crate::migration;

use rand::Rng;
use rand::SeedableRng;

use regex::Regex;

const DEFINITIONS: &str = include_str!("../definitions.txt");
const HINTS: &str = include_str!("../hints.txt");
const COMMON_WORDS: &str = include_str!("../common-words.txt");
const DAILY_WORDS: &str = include_str!("../daily-words.txt");
const PROFANITIES: &str = include_str!("../profanities.txt");
const SUCCESS_EMOJIS: [&str; 8] = ["🥳", "🤩", "🤗", "🎉", "😊", "😺", "😎", "👏"];
pub const EMPTY: char = '\u{00a0}'; // &nbsp;
pub const DEFAULT_WORD_LENGTH: usize = 5;
pub const DEFAULT_MAX_GUESSES: usize = 6;
pub const DEFAULT_ALLOW_PROFANITIES: bool = false;
pub const DEFAULT_SHOW_HINTS: bool = true;
pub const DAILY_WORD_LEN: usize = 5;

type WordLists = HashMap<(WordList, usize), HashSet<Vec<char>>>;

fn parse_all_words() -> Rc<WordLists> {
    let mut word_lists: HashMap<(WordList, usize), HashSet<Vec<char>>> = HashMap::with_capacity(3);
    let definition_line_part = Regex::new(r"\t.*").unwrap();
    for line in DEFINITIONS.lines() {
        let word = definition_line_part
            .replace_all(&line, "")
            .to_string()
            .to_uppercase();

        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Full, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.collect());
    }

    for word in COMMON_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Common, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.clone().collect());
        word_lists
            .entry((WordList::Full, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.collect());
    }

    for word in PROFANITIES.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Profanities, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.clone().collect());
        word_lists
            .entry((WordList::Full, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.collect());
    }

    Rc::new(word_lists)
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum WordList {
    Full,
    Common,
    Profanities,
    Daily,
}

impl Default for WordList {
    fn default() -> Self {
        WordList::Common
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Relay,
    DailyWord(NaiveDate),
    Shared,
}

impl Default for GameMode {
    fn default() -> Self {
        let today = Local::now().naive_local().date();
        GameMode::DailyWord(today)
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Colorblind,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum CharacterState {
    Correct,
    Absent,
    Unknown,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TileState {
    Correct,
    Absent,
    Present,
    Unknown,
}

impl fmt::Display for TileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileState::Correct => write!(f, "correct"),
            TileState::Absent => write!(f, "absent"),
            TileState::Present => write!(f, "present"),
            TileState::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum CharacterCount {
    AtLeast(usize),
    Exactly(usize),
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub allow_profanities: bool,
    pub show_hints: bool,
    pub current_game_mode: GameMode,
    pub current_word_list: WordList,
    pub current_word_length: usize,
    pub current_max_guesses: usize,

    pub previous_game: (GameMode, WordList, usize),

    pub theme: Theme,

    pub max_streak: usize,
    pub total_played: usize,
    pub total_solved: usize,

    #[serde(skip)]
    pub game: Game,
    #[serde(skip)]
    pub background_games: HashMap<(GameMode, WordList, usize), Game>,
    #[serde(skip)]
    pub word_lists: Rc<WordLists>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            allow_profanities: DEFAULT_ALLOW_PROFANITIES,
            show_hints: DEFAULT_SHOW_HINTS,
            current_game_mode: GameMode::default(),
            current_word_list: WordList::default(),
            current_word_length: DEFAULT_WORD_LENGTH,
            current_max_guesses: DEFAULT_MAX_GUESSES,

            previous_game: (
                GameMode::default(),
                WordList::default(),
                DEFAULT_WORD_LENGTH,
            ),

            theme: Theme::default(),

            max_streak: 0,
            total_played: 0,
            total_solved: 0,

            game: Game::default(),
            background_games: HashMap::new(),
            word_lists: Rc::new(HashMap::new()),
        }
    }
}

impl State {
    pub fn new() -> Self {
        let word_lists = parse_all_words();

        // Attempt to rehydrate state from localStorage
        let mut initial_state = if let Ok(mut state) = State::rehydrate() {
            if let GameMode::DailyWord(date) = state.current_game_mode {
                let today = Local::today().naive_local();

                if date < today {
                    // Page was refreshed after the day changed - rehydrate the daily word of today
                    state.current_game_mode = GameMode::DailyWord(today);
                }
            }

            let game = Game::new_or_rehydrate(
                state.current_game_mode,
                state.current_word_list,
                state.current_word_length,
                state.allow_profanities,
                state.show_hints,
                word_lists.clone(),
            );

            state.game = game;
            state.word_lists = word_lists;
            state.game.clear_message();

            state
        } else {
            // Otherwise either create everything from scratch or recover some data from legacy storage state
            let game = Game::new(
                GameMode::Relay,
                WordList::Common,
                DEFAULT_WORD_LENGTH,
                DEFAULT_ALLOW_PROFANITIES,
                DEFAULT_SHOW_HINTS,
                word_lists.clone(),
            );

            let mut state = Self {
                game,
                word_lists,
                ..Self::default()
            };

            // Try to migrate old settings and stats from localStorage to current format
            // TODO: Doesn't do anything if the old state isn't present, but get rid of this at some point
            let _res = migration::migrate_settings_and_stats(&mut state);
            state.switch_active_game();

            // Try to migrate old game streak from localStorage to current format, if the game mode is not daily
            // TODO: Doesn't do anything if the old state isn't present, but get rid of this at some point
            let _res = migration::migrate_game(&mut state.game);

            let _res = state.persist();
            let _res = state.game.persist();
            state.game.clear_message();

            state
        };

        if let Some(game) = initial_state.restore_shared_game() {
            initial_state.game = game;
        }

        initial_state
    }

    fn restore_shared_game(&self) -> Option<Game> {
        let window: Window = window().expect("window not available");
        let qs = window.location().search().ok()?;
        if qs.is_empty() {
            return None;
        }

        // Skip the leading "?"
        for param in qs.chars().skip(1).collect::<String>().split("&") {
            let mut parts = param.split("=");

            let key = parts.next()?;
            let value = parts.next()?;

            if key == "game" && !value.is_empty() {
                let game_str = window
                    .atob(value)
                    .ok()?;

                return Game::from_shared_link(&game_str, self.word_lists.clone());
            }
        }

        return None;
    }

    pub fn submit_guess(&mut self) -> bool {
        self.game.submit_guess();
        if !self.game.is_guessing {
            self.update_game_statistics(self.game.is_winner, self.game.streak);
        }

        true
    }

    pub fn change_word_length(&mut self, new_length: usize) {
        if self.current_word_length == new_length {
            return;
        }

        self.current_word_length = new_length;
        self.switch_active_game();

        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_game_mode(&mut self, new_mode: GameMode) {
        self.game.clear_message();
        if self.current_game_mode == new_mode {
            return;
        }

        if matches!(self.current_game_mode, GameMode::DailyWord(_)) {
            self.current_word_list = self.previous_game.1;
            self.current_word_length = self.previous_game.2;
        }

        if matches!(new_mode, GameMode::DailyWord(_)) {
            self.current_word_list = WordList::Daily;
            self.current_word_length = DAILY_WORD_LEN;
        } else if self.current_word_list == WordList::Daily {
            // Prevent getting stuck in non-daily word gamemode with
            // daily list somehow, for instance by having a daily game as
            // the previous game in state
            self.current_word_list = WordList::default();
        }
        self.game.clear_message();

        self.current_game_mode = new_mode;
        self.switch_active_game();
        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_word_list(&mut self, new_list: WordList) {
        if self.current_word_list == new_list {
            return;
        }

        self.current_word_list = new_list;
        self.switch_active_game();

        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_previous_game_mode(&mut self) {
        let (game_mode, word_list, word_length) = self.previous_game;

        if matches!(game_mode, GameMode::DailyWord(_))
            && matches!(self.current_game_mode, GameMode::DailyWord(_))
        {
            // Force the user to reset to the base game
            self.current_game_mode = GameMode::default();
            self.current_word_list = WordList::default();
            self.current_word_length = DEFAULT_WORD_LENGTH;
        } else {
            self.current_game_mode = game_mode;
            self.current_word_list = word_list;
            self.current_word_length = word_length;
        }

        self.switch_active_game();

        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_allow_profanities(&mut self, is_allowed: bool) {
        self.allow_profanities = is_allowed;
        self.game.allow_profanities = self.allow_profanities;
        self.background_games.values_mut().for_each(|game| {
            game.allow_profanities = self.allow_profanities;
        });
        let _result = self.persist();
    }

    pub fn change_show_hints(&mut self, is_allowed: bool) {
        self.show_hints = is_allowed;
        self.game.show_hints = self.show_hints;
        self.game.clear_message();
        self.background_games.values_mut().for_each(|game| {
            game.show_hints = self.show_hints;
            game.clear_message();
        });
        let _result = self.persist();
    }

    pub fn change_theme(&mut self, theme: Theme) -> bool {
        self.theme = theme;
        let _result = self.persist();
        true
    }

    fn switch_active_game(&mut self) -> bool {
        let next_game = (
            self.current_game_mode,
            self.current_word_list,
            self.current_word_length,
        );

        let previous_game = (
            self.game.game_mode,
            self.game.word_list,
            self.game.word_length,
        );

        if next_game.0 == previous_game.0
            && next_game.1 == previous_game.1
            && next_game.2 == previous_game.2
        {
            return false;
        }

        self.previous_game = previous_game;

        // Restore a suspended game or create a new one
        let mut game = self.background_games.remove(&next_game).unwrap_or_else(|| {
            Game::new_or_rehydrate(
                next_game.0,
                next_game.1,
                next_game.2,
                self.allow_profanities,
                self.show_hints,
                self.word_lists.clone(),
            )
        });

        // For playing the animation populate previous_guesses
        if previous_game.2 <= next_game.2 {
            game.previous_guesses = self.game.guesses.clone();
        } else {
            game.previous_guesses = self
                .game
                .guesses
                .iter()
                .cloned()
                .map(|guess| guess.into_iter().take(game.word_length).collect())
                .collect();
        }

        if self.game.current_guess < game.max_guesses - 1 {
            game.previous_guesses.truncate(self.game.current_guess);
        }
        game.is_reset = true;

        game.clear_message();
        self.background_games
            .insert(previous_game, mem::replace(&mut self.game, game));

        true
    }

    fn update_game_statistics(&mut self, is_winner: bool, streak: usize) {
        self.total_played += 1;

        if is_winner {
            self.total_solved += 1;

            if streak > self.max_streak {
                self.max_streak = streak;
            }
        }
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn share_emojis(&self) -> String {
        self.game.share_emojis(self.theme)
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn share_link(&self) -> Option<String> {
        self.game.share_link()
    }

    fn persist(&self) -> Result<(), StorageError> {
        LocalStorage::set("settings", self)
    }

    fn rehydrate() -> Result<Self, StorageError> {
        let mut state: Self = LocalStorage::get("settings")?;
        state.word_lists = parse_all_words();
        Ok(state)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub game_mode: GameMode,
    pub word_list: WordList,
    pub word_length: usize,
    pub max_guesses: usize,

    pub word: Vec<char>,
    pub guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,
    pub streak: usize,

    pub is_guessing: bool,
    pub is_winner: bool,
    pub is_unknown: bool,
    pub is_reset: bool,
    pub message: String,

    #[serde(skip)]
    pub is_hidden: bool,
    #[serde(skip)]
    pub allow_profanities: bool,
    #[serde(skip)]
    pub show_hints: bool,
    #[serde(skip)]
    pub word_lists: Rc<WordLists>,
    #[serde(skip)]
    pub known_states: Vec<HashMap<(char, usize), CharacterState>>,
    #[serde(skip)]
    pub discovered_counts: Vec<HashMap<char, CharacterCount>>,
    #[serde(skip)]
    pub previous_guesses: Vec<Vec<(char, TileState)>>,
}

impl Default for Game {
    fn default() -> Self {
        Game::new(
            GameMode::default(),
            WordList::default(),
            DEFAULT_WORD_LENGTH,
            DEFAULT_ALLOW_PROFANITIES,
            DEFAULT_SHOW_HINTS,
            Rc::new(HashMap::new()),
        )
    }
}

impl Game {
    pub fn new(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        show_hints: bool,
        word_lists: Rc<WordLists>,
    ) -> Self {
        let max_guesses = DEFAULT_MAX_GUESSES;

        let guesses = std::iter::repeat(Vec::with_capacity(word_length))
            .take(max_guesses)
            .collect::<Vec<_>>();

        let known_states = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let discovered_counts = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let word = if word_lists.is_empty() {
            // Default initialization runs into this
            vec!['X'; word_length]
        } else {
            Game::get_word(
                game_mode,
                word_list,
                word_length,
                allow_profanities,
                &word_lists,
            )
        };

        Self {
            game_mode,
            word_list,
            word_lists,
            word_length,
            max_guesses,
            word,
            allow_profanities,
            show_hints,
            is_guessing: true,
            is_winner: false,
            is_unknown: false,
            is_reset: false,
            is_hidden: false,
            message: EMPTY.to_string(),
            known_states,
            discovered_counts,
            guesses,
            previous_guesses: Vec::new(),
            current_guess: 0,
            streak: 0,
        }
    }

    pub fn from_shared_link(game_str: &str, word_lists: Rc<WordLists>) -> Option<Self> {
        let max_guesses = DEFAULT_MAX_GUESSES;

        let mut parts = game_str.split("|");
        let word = parts.next()?.chars().collect::<Vec<_>>();
        let word_length = word.len();

        let guesses_str = parts.next()?;

        let mut guesses = guesses_str
            .chars()
            .map(|c| (c, TileState::Unknown))
            .collect::<Vec<_>>()
            .chunks(5)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>();

        let current_guess = guesses.len();

        guesses.resize(max_guesses, Vec::with_capacity(word_length));

        let known_states = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let discovered_counts = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let mut game = Self {
            game_mode: GameMode::Shared,
            word_list: WordList::Full,
            word_lists,
            word_length,
            max_guesses,
            word,
            allow_profanities: true,
            show_hints: true,
            is_guessing: false,
            is_winner: false,
            is_unknown: false,
            is_reset: false,
            is_hidden: true,
            message: EMPTY.to_string(),
            known_states,
            discovered_counts,
            guesses,
            previous_guesses: Vec::new(),
            current_guess,
            streak: 0,
        };

        game.recalculate();

        return Some(game);
    }

    fn new_or_rehydrate(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        show_hints: bool,
        word_lists: Rc<WordLists>,
    ) -> Self {
        if let Ok(game) = Game::rehydrate(
            game_mode,
            word_list,
            word_length,
            allow_profanities,
            show_hints,
            word_lists.clone(),
        ) {
            game
        } else {
            Game::new(
                game_mode,
                word_list,
                word_length,
                allow_profanities,
                show_hints,
                word_lists,
            )
        }
    }

    fn get_word(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: &Rc<WordLists>,
    ) -> Vec<char> {
        if let GameMode::DailyWord(date) = game_mode {
            Game::get_daily_word(date)
        } else {
            Game::get_random_word(word_list, word_length, allow_profanities, word_lists)
        }
    }

    fn get_random_word(
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: &Rc<WordLists>,
    ) -> Vec<char> {
        let mut words = word_lists
            .get(&(word_list, word_length))
            .unwrap()
            .iter()
            .collect::<Vec<_>>();

        if !allow_profanities {
            if let Some(profanities) = word_lists.get(&(WordList::Profanities, word_length)) {
                words.retain(|word| !profanities.contains(*word));
            }
        }

        let chosen = words.choose(&mut rand::thread_rng()).unwrap();
        (*chosen).clone()
    }
    pub fn get_daily_word_index(_date: NaiveDate) -> usize {
        let epoch = NaiveDate::from_ymd(2022, 1, 07); // Epoch of the daily word mode, index 0
        let index = Local::now()
            .naive_local()
            .date()
            .signed_duration_since(epoch)
            .num_days() as u64;
        let no_of_daily_words: u64 = DAILY_WORDS.lines().count() as u64;
        let rng = rand::rngs::StdRng::seed_from_u64(index).gen_range(0..no_of_daily_words) as usize;
        rng
    }

    fn get_daily_word(date: NaiveDate) -> Vec<char> {
        DAILY_WORDS
            .lines()
            .nth(Game::get_daily_word_index(date))
            .unwrap()
            .chars()
            .collect()
    }

    pub fn next_word(&mut self) -> bool {
        let next_word = Game::get_word(
            self.game_mode,
            self.word_list,
            self.word_length,
            self.allow_profanities,
            &self.word_lists,
        );

        let previous_word = mem::replace(&mut self.word, next_word);

        if previous_word.len() <= self.word_length {
            self.previous_guesses = mem::take(&mut self.guesses);
            if self.game_mode == GameMode::Relay && self.is_winner {
                self.previous_guesses.truncate(self.current_guess);
            } else {
                self.previous_guesses.truncate(self.current_guess + 1);
            }
        } else {
            let previous_guesses = mem::take(&mut self.guesses);
            self.previous_guesses = previous_guesses
                .into_iter()
                .map(|guess| guess.into_iter().take(self.word_length).collect())
                .collect();
            self.previous_guesses.truncate(self.current_guess);
        }

        self.guesses = Vec::with_capacity(self.max_guesses);

        self.known_states = std::iter::repeat(HashMap::new())
            .take(DEFAULT_MAX_GUESSES)
            .collect::<Vec<_>>();
        self.discovered_counts = std::iter::repeat(HashMap::new())
            .take(DEFAULT_MAX_GUESSES)
            .collect::<Vec<_>>();

        if previous_word.len() == self.word_length
            && self.is_winner
            && self.game_mode == GameMode::Relay
        {
            let empty_guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
                .take(self.max_guesses - 1)
                .collect::<Vec<_>>();

            self.guesses.push(
                previous_word
                    .iter()
                    .map(|c| (*c, TileState::Unknown))
                    .collect(),
            );
            self.guesses.extend(empty_guesses);

            self.current_guess = 0;
            self.calculate_current_guess();
            self.current_guess = 1;
        } else {
            self.guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
                .take(self.max_guesses)
                .collect::<Vec<_>>();
            self.current_guess = 0;
        }

        self.is_guessing = true;
        self.is_winner = false;
        self.is_reset = true;
        self.clear_message();
        let _result = self.persist();

        true
    }

    pub fn keyboard_tilestate(&self, key: &char) -> TileState {
        let is_correct = self.known_states[self.current_guess]
            .iter()
            .any(|((c, _index), state)| c == key && state == &CharacterState::Correct);
        if is_correct {
            return TileState::Correct;
        }

        match self.discovered_counts[self.current_guess].get(key) {
            Some(CharacterCount::AtLeast(count)) => {
                if *count == 0 {
                    return TileState::Unknown;
                }
                TileState::Present
            }
            Some(CharacterCount::Exactly(count)) => {
                if *count == 0 {
                    return TileState::Absent;
                }
                TileState::Present
            }
            None => TileState::Unknown,
        }
    }

    fn current_guess_state(&mut self, character: char, index: usize) -> TileState {
        match self.known_states[self.current_guess].get(&(character, index)) {
            Some(CharacterState::Correct) => TileState::Correct,
            Some(CharacterState::Absent) => TileState::Absent,
            _ => {
                match self.discovered_counts[self.current_guess].get(&character) {
                    Some(CharacterCount::Exactly(count)) => {
                        // We may know the exact count, but not the exact index of any characters..
                        if *count == 0 {
                            return TileState::Absent;
                        }

                        let is_every_correct_found = self.known_states[self.current_guess]
                            .iter()
                            .filter(|((c, _i), state)| {
                                c == &character && *state == &CharacterState::Correct
                            })
                            .count()
                            == *count;

                        if !is_every_correct_found {
                            return TileState::Present;
                        }

                        TileState::Absent
                    }
                    Some(CharacterCount::AtLeast(_)) => TileState::Present,
                    None => TileState::Unknown,
                }
            }
        }
    }

    fn reveal_row_tiles(&mut self, row: usize) {
        if let Some(guess) = self.guesses.get_mut(row) {
            let mut revealed_count_on_row: HashMap<char, usize> =
                HashMap::with_capacity(self.word_length);

            for (index, (character, _)) in guess.iter().enumerate() {
                if let Some(CharacterState::Correct) =
                    self.known_states[row].get(&(*character, index))
                {
                    revealed_count_on_row
                        .entry(*character)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }

            for (index, (character, tile_state)) in guess.iter_mut().enumerate() {
                match self.known_states[row].get(&(*character, index)) {
                    Some(CharacterState::Correct) => {
                        *tile_state = TileState::Correct;
                    }
                    Some(CharacterState::Absent) => {
                        let revealed = revealed_count_on_row
                            .entry(*character)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);

                        let discovered_count = self.discovered_counts[row]
                            .get(character)
                            .unwrap_or(&CharacterCount::AtLeast(0));

                        match discovered_count {
                            CharacterCount::AtLeast(count) | CharacterCount::Exactly(count) => {
                                if *revealed <= *count {
                                    *tile_state = TileState::Present;
                                } else {
                                    *tile_state = TileState::Absent;
                                }
                            }
                        }
                    }
                    _ => {
                        *tile_state = TileState::Unknown;
                    }
                }
            }
        }
    }

    pub fn calculate_current_guess(&mut self) {
        for (index, (character, _)) in self.guesses[self.current_guess].iter().enumerate() {
            let known = self.known_states[self.current_guess]
                .entry((*character, index))
                .or_insert(CharacterState::Unknown);

            if self.word[index] == *character {
                *known = CharacterState::Correct;
            } else {
                *known = CharacterState::Absent;

                let discovered_count = self.discovered_counts[self.current_guess]
                    .entry(*character)
                    .or_insert(CharacterCount::AtLeast(0));

                // At most the same amount of characters are highlighted as there are in the word
                let count_in_word = self.word.iter().filter(|c| *c == character).count();
                if count_in_word == 0 {
                    *discovered_count = CharacterCount::Exactly(0);
                    continue;
                }

                let count_in_guess = self.guesses[self.current_guess]
                    .iter()
                    .filter(|(c, _)| c == character)
                    .count();

                match discovered_count {
                    CharacterCount::AtLeast(count) => {
                        if count_in_guess > count_in_word {
                            if count_in_word >= *count {
                                // The guess had more copies of the character than the word,
                                // the exact count is revealed
                                *discovered_count = CharacterCount::Exactly(count_in_word);
                            }
                        } else if count_in_guess == count_in_word || count_in_guess > *count {
                            // One of:
                            // 1) The count had the exact count but that isn't revealed yet
                            // 2) Found more than before, but the exact count is still unknown
                            *discovered_count = CharacterCount::AtLeast(count_in_guess);
                        }
                    }
                    // Exact count should never change
                    CharacterCount::Exactly(_) => {}
                }
            }
        }

        // Copy the previous knowledge to the next round
        if self.current_guess < self.max_guesses - 1 {
            let next = self.current_guess + 1;
            self.known_states[next] = self.known_states[self.current_guess].clone();
            self.discovered_counts[next] = self.discovered_counts[self.current_guess].clone();
        }

        self.reveal_row_tiles(self.current_guess);
    }

    pub fn push_character(&mut self, character: char) -> bool {
        if !self.is_guessing || self.guesses[self.current_guess].len() >= self.word_length {
            return false;
        }

        self.clear_message();

        let tile_state =
            self.current_guess_state(character, self.guesses[self.current_guess].len());
        self.guesses[self.current_guess].push((character, tile_state));
        true
    }

    pub fn pop_character(&mut self) -> bool {
        if !self.is_guessing || self.guesses[self.current_guess].is_empty() {
            return false;
        }

        self.clear_message();
        self.guesses[self.current_guess].pop();

        true
    }

    fn is_guess_allowed(&self) -> bool {
        self.is_guessing && self.guesses[self.current_guess].len() == self.word_length
    }

    fn is_guess_real_word(&self) -> bool {
        match self.word_lists.get(&(WordList::Full, self.word_length)) {
            Some(list) => {
                let word: &Vec<char> = &self.guesses[self.current_guess]
                    .iter()
                    .map(|(c, _)| *c)
                    .collect();

                list.contains(word)
            }
            None => false,
        }
    }

    fn is_correct_word(&self) -> bool {
        self.guesses[self.current_guess]
            .iter()
            .map(|(c, _)| *c)
            .collect::<Vec<char>>()
            == self.word
    }

    fn is_game_ended(&self) -> bool {
        self.is_winner || self.current_guess == self.max_guesses - 1
    }

    fn get_hint(&mut self, word: &std::string::String) -> std::string::String {
        let definition = self.get_word_hint(word);
        let formatted = format!("{}", definition);
        formatted
    }

    fn clear_message(&mut self) {
        self.is_unknown = false;

        let word = &self.word.iter().collect::<String>();
        if self.show_hints == true {
            self.message = self.get_hint(word);
        } else {
            self.message = EMPTY.to_string();
        }
    }

    fn get_word_hint(&mut self, word: &str) -> String {
        let mut word_with_suffix = word.clone().to_owned();
        word_with_suffix.push_str(&"\t");

        let line = HINTS
            .lines()
            .filter(|x| {
                if x.starts_with(&word_with_suffix) {
                    true
                } else {
                    false
                }
            })
            .nth(0)
            .unwrap_or("");
        let re = Regex::new(r"[$_{}]").unwrap();
        let re2 = Regex::new(r"^.*\t").unwrap();
        let mut result = re.replace_all(line, "").to_string();
        result = re2.replace_all(&result, "").to_string();
        result
    }
    fn get_word_definition(&mut self, word: &str) -> String {
        let mut word_with_suffix = word.clone().to_owned();
        word_with_suffix.push_str(&"\t");

        let line = DEFINITIONS
            .lines()
            .filter(|x| {
                if x.starts_with(&word_with_suffix) {
                    true
                } else {
                    false
                }
            })
            .nth(0)
            .unwrap_or("");
        let re = Regex::new(r"[$_{}]").unwrap();
        let re2 = Regex::new(r"^.*\t").unwrap();
        let mut result = re.replace_all(line, "").to_string();
        result = re2.replace_all(&result, "").to_string();
        result
    }

    fn set_game_end_message(&mut self) {
        let stringified_word = &self.word.iter().collect::<String>();
        let def = self.get_word_definition(stringified_word);
        if self.is_winner {
            if let GameMode::DailyWord(_) = self.game_mode {
                self.message = format!(
                    "You found the daily word {}! {} - {}",
                    SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap(),
                    stringified_word,
                    def,
                );
            } else {
                self.message = format!(
                    "You found the word {}! {} - {}",
                    SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap(),
                    stringified_word,
                    def,
                );
            }
        } else {
            self.message = format!("The word was \"{}\" - {}", stringified_word, def);
        }
    }

    fn submit_guess(&mut self) {
        if !self.is_guess_allowed() {
            self.message = "Too few letters!".to_owned();
            return;
        }
        if !self.is_guess_real_word() {
            self.is_unknown = true;
            self.message = "Not in the word list.".to_owned();
            return;
        }

        self.is_reset = false;
        self.clear_message();

        self.is_winner = self.is_correct_word();
        self.calculate_current_guess();
        if self.is_game_ended() {
            self.is_guessing = false;

            if let GameMode::DailyWord(_) = self.game_mode {
                // Do nothing?
            } else if self.is_winner {
                self.streak += 1;
            } else {
                self.streak = 0;
            }

            self.set_game_end_message();
        } else {
            self.current_guess += 1;
        }
        let _result = self.persist();
    }

    #[cfg(web_sys_unstable_apis)]
    fn share_emojis(&self, theme: Theme) -> String {
        let mut message = String::new();

        if let GameMode::DailyWord(date) = self.game_mode {
            let index = Game::get_daily_word_index(date) + 1;
            let guess_count = if self.is_winner {
                format!("{}", self.current_guess + 1)
            } else {
                "X".to_owned()
            };

            message += &format!(
                "la .valsr. zo'u #{} {}/{}",
                index, guess_count, self.max_guesses
            );
            message += "\n\n";

            for guess in self.guesses.iter() {
                if guess.is_empty() {
                    continue;
                }
                let guess_string = guess
                    .iter()
                    .map(|(_, state)| match state {
                        TileState::Correct => match theme {
                            Theme::Colorblind => "🟧",
                            _ => "🟩",
                        },
                        TileState::Present => match theme {
                            Theme::Colorblind => "🟦",
                            _ => "🟨",
                        },
                        TileState::Absent => "⬛",
                        TileState::Unknown => "⬜",
                    })
                    .collect::<String>();

                message += &guess_string;
                message += "\n";
            }
        }

        message
    }

    #[cfg(web_sys_unstable_apis)]
    fn share_link(&self) -> Option<String> {
        let game_str = format!(
            "{}|{}",
            self.word.iter().collect::<String>(),
            self.guesses
                .iter()
                .flat_map(|guess| guess.iter().map(|(c, _)| c))
                .collect::<String>(),
        );
        let window: Window = window().expect("window not available");
        let share_str = window.btoa(&game_str).ok()?;
        let base_url = window.location().origin().ok()?;

        Some(format!("{}/?game={}", base_url, share_str))
    }

    fn recalculate(&mut self) {
        self.known_states = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        self.discovered_counts = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        let current_guess = self.current_guess;
        // Rerrun the game to repuplate known_states and discovered_counts
        for guess_index in 0..self.current_guess {
            self.current_guess = guess_index;
            self.calculate_current_guess();
        }

        // Restore the current guess
        self.current_guess = current_guess;

        // If the game is ended also recalculate the current guess
        if !self.is_guessing {
            self.calculate_current_guess();
        }
    }

    pub fn persist(&self) -> Result<(), StorageError> {
        let game_key = &format!(
            "game|{}|{}|{}",
            serde_json::to_string(&self.game_mode).unwrap(),
            serde_json::to_string(&self.word_list).unwrap(),
            self.word_length
        );

        LocalStorage::set(game_key, self)
    }

    fn rehydrate(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        show_hints: bool,
        word_lists: Rc<WordLists>,
    ) -> Result<Game, StorageError> {
        let game_key = &format!(
            "game|{}|{}|{}",
            serde_json::to_string(&game_mode).unwrap(),
            serde_json::to_string(&word_list).unwrap(),
            word_length
        );

        let mut game: Game = LocalStorage::get(game_key)?;
        game.allow_profanities = allow_profanities;
        game.show_hints = show_hints;
        game.word_lists = word_lists;

        game.recalculate();

        Ok(game)
    }
}
