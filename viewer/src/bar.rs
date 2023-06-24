use yew::{classes, function_component, html, AttrValue, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct BarProps {
    pub fill: f32,
    pub class: AttrValue,
}

#[function_component(Bar)]
pub fn bar(props: &BarProps) -> Html {
    html! {
        <div class={classes!("bar", props.class.to_string())}>
            <div class="bar-inner" style={format!("width: {}%;", props.fill)}></div>
            <label class="uk-position-z-index">{format!("{:.2}% usage", props.fill)}</label>
        </div>
    }
}
