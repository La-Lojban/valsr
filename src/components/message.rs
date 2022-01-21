use yew::prelude::*;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://la-lojban.github.io/sutysisku/lojban/";
const FORMS_LINK_TEMPLATE_DEL: &str = "https://la-lojban.github.io/sutysisku/lojban/";
const DICTIONARY_LINK_TEMPLATE: &str = "https://la-lojban.github.io/sutysisku/lojban/index.html#seskari=catni&sisku=";

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub message: String,
    pub is_unknown: bool,
    pub is_winner: bool,
    pub is_guessing: bool,
    pub word: String,
    pub last_guess: String,
}

#[function_component(Message)]
pub fn message(props: &Props) -> Html {
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
                            <a class="link" href={format!("{}{}&bangu=en&versio=masno", DICTIONARY_LINK_TEMPLATE, word)}
                                target="_blank">{ "Dictionary" }
                            </a>
                            {" | "}
                            <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_DEL, word)}
                                target="_blank">{ "Suggest a removal?" }
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