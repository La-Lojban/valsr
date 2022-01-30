use chrono::Local;
use yew::prelude::*;

use crate::state::{GameMode, Theme, WordList};
use crate::Msg;

const LIVE_CHAT: &str = "https://discord.gg/4KhzRzpmVr";
const CHANGELOG_URL: &str = "https://github.com/La-Lojban/valsr/blob/master/CHANGELOG.md";
const VERSION: &str = "v1.11";

macro_rules! onmousedown {
    ( $cb:ident, $msg:expr ) => {{
        let $cb = $cb.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            $cb.emit($msg);
        })
    }};
}

#[derive(Properties, Clone, PartialEq)]
pub struct HelpModalProps {
    pub callback: Callback<Msg>,
}

#[function_component(HelpModal)]
pub fn help_modal(props: &HelpModalProps) -> Html {
    let callback = props.callback.clone();
    let toggle_help = onmousedown!(callback, Msg::ToggleHelp);

    html! {
        <div class="modal">
            <span onmousedown={toggle_help} class="modal-close">{"✖"}</span>
            <p>{"Guess the hidden "}<i>{"word"}</i>{" in six tries."}</p>
            <p>{"After each guess the letters change color."}</p>
            <div class="row-5 example">
                <div class={classes!("tile", "correct")}>{"B"}</div>
                <div class={classes!("tile", "absent")}>{"A"}</div>
                <div class={classes!("tile", "present")}>{"N"}</div>
                <div class={classes!("tile", "absent")}>{"G"}</div>
                <div class={classes!("tile", "absent")}>{"U"}</div>
            </div>
            <p><span class="present">{"Yellow"}</span>{": The letter is in the word, but in the wrong spot."}</p>
            <p><span class="correct">{"Green"}</span>{": The letter is in the word and in the correct spot."}</p>
            <p><span class="absent">{"Gray"}</span>{": The letter is not in the word."}</p>
            <p>
                {"The word list to be used can be changed from the settings. "}
                {"The common list includes all the gismu. The full list includes all the gismu and all the experimental gismu."}
            </p>
            <p>
                {"The daily word includes all the gismu. The daily word is same for everyone."}
            </p>
            <p>
                {"Join the "}
                <a class="link" href={format!("{}", LIVE_CHAT)}
                            target="_blank">{ "Live Discord chat" }
                        </a>

            </p>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuModalProps {
    pub callback: Callback<Msg>,
    pub word_length: usize,
    pub game_mode: GameMode,
    pub current_word_list: WordList,
    pub allow_profanities: bool,
    pub theme: Theme,

    pub max_streak: usize,
    pub total_played: usize,
    pub total_solved: usize,
}

#[function_component(MenuModal)]
pub fn menu_modal(props: &MenuModalProps) -> Html {
    let callback = props.callback.clone();

    let today = Local::now().naive_local().date();

    let toggle_menu = onmousedown!(callback, Msg::ToggleMenu);
    let change_word_length_5 = onmousedown!(callback, Msg::ChangeWordLength(5));
    let change_word_length_6 = onmousedown!(callback, Msg::ChangeWordLength(6));
    let change_game_mode_classic = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Classic));
    let change_game_mode_relay = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Relay));
    let change_game_mode_daily =
        onmousedown!(callback, Msg::ChangeGameMode(GameMode::DailyWord(today)));
    let change_word_list_full = onmousedown!(callback, Msg::ChangeWordList(WordList::Full));
    let change_word_list_common = onmousedown!(callback, Msg::ChangeWordList(WordList::Common));
    let change_allow_profanities_yes = onmousedown!(callback, Msg::ChangeAllowProfanities(true));
    let change_allow_profanities_no = onmousedown!(callback, Msg::ChangeAllowProfanities(false));
    let change_theme_dark = onmousedown!(callback, Msg::ChangeTheme(Theme::Dark));
    let change_theme_colorblind = onmousedown!(callback, Msg::ChangeTheme(Theme::Colorblind));

    let is_daily_word = matches!(props.game_mode, GameMode::DailyWord(_));

    html! {
        <div class="modal">
            <span onmousedown={toggle_menu} class="modal-close">{"✖"}</span>
            {if !is_daily_word {
                html! {
                    <>
                        // <div>
                        //     <label class="label">{"Sanulien pituus:"}</label>
                        //     <div class="select-container">
                        //         <button class={classes!("select", (props.word_length == 5).then(|| Some("select-active")))}
                        //             onmousedown={change_word_length_5}>
                        //             {"5 merkkiä"}
                        //         </button>
                        //         <button class={classes!("select", (props.word_length == 6).then(|| Some("select-active")))}
                        //             onmousedown={change_word_length_6}>
                        //             {"6 merkkiä"}
                        //         </button>
                        //     </div>
                        // </div>
                        // <div>
                        //     <label class="label">{"Sanulista:"}</label>
                        //     <div class="select-container">
                        //         <button class={classes!("select", (props.current_word_list == WordList::Common).then(|| Some("select-active")))}
                        //             onmousedown={change_word_list_common}>
                        //             {"Suppea"}
                        //         </button>
                        //         <button class={classes!("select", (props.current_word_list == WordList::Full).then(|| Some("select-active")))}
                        //             onmousedown={change_word_list_full}>
                        //             {"Laaja"}
                        //         </button>
                        //     </div>
                        // </div>
                        <div>
                            <label class="label">{"Experimental gismu:"}</label>
                            <div class="select-container">
                                <button class={classes!("select", (!props.allow_profanities).then(|| Some("select-active")))}
                                    onmousedown={change_allow_profanities_no}>
                                    {"No"}
                                </button>
                                <button class={classes!("select", (props.allow_profanities).then(|| Some("select-active")))}
                                    onmousedown={change_allow_profanities_yes}>
                                    {"Yes"}
                                </button>
                            </div>
                        </div>
                    </>
                }
            } else {
                html! {}
            }}
            <div>
                <label class="label">{"Game mode:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.game_mode == GameMode::Classic).then(|| Some("select-active")))}
                        onmousedown={change_game_mode_classic}>
                        {"Basic game"}
                    </button>
                    <button class={classes!("select", (props.game_mode == GameMode::Relay).then(|| Some("select-active")))}
                        onmousedown={change_game_mode_relay}>
                        {"Word chain"}
                    </button>
                    <button class={classes!("select", is_daily_word.then(|| Some("select-active")))}
                        onclick={change_game_mode_daily}>
                        {"Daily word"}
                    </button>
                </div>
            </div>
            <div>
                <label class="label">{"My stats:"}</label>
                <ul>
                    <li class="statistics">{format!("Longest streak: {}", props.max_streak)}</li>
                    <li class="statistics">{format!("Played words: {}", props.total_played)}</li>
                    <li class="statistics">{format!("Solved words: {}", props.total_solved)}</li>
                </ul>
            </div>
            <div>
                <label class="label">{"Theme:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.theme == Theme::Dark).then(|| Some("select-active")))}
                        onmousedown={change_theme_dark}>
                        {"Default"}
                    </button>
                    <button class={classes!("select", (props.theme == Theme::Colorblind).then(|| Some("select-active")))}
                        onmousedown={change_theme_colorblind}>
                        {"Color blind"}
                    </button>
                </div>
            </div>
            <div class="version">
                <a class="version" href={CHANGELOG_URL} target="_blank">{ VERSION }</a>
            </div>
        </div>
    }
}
