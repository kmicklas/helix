use super::*;

use helix_core::hashmap;
use helix_term::{application::Application, keymap};
use helix_view::document::Mode;

async fn test_text(
    app_builder: AppBuilder,
    input: &str,
    keys: &str,
    expected: &str,
) -> anyhow::Result<()> {
    let test_case: TestCase = (input, keys, expected, LineFeedHandling::AsIs).into();
    let app = app_builder.build()?;

    test_key_sequence_with_input_text(
        Some(app),
        test_case.clone(),
        &|app| {
            assert_eq!(&test_case.out_text, helix_view::doc!(app.editor).text());
            assert_eq!(Mode::Insert, app.editor.mode());
        },
        false,
    )
    .await
}

fn non_modal_builder() -> AppBuilder {
    let mut config = Config::default();
    config.editor.auto_completion = false;
    AppBuilder::new()
        .with_config(config)
        .with_mode(Mode::Insert)
}

#[tokio::test(flavor = "multi_thread")]
async fn inserts_without_entering_insert_mode() -> anyhow::Result<()> {
    test_text(non_modal_builder(), "#[|]#", "abc", "abc#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn comma_menu_preserves_literal_comma_sequences() -> anyhow::Result<()> {
    test_text(non_modal_builder(), "#[|]#", "a,,b, c", "a,b, c#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_replaces_multiple_selections() -> anyhow::Result<()> {
    test_text(
        non_modal_builder().with_mode(Mode::Select),
        "#[foo|]# x #(foo|)#",
        "xy",
        "xy#[|]# x xy#(|)#",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn comma_menu_escapes_replace_multiple_selections() -> anyhow::Result<()> {
    test_text(
        non_modal_builder().with_mode(Mode::Select),
        "#[foo|]# x #(foo|)#",
        ",,x",
        ",x#[|]# x ,x#(|)#",
    )
    .await?;
    test_text(
        non_modal_builder().with_mode(Mode::Select),
        "#[foo|]# x #(foo|)#",
        ", ",
        ", #[|]# x , #(|)#",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn shift_movement_selects_from_each_cursor() -> anyhow::Result<()> {
    test_text(
        non_modal_builder(),
        "ab#[|]#cd ab#(|)#cd",
        "<S-right>x",
        "abx#[|]#d abx#(|)#d",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn set_mark_extends_with_regular_movement() -> anyhow::Result<()> {
    test_text(
        non_modal_builder(),
        "ab#[|]#cd ab#(|)#cd",
        "<C-space><right><right>x",
        "abx#[|]# abx#(|)#",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn configured_select_regex_replaces_all_matches() -> anyhow::Result<()> {
    let mut config = Config::default();
    config.editor.auto_completion = false;
    config.keys.insert(
        Mode::Insert,
        keymap!({ "Insert mode"
            "C-b" => select_regex,
        }),
    );

    test_text(
        AppBuilder::new()
            .with_config(config)
            .with_mode(Mode::Insert),
        "#[foo x foo|]#",
        "<C-b>foo<ret>x",
        "x x x#[|]#",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn commands_end_the_current_insert_undo_batch() -> anyhow::Result<()> {
    let mut config = Config::default();
    config.editor.auto_completion = false;
    config.keys.insert(
        Mode::Insert,
        keymap!({ "Insert mode"
            "C-z" => undo,
        }),
    );

    test_text(
        AppBuilder::new()
            .with_config(config)
            .with_mode(Mode::Insert),
        "#[|]#",
        "abc<left>Z<C-z>",
        "ab#[|]#c",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn idle_timeout_ends_the_current_insert_undo_batch() -> anyhow::Result<()> {
    let mut config = Config::default();
    config.editor.auto_completion = false;
    config.keys.insert(
        Mode::Insert,
        keymap!({ "Insert mode"
            "C-z" => undo,
        }),
    );

    let mut app = AppBuilder::new()
        .with_config(config)
        .with_input_text("#[|]#")
        .with_mode(Mode::Insert)
        .build()?;
    let after_typing = |app: &Application| {
        assert_eq!("abc", helix_view::doc!(app.editor).text().to_string());
    };
    let after_undo = |app: &Application| {
        assert_eq!("abc", helix_view::doc!(app.editor).text().to_string());
        assert_eq!(Mode::Insert, app.editor.mode());
    };

    test_key_sequences(
        &mut app,
        vec![
            (Some("abc"), Some(&after_typing)),
            (Some("Z<C-z>"), Some(&after_undo)),
        ],
        false,
    )
    .await
}
