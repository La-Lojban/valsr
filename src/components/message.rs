use yew::prelude::*;

use crate::state::GameMode;
use crate::Msg as GameMsg;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://discord.gg/4KhzRzpmVr";
const FORMS_LINK_TEMPLATE_DEL: &str = "https://discord.gg/4KhzRzpmVr";
const DICTIONARY_LINK_TEMPLATE: &str =
    "https://la-lojban.github.io/sutysisku/lojban/index.html#seskari=catni&sisku=";

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub message: String,
    pub is_unknown: bool,
    pub is_winner: bool,
    pub is_guessing: bool,
    pub word: String,
    pub last_guess: String,
    pub game_mode: GameMode,
    pub callback: Callback<GameMsg>,
}

pub struct Message {
    is_emojis_copied: bool,
}

pub enum Msg {
    SetIsEmojisCopied(bool),
}

impl Component for Message {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            is_emojis_copied: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetIsEmojisCopied(is_copied) => {
                self.is_emojis_copied = is_copied;
            }
        }
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        if self.is_emojis_copied {
            ctx.link().send_message(Msg::SetIsEmojisCopied(false));
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <div class="message">
                { &props.message }
                <div class="message-small">{{
                    if props.is_unknown {
                        let last_guess = props.last_guess.to_lowercase();
                        html! {
                            <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_ADD, last_guess)}
                                target="_blank">{ "Suggest a new word?" }
                            </a>
                        }
                    } else if !props.is_winner & !props.is_guessing {
                        let word = props.word.to_lowercase();

                        html! {
                            <>
                                <a class="link" href={format!("{}{}", DICTIONARY_LINK_TEMPLATE, word)}
                                    target="_blank">{ "Dictionary" }
                                </a>
                                {" | "}
                                {
                                    if matches!(props.game_mode, GameMode::DailyWord(_)) {
                                        let callback = props.callback.clone();
                                        let onclick = ctx.link().callback(move |e: MouseEvent| {
                                            e.prevent_default();
                                            callback.emit(GameMsg::ShareEmojis);
                                            Msg::SetIsEmojisCopied(true)
                                        });

                                        html! {
                                            if !self.is_emojis_copied {
                                                <a class="link" href={"javascript:void(0)"} {onclick}>
                                                    {"Copy the emoji?"}
                                                </a>
                                            } else {
                                                <a class="link" {onclick}>
                                                    {"Copied!"}
                                                </a>
                                            }
                                        }
                                    } else {
                                        html! {
                                            <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_DEL, word)}
                                                target="_blank">{ "Suggest removal?" }
                                            </a>
                                        }
                                    }
                                }
                            </>
                        }
                    } else if !props.is_guessing && matches!(props.game_mode, GameMode::DailyWord(_)) {
                        let callback = props.callback.clone();
                        let share_emojis = ctx.link().callback(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(GameMsg::ShareEmojis);
                            Msg::SetIsEmojisCopied(true)
                        });
                        let callback = props.callback.clone();
                        let share_link = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(GameMsg::ShareLink);
                        });

                        html! {
                            <>
                                <a>
                                    {"Share your solution:\u{00a0}"}
                                </a>
                                <a class="link" href={"javascript:void(0)"} onclick={share_emojis}>
                                    {
                                        if !self.is_emojis_copied {
                                            {"Emoji"}
                                        } else {
                                            {"Copied!"}
                                        }
                                    }
                                </a>
                                {" | "}
                                <a class="link" href={"javascript:void(0)"} onclick={share_link}>
                                    {"Link"}
                                </a>
                            </>
                        }
                    } else {
                        html! {}
                    }
                }}
                </div>
            </div>
        }
    }
}
