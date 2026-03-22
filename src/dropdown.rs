use yew::prelude::*;

/// A Bulma-styled dropdown that auto-sizes its trigger button to the width and
/// height of the largest option (including placeholder) without manual measurement.
///
/// Technique: `display:inline-grid` on the trigger wrapper stacks two invisible
/// sizer spans and the visible button in the same grid cell.  The cell grows to
/// the max dimensions of all three children:
///   • Width sizer  — `white-space:nowrap` + longest single line → sets min width.
///   • Height sizer — tallest option rendered via `multiline()` (uses `<br/>`) →
///     sets min height; `<br/>` forces line breaks even with `white-space:nowrap`.
///
/// The dropdown menu is capped at 60 vh so it never runs off the bottom of the
/// screen and gets a scrollbar instead.
/// Render a string that may contain `\n` as a sequence of text nodes separated
/// by `<br/>` elements.  Single-line strings render as plain text with no
/// extra DOM nodes.
fn multiline(text: &str) -> Html {
    let mut parts = text.split('\n');
    let first = parts.next().unwrap_or("");
    let mut out = html! { {first} };
    for line in parts {
        out = html! { <>{out}<br/>{line}</> };
    }
    out
}

#[derive(Clone, PartialEq, Properties)]
pub struct DropdownSelectProps {
    /// `(key, label)` pairs. Keys are opaque strings used to identify a selection.
    pub options: Vec<(String, String)>,
    /// The key of the currently selected option, or `None` if nothing is selected.
    pub selected: Option<String>,
    /// Text shown in the button when nothing is selected.
    pub placeholder: String,
    /// Emits the key of the option the user clicked.
    pub on_select: Callback<String>,
}

#[component]
pub fn DropdownSelect(props: &DropdownSelectProps) -> Html {
    let open = use_state(|| false);

    let current_label_str = props
        .selected
        .as_ref()
        .and_then(|k| props.options.iter().find(|(key, _)| key == k))
        .map(|(_, label)| label.as_str())
        .unwrap_or(props.placeholder.as_str());

    // Labels may contain '\n' to produce multi-line bullet-point options.
    // Two sizers are needed: one enforces the correct width, one the correct height.

    // Width sizer: longest *single line* across all labels (white-space:nowrap).
    let sizer_width = props
        .options
        .iter()
        .flat_map(|(_, label)| label.split('\n'))
        .chain(props.placeholder.split('\n'))
        .max_by_key(|s| s.len())
        .unwrap_or("");

    // Height sizer: the label with the most lines, rendered via multiline().
    // <br/> elements still force line breaks even with white-space:nowrap, so
    // the grid cell grows to the tallest option's height.
    let sizer_tallest = props
        .options
        .iter()
        .map(|(_, label)| label.as_str())
        .chain(std::iter::once(props.placeholder.as_str()))
        .max_by_key(|s| s.matches('\n').count())
        .unwrap_or("");

    let on_focus = {
        let open = open.clone();
        Callback::from(move |_: FocusEvent| open.set(true))
    };
    let on_blur = {
        let open = open.clone();
        Callback::from(move |_: FocusEvent| open.set(false))
    };

    let items: Html = props
        .options
        .iter()
        .map(|(key, label)| {
            let key = key.clone();
            let label = label.clone();
            let on_select = props.on_select.clone();
            let open = open.clone();
            html! {
                <a class="dropdown-item"
                   onmousedown={Callback::from(move |_: MouseEvent| {
                       on_select.emit(key.clone());
                       open.set(false);
                   })}>
                    { multiline(&label) }
                </a>
            }
        })
        .collect();

    html! {
        <div class={ if *open { "dropdown is-active" } else { "dropdown" } }>
            // `display:inline-grid` makes the trigger as wide as the widest child.
            <div class="dropdown-trigger" style="display:inline-grid">
                // Width sizer: longest single line with white-space:nowrap prevents
                // any wrapping, so the grid cell is at least as wide as this line.
                <span style="
                    grid-area: 1/1;
                    visibility: hidden;
                    overflow: hidden;
                    pointer-events: none;
                    white-space: nowrap;
                    padding: calc(.5em - 1px) 2.75em calc(.5em - 1px) calc(1em - 1px);
                    border: 1px solid transparent;
                    line-height: 1.5;
                    box-sizing: border-box;
                ">
                    { sizer_width }
                </span>
                // Height sizer: tallest option rendered with multiline() so <br/>
                // elements force the correct number of rows.
                <span style="
                    grid-area: 1/1;
                    visibility: hidden;
                    overflow: hidden;
                    pointer-events: none;
                    white-space: nowrap;
                    padding: calc(.5em - 1px) 2.75em calc(.5em - 1px) calc(1em - 1px);
                    border: 1px solid transparent;
                    line-height: 1.5;
                    box-sizing: border-box;
                ">
                    { multiline(sizer_tallest) }
                </span>
                <button
                    class="button is-justify-content-space-between"
                    style="grid-area:1/1; width:100%; height:100%"
                    onfocus={on_focus}
                    onblur={on_blur}
                >
                    <span>{ multiline(current_label_str) }</span>
                    <span class="material-symbols-outlined">{ "expand_more" }</span>
                </button>
            </div>
            // min-width:100% keeps the menu at least as wide as the button.
            // max-height + overflow-y prevent it running off the bottom of the screen.
            <div class="dropdown-menu" style="min-width:100%">
                <div class="dropdown-content" style="max-height:60vh; overflow-y:auto">
                    { items }
                </div>
            </div>
        </div>
    }
}
