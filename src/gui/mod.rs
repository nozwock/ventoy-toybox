pub mod app;

#[derive(Default)]
struct PromptDialog {
    visible: bool,
    title: String,
    text: String,
}

#[derive(Default, PartialEq, Debug)]
enum VentoyUpdateFrames {
    #[default]
    FoundRelease,
    Downloading,
    Done,
    Failed,
}
