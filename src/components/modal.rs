use yew::prelude::*;

use crate::state::{GameMode, WordList, Theme};
use crate::Msg;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const CHANGELOG_URL: &str = "https://github.com/La-Lojban/sanuli/blob/master/CHANGELOG.md";
const VERSION: &str = "v1.7";

macro_rules! onmousedown {
    ( $cb:ident, $msg:expr ) => {
        {
            let $cb = $cb.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                $cb.emit($msg);
            })
        }
    };
}

#[derive(Properties, Clone, PartialEq)]
pub struct HelpModalProps {
    pub callback: Callback<Msg>
}

#[function_component(HelpModal)]
pub fn help_modal(props: &HelpModalProps) -> Html {
    let callback = props.callback.clone();
    let toggle_help = onmousedown!(callback, Msg::ToggleHelp);

    html! {
        <div class="modal">
            <span onmousedown={toggle_help} class="modal-close">{"✖"}</span>
            <p>{"Guess the hidden "}<i>{"word"}</i>{" with six tries."}</p>
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
                {"The word list to be used can be changed from the settings."}
                {"The common list includes the most common gismu. The full list includes all the gismu and all the lujvo that has 6 characters."}
            </p>
            <p>
                {"The daily word includes all the gismu. The daily word is same for everyone."}
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

    let toggle_menu = onmousedown!(callback, Msg::ToggleMenu);
    let change_word_length_5 = onmousedown!(callback, Msg::ChangeWordLength(5));
    let change_word_length_6 = onmousedown!(callback, Msg::ChangeWordLength(6));
    let change_game_mode_classic = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Classic));
    let change_game_mode_relay = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Relay));
    let change_game_mode_daily = onmousedown!(callback, Msg::ChangeGameMode(GameMode::DailyWord));
    let change_word_list_full = onmousedown!(callback, Msg::ChangeWordList(WordList::Full));
    let change_word_list_common = onmousedown!(callback, Msg::ChangeWordList(WordList::Common));
    let change_allow_profanities_yes = onmousedown!(callback, Msg::ChangeAllowProfanities(true));
    let change_allow_profanities_no = onmousedown!(callback, Msg::ChangeAllowProfanities(false));
    let change_theme_dark = onmousedown!(callback, Msg::ChangeTheme(Theme::Dark));
    let change_theme_colorblind = onmousedown!(callback, Msg::ChangeTheme(Theme::Colorblind));

    html! {
        <div class="modal">
            <span onmousedown={toggle_menu} class="modal-close">{"✖"}</span>
            <div>
                <label class="label">{"Word length:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.word_length == 5).then(|| Some("select-active")))}
                        onmousedown={change_word_length_5}>
                        {"5 characters"}
                    </button>
                    <button class={classes!("select", (props.word_length == 6).then(|| Some("select-active")))}
                        onmousedown={change_word_length_6}>
                        {"6 characters"}
                    </button>
                </div>
            </div>
            <div>
                <label class="label">{"The word list:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.current_word_list == WordList::Common).then(|| Some("select-active")))}
                        onmousedown={change_word_list_common}>
                        {"Common"}
                    </button>
                    <button class={classes!("select", (props.current_word_list == WordList::Full).then(|| Some("select-active")))}
                        onmousedown={change_word_list_full}>
                        {"Full"}
                    </button>
                </div>
            </div>
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
                    <button class={classes!("select", (props.game_mode == GameMode::DailyWord).then(|| Some("select-active")))}
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
