use cursive::Cursive;
use cursive::event::{Event, Key};
use cursive::view::Margins;
use cursive::views::{Dialog, LinearLayout, OnEventView, RadioGroup, TextView};
use std::cmp::PartialEq;
use std::fmt::Display;
use std::sync::mpsc;
use super::processor::Msg;
use weaver_db::config::{Channel, Content, OutputKind};

fn all_channel() -> Vec<Channel> {
    vec![
        Channel::Print,
        Channel::Run,
        Channel::Copy,
    ]
}

fn all_content() -> Vec<Content> {
    vec![
        Content::Command,
        Content::PathWithCommand,
        Content::Path
    ]
}

/// Create radio buttons for all the `values`, select the one matching `initial`.
fn create_radio_group<T>(container: &mut LinearLayout, values: Vec<T>, initial: &T) -> RadioGroup<T>
    where T: Display + PartialEq + 'static {
    let mut group: RadioGroup<T> = RadioGroup::new();

    for k in values {
        let is_selected = *initial == k;
        let label = format!("{}", k);
        let run = group.button(k, label);
        let run = if is_selected {
            run.selected()
        } else {
            run
        };
        container.add_child(run);
    };

    return group;
}

/// Display the Outpu selector with the current state selected.
pub fn show_output_selection(siv: &mut Cursive, kind: OutputKind, ch: mpsc::Sender<Msg>) {
    let mut output_pane = LinearLayout::vertical();

    output_pane.add_child(TextView::new("Output content:"));
    let content_group = create_radio_group(&mut output_pane,
                                           all_content(),
                                           &kind.content);

    output_pane.add_child(TextView::new("Output channel:"));
    let channel_group = create_radio_group(&mut output_pane,
                                           all_channel(),
                                           &kind.channel);
    output_pane.add_child(TextView::new("<ESC> to exit"));

    let esc_handler = {

        // Clone our own handles to data that is needed later.
        let my_channel = channel_group.clone();
        let my_content = content_group.clone();
        let channel = ch.clone();

        // Build the actual handler and take over the above UI handles.
        // We need to use the handles when invoked to get the value at that time.
        move |s: &mut Cursive| {
            let kind = OutputKind {
                channel: (&*my_channel.selection()).clone(),
                content: (&*my_content.selection()).clone(),
            };

            channel.send(Msg::SelectKind(kind)).expect("Send SelectKind message");
            s.pop_layer();
        }
    };
    siv.add_layer(
        OnEventView::new(
            Dialog::new()
                .title("Change output kind")
                .content(output_pane)
                .padding(Margins::new(2, 2, 2, 2))
        ).on_event(Event::Key(Key::Esc), esc_handler),
    )
}

