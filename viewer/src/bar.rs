use yew::{classes, function_component, html, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct BarProps {
    pub fill: f32,
    pub class: String,
}

#[function_component(Bar)]
pub fn bar(props: &BarProps) -> Html {
    html! {
        <div class={classes!("bar", props.class.clone())}>
            <div class="bar-inner" style={format!("width: {}%;", props.fill)}></div>
            <label class="uk-position-z-index">{format!("{:.2}% usage", props.fill)}</label>
        </div>
    }
}
